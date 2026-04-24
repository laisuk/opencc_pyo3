#!/usr/bin/env python3
"""
OpenCC benchmark runner with CI-friendly output.

Highlights:
- Defaults to benchmarking the locally built `opencc_pyo3` package.
- Validates that bundled sample texts are Simplified Chinese before running.
- Writes stable output filenames so GitHub Actions can upload artifacts easily.
- Can append a short Markdown summary to `$GITHUB_STEP_SUMMARY`.
"""

from __future__ import annotations

import argparse
import csv
import gc
import importlib
import json
import os
import platform
import statistics
import time
from dataclasses import asdict, dataclass
from datetime import datetime, timezone
from pathlib import Path
from typing import Dict, Iterable, List, Optional, Type, Any

# REPO_ROOT = Path(__file__).resolve().parents[1]
# if str(REPO_ROOT) not in sys.path:
#     sys.path.insert(0, str(REPO_ROOT))

SUPPORTED_PACKAGES = {
    "opencc_pyo3": "opencc_pyo3",
    "opencc_jieba_pyo3": "opencc_jieba_pyo3",
    "opencc_purepy": "opencc_purepy",
}

PACKAGE_NAME = "opencc_pyo3"
OPENCC_MODULE: Optional[Type[Any]] = None
OPENCC_AVAILABLE = False


def setup_opencc_package(package_name: str = "opencc_pyo3") -> None:
    """Import the requested OpenCC implementation."""
    global PACKAGE_NAME, OPENCC_MODULE, OPENCC_AVAILABLE

    PACKAGE_NAME = package_name
    module_name = SUPPORTED_PACKAGES.get(package_name)
    if module_name is None:
        OPENCC_AVAILABLE = False
        OPENCC_MODULE = None
        raise ValueError(
            f"Unsupported package '{package_name}'. Supported values: {', '.join(sorted(SUPPORTED_PACKAGES))}"
        )

    try:
        module = importlib.import_module(module_name)
        OPENCC_MODULE = getattr(module, "OpenCC")
        OPENCC_AVAILABLE = True
        print(f"Using package: {package_name}")
    except (ImportError, ModuleNotFoundError) as exc:
        OPENCC_AVAILABLE = False
        OPENCC_MODULE = None
        raise ImportError(
            f"Unable to import '{package_name}' via module '{module_name}': {exc}"
        ) from exc


@dataclass
class BenchmarkResult:
    name: str
    config: str
    text_label: str
    text_size: int
    iterations: int
    total_time: float
    mean_time: float
    median_time: float
    std_dev: float
    min_time: float
    max_time: float
    ops_per_second: float
    chars_per_second: float
    allocation_mb: float = 0.0


@dataclass
class TextValidationResult:
    name: str
    text_size: int
    zho_check: int
    roundtrip_equal: bool


def get_opencc() -> Type[Any]:
    if OPENCC_MODULE is None:
        raise RuntimeError("OpenCC module not initialized")
    return OPENCC_MODULE


def get_system_info() -> dict:
    return {
        "platform": platform.platform(),
        "python": platform.python_version(),
        "cpu": get_cpu_name() or "Unknown",
        "machine": platform.machine(),
        "cores": os.cpu_count() or 0,
    }


def get_cpu_name() -> str:
    cpu = platform.processor()
    if cpu:
        return cpu

    try:
        with open("/proc/cpuinfo", "r", encoding="utf-8") as f:
            for line in f:
                if line.startswith("model name"):
                    return line.split(":", 1)[1].strip()
    except (FileNotFoundError, PermissionError, OSError):
        pass

    return "Unknown"


class OpenCCBenchmark:
    """OpenCC benchmarking suite."""

    CONFIGS = [
        "s2t",
        "t2s",
        "s2tw",
        "tw2s",
        "s2hk",
        "hk2s",
        "s2twp",
        "tw2sp",
        "t2tw",
        "tw2t",
    ]

    TARGET_LENGTHS = {
        "Small": 100,
        "Medium": 1000,
        "Large": 10000,
        "XLarge": 100000,
    }

    BASE_TEXTS = {
        "Small": (
            "这是一个用于测试OpenCC基本功能的简体中文样本文本。"
            "它包含常见词语和标准标点，用来验证转换结果是否稳定可靠。"
            ""
        ),
        "Medium": (
            "这是一个中等长度的简体中文测试段落，专门用于评估OpenCC在处理中等规模文本时的性能表现。"
            "文本包含常见词汇、技术描述和日常表达，能够覆盖简繁转换中比较常见的情况。"
            "通过重复但保持语义自然的句子，我们可以更稳定地观察吞吐量、延迟和结果一致性。"
        ),
        "Large": (
            "这是一个大型简体中文测试文本，用于评估OpenCC在处理大量文本时的性能和稳定性。"
            "在真实应用中，系统可能需要转换文章、评论、产品说明、知识库内容和批量导出的历史数据。"
            "因此，这段样本文本会持续重复自然语言内容，以模拟较长文档在持续转换过程中的典型负载。"
            "文本仍然保持简体中文书写习惯，避免混入繁体字，以便让基准结果更加可信。"
        ),
        "XLarge": (
            "这是一个超大型的简体中文压力测试文本，专门用于在持续负载下评估OpenCC的极限性能表现。"
            "它模拟长篇文档、批量转换任务和服务端连续请求等场景，重点观察在高字符数量下的处理速度与稳定性。"
            "为了让基准测试更贴近真实输入，文本使用自然的现代汉语叙述，并刻意保持为简体中文，"
            "从而避免因为样本混杂而影响简繁转换性能数据的可解释性。"
        ),
    }

    def __init__(self) -> None:
        self.results: List[BenchmarkResult] = []
        self.validation_results: List[TextValidationResult] = []
        self.test_texts = self._build_test_texts()

    def _build_test_texts(self) -> Dict[str, str]:
        texts: Dict[str, str] = {}
        for size_name, target_length in self.TARGET_LENGTHS.items():
            seed = self.BASE_TEXTS[size_name]
            if not seed:
                raise ValueError(f"Base text for {size_name} cannot be empty")
            repeated = (seed * ((target_length // len(seed)) + 2))[:target_length]
            texts[size_name] = repeated
        return texts

    def print_text_lengths(self) -> None:
        for size_name, target_length in self.TARGET_LENGTHS.items():
            actual_length = len(self.test_texts[size_name])
            print(f"Text size '{size_name}': {actual_length} characters (target: {target_length})")

    def validate_test_texts(self) -> List[TextValidationResult]:
        if not OPENCC_AVAILABLE:
            raise ImportError("OpenCC package is not available")

        checker = get_opencc()("s2t")
        reverse = get_opencc()("t2s")
        validations: List[TextValidationResult] = []

        for name, text in self.test_texts.items():
            zho_check = checker.zho_check(text)
            roundtrip_equal = reverse.convert(checker.convert(text)) == text
            validations.append(
                TextValidationResult(
                    name=name,
                    text_size=len(text),
                    zho_check=zho_check,
                    roundtrip_equal=roundtrip_equal,
                )
            )

        self.validation_results = validations
        return validations

    def print_validation_table(self) -> None:
        if not self.validation_results:
            print("No text validation results available.")
            return

        print("\nTEXT VALIDATION")
        print("=" * 72)
        print(f"{'Text':<10} {'Chars':<8} {'zho_check':<10} {'Roundtrip'}")
        print("-" * 72)
        for result in self.validation_results:
            print(
                f"{result.name:<10} {result.text_size:<8} {result.zho_check:<10} {str(result.roundtrip_equal)}"
            )

    @staticmethod
    def _measure_memory() -> float:
        try:
            import psutil
        except ImportError:
            return 0.0

        process = psutil.Process()
        return process.memory_info().rss / 1024 / 1024

    def _run_single_benchmark(
            self,
            config: str,
            text: str,
            text_size_name: str,
            iterations: int,
            warmup: int,
    ) -> BenchmarkResult:
        if not OPENCC_AVAILABLE:
            raise ImportError("OpenCC package is not available")

        converter = get_opencc()(config)

        sample = text[: min(256, len(text))]
        for _ in range(max(warmup, 0)):
            converter.convert(sample)

        gc.collect()
        mem_before = self._measure_memory()

        times: List[float] = []
        for _ in range(iterations):
            start_time = time.perf_counter()
            converter.convert(text)
            times.append(time.perf_counter() - start_time)

        mem_after = self._measure_memory()

        total_time = sum(times)
        mean_time = statistics.mean(times)
        median_time = statistics.median(times)
        std_dev = statistics.stdev(times) if len(times) > 1 else 0.0
        min_time = min(times)
        max_time = max(times)
        ops_per_second = 1.0 / mean_time if mean_time > 0 else 0.0
        chars_per_second = len(text) / mean_time if mean_time > 0 else 0.0

        return BenchmarkResult(
            name=f"Convert_{text_size_name}",
            config=config,
            text_label=text_size_name,
            text_size=len(text),
            iterations=iterations,
            total_time=total_time,
            mean_time=mean_time,
            median_time=median_time,
            std_dev=std_dev,
            min_time=min_time,
            max_time=max_time,
            ops_per_second=ops_per_second,
            chars_per_second=chars_per_second,
            allocation_mb=mem_after - mem_before,
        )

    def run_benchmarks(
            self,
            configs: Optional[Iterable[str]] = None,
            text_sizes: Optional[Iterable[str]] = None,
            iterations: int = 100,
            warmup: int = 5,
            fail_fast: bool = False,
    ) -> List[BenchmarkResult]:
        if not OPENCC_AVAILABLE:
            raise ImportError("OpenCC package is not available")

        selected_configs = list(configs or self.CONFIGS[:4])
        selected_text_sizes = list(text_sizes or self.test_texts.keys())

        print(f"OpenCC Benchmark Suite ({PACKAGE_NAME})")
        print("=" * 72)
        print(f"Package: {PACKAGE_NAME}")
        print(f"Python: {platform.python_version()}")
        print(f"Platform: {platform.platform()}")
        print(f"Processor: {platform.processor()}")
        print(f"Started at: {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}")
        print("=" * 72)

        total_benchmarks = len(selected_configs) * len(selected_text_sizes)
        current = 0
        self.results = []

        for config in selected_configs:
            for text_size_name in selected_text_sizes:
                current += 1
                text = self.test_texts[text_size_name]
                print(
                    f"Running [{current}/{total_benchmarks}]: {config} - {text_size_name} "
                    f"({len(text)} chars, {iterations} iterations)"
                )
                try:
                    result = self._run_single_benchmark(
                        config=config,
                        text=text,
                        text_size_name=text_size_name,
                        iterations=iterations,
                        warmup=warmup,
                    )
                    self.results.append(result)
                    print(
                        f"  Mean: {result.mean_time * 1000:.3f} ms, "
                        f"Ops/sec: {result.ops_per_second:.0f}, "
                        f"Chars/sec: {result.chars_per_second:.0f}"
                    )
                except Exception as exc:
                    print(f"  Error: {exc}")
                    if fail_fast:
                        raise

        return self.results

    def print_results_table(self) -> None:
        if not self.results:
            print("No benchmark results available.")
            return

        print("\nBENCHMARK RESULTS")
        print("=" * 120)
        header = (
            f"{'Method':<20} {'Config':<8} {'TextSize':<10} {'Mean':<12} {'StdDev':<12} "
            f"{'Min':<12} {'Max':<12} {'Ops/sec':<10} {'Chars/sec':<12}"
        )
        print(header)
        print("-" * len(header))

        for result in sorted(self.results, key=lambda item: (item.config, item.text_size)):
            print(
                f"{result.name:<20} {result.config:<8} {result.text_size:<10} "
                f"{result.mean_time * 1000:>8.3f} ms   {result.std_dev * 1000:>8.3f} ms   "
                f"{result.min_time * 1000:>8.3f} ms   {result.max_time * 1000:>8.3f} ms   "
                f"{result.ops_per_second:>8.0f}   {result.chars_per_second:>10.0f}"
            )

    def export_results(self, export_format: str, filename: Path) -> None:
        if not self.results:
            raise ValueError("No benchmark results to export")

        filename.parent.mkdir(parents=True, exist_ok=True)
        result_dicts = [asdict(result) for result in self.results]
        validation_dicts = [asdict(result) for result in self.validation_results]

        if export_format == "json":
            payload = {
                "metadata": {
                    "package": PACKAGE_NAME,
                    "generated_at": datetime.now(timezone.utc).isoformat(),
                    "platform": platform.platform(),
                    "python_version": platform.python_version(),
                    "total_benchmarks": len(self.results),
                },
                "validation": validation_dicts,
                "results": result_dicts,
            }
            filename.write_text(json.dumps(payload, indent=2, ensure_ascii=False), encoding="utf-8")
            print(f"Results exported to {filename}")
            return

        if export_format == "csv":
            with filename.open("w", newline="", encoding="utf-8") as handle:
                writer = csv.writer(handle)
                writer.writerow(
                    [
                        "Name",
                        "Config",
                        "TextLabel",
                        "TextSize",
                        "Iterations",
                        "MeanTimeMs",
                        "MedianTimeMs",
                        "StdDevMs",
                        "MinTimeMs",
                        "MaxTimeMs",
                        "OpsPerSecond",
                        "CharsPerSecond",
                        "AllocationMB",
                    ]
                )
                for result in self.results:
                    writer.writerow(
                        [
                            result.name,
                            result.config,
                            result.text_label,
                            result.text_size,
                            result.iterations,
                            result.mean_time * 1000,
                            result.median_time * 1000,
                            result.std_dev * 1000,
                            result.min_time * 1000,
                            result.max_time * 1000,
                            result.ops_per_second,
                            result.chars_per_second,
                            result.allocation_mb,
                        ]
                    )
            print(f"Results exported to {filename}")
            return

        if export_format == "md":
            filename.write_text(self._build_markdown_report(), encoding="utf-8")
            print(f"Results exported to {filename}")
            return

        raise ValueError(f"Unsupported export format: {export_format}")

    def _build_markdown_report(self) -> str:
        sysinfo = get_system_info()

        lines = [
            f"# Benchmark Results for `{PACKAGE_NAME}`",
            "",
            f"**Generated:** {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}",
            f"**Platform:** {sysinfo['platform']}",
            f"**CPU:** {sysinfo['cpu']}",
            f"**Cores:** {sysinfo['cores']}",
            f"**Python:** {sysinfo['python']}",
            "",
            "## Text Validation",
            "",
            "| Text | Chars | zho_check | Roundtrip |",
            "|------|------:|----------:|-----------|",
        ]

        for result in self.validation_results:
            lines.append(
                f"| {result.name} | {result.text_size} | {result.zho_check} | {result.roundtrip_equal} |"
            )

        lines.extend(
            [
                "",
                "## Benchmark Results",
                "",
                "| Method | Config | Text Size | Mean (ms) | StdDev (ms) | Min (ms) | Max (ms) | Ops/sec | Chars/sec |",
                "|--------|--------|----------:|----------:|------------:|---------:|---------:|--------:|----------:|",
            ]
        )

        for result in sorted(self.results, key=lambda item: (item.config, item.text_size)):
            lines.append(
                f"| {result.name} | {result.config} | {result.text_size} | "
                f"{result.mean_time * 1000:.3f} | {result.std_dev * 1000:.3f} | "
                f"{result.min_time * 1000:.3f} | {result.max_time * 1000:.3f} | "
                f"{result.ops_per_second:.0f} | {result.chars_per_second:.0f} |"
            )

        return "\n".join(lines) + "\n"


def default_output_path(output_dir: Path, package_name: str, export_format: str, ci_mode: bool) -> Path:
    suffix = export_format.lower()
    if ci_mode:
        filename = f"opencc_benchmark_{package_name}.{suffix}"
    else:
        timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
        filename = f"opencc_benchmark_{package_name}_{timestamp}.{suffix}"
    return output_dir / filename


def append_github_step_summary(summary_path: Path, benchmark: OpenCCBenchmark, md_path: Path) -> None:
    lines = [
        "## OpenCC Benchmark",
        "",
        f"- Package: `{PACKAGE_NAME}`",
        f"- Markdown report: `{md_path.as_posix()}`",
        "",
        "### Text Validation",
        "",
        "| Text | zho_check | Roundtrip |",
        "|------|----------:|-----------|",
    ]

    for result in benchmark.validation_results:
        lines.append(f"| {result.name} | {result.zho_check} | {result.roundtrip_equal} |")

    lines.extend(["", "### Fastest Results", "", "| Config | Text | Mean (ms) | Chars/sec |",
                  "|--------|------|----------:|----------:|"])

    for result in sorted(benchmark.results, key=lambda item: item.mean_time)[: min(5, len(benchmark.results))]:
        lines.append(
            f"| {result.config} | {result.text_label} | {result.mean_time * 1000:.3f} | {result.chars_per_second:.0f} |"
        )

    lines.append("")
    summary_path.parent.mkdir(parents=True, exist_ok=True)
    with summary_path.open("a", encoding="utf-8") as handle:
        handle.write("\n".join(lines))


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="OpenCC Benchmark Suite")
    parser.add_argument(
        "--package",
        default="opencc_pyo3",
        choices=sorted(SUPPORTED_PACKAGES),
        help="OpenCC package to benchmark",
    )
    parser.add_argument(
        "--configs",
        nargs="+",
        default=["s2t", "t2s", "s2tw", "tw2s"],
        help="Configurations to benchmark",
    )
    parser.add_argument(
        "--sizes",
        nargs="+",
        default=["Small", "Medium", "Large", "XLarge"],
        help="Text sizes to test",
    )
    parser.add_argument("--iterations", type=int, default=100, help="Iterations per benchmark")
    parser.add_argument("--warmup", type=int, default=5, help="Warmup iterations per benchmark")
    parser.add_argument(
        "--export",
        nargs="+",
        choices=["json", "csv", "md"],
        default=["md"],
        help="One or more export formats",
    )
    parser.add_argument(
        "--output-dir",
        default="bench/out",
        help="Directory for exported benchmark artifacts",
    )
    parser.add_argument("--ci", action="store_true", help="Use CI-friendly defaults and stable filenames")
    parser.add_argument(
        "--github-step-summary",
        action="store_true",
        help="Append a short summary to the path in GITHUB_STEP_SUMMARY",
    )
    parser.add_argument(
        "--validate-text-only",
        action="store_true",
        help="Validate the bundled zh-Hans benchmark texts and exit",
    )
    parser.add_argument("--no-table", action="store_true", help="Skip printing the console result table")
    return parser.parse_args()


def main() -> int:
    args = parse_args()

    if args.ci:
        if args.iterations == 100:
            args.iterations = 30
        if args.warmup == 5:
            args.warmup = 3

    try:
        setup_opencc_package(args.package)
    except Exception as exc:
        print(f"Error: {exc}")
        return 1

    benchmark = OpenCCBenchmark()
    benchmark.print_text_lengths()

    try:
        validations = benchmark.validate_test_texts()
    except Exception as exc:
        print(f"Error while validating sample texts: {exc}")
        return 1

    benchmark.print_validation_table()

    invalid = [result for result in validations if result.zho_check != 2 or not result.roundtrip_equal]
    if invalid:
        print("Sample text validation failed.")
        return 1

    if args.validate_text_only:
        print("Sample text validation completed successfully.")
        return 0

    try:
        results = benchmark.run_benchmarks(
            configs=args.configs,
            text_sizes=args.sizes,
            iterations=args.iterations,
            warmup=args.warmup,
            fail_fast=args.ci,
        )
    except Exception as exc:
        print(f"Benchmark failed: {exc}")
        return 1

    if not results:
        print("No benchmark results generated.")
        return 1

    if not args.no_table:
        benchmark.print_results_table()

    output_dir = Path(args.output_dir)
    export_paths: Dict[str, Path] = {}
    for export_format in args.export:
        export_path = default_output_path(output_dir, args.package, export_format, args.ci)
        benchmark.export_results(export_format, export_path)
        export_paths[export_format] = export_path

    if args.github_step_summary:
        summary_file = os.environ.get("GITHUB_STEP_SUMMARY")
        md_path = export_paths.get("md")
        if summary_file and md_path is not None:
            append_github_step_summary(Path(summary_file), benchmark, md_path)
            print(f"GitHub step summary appended to {summary_file}")
        elif not summary_file:
            print("GITHUB_STEP_SUMMARY is not set; skipped step summary export.")
        else:
            print("Markdown export not requested; skipped step summary export.")

    print("\nBenchmark completed successfully.")
    print(f"Total benchmarks run: {len(results)}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
