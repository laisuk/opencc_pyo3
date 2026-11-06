mod cjk_text;
mod punct_sets;
mod reflow_helper;

use opencc_fmmseg::{
    CustomDictFileSpec, CustomDictMode, CustomDictSpec, DetofuLevel, DetofuMap, DictSlot,
    DictionaryMaxlength, OpenCC as _OpenCC, OpenccConfig,
};
use pyo3::exceptions::{PyIOError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::PyType;
use reflow_helper::reflow_cjk_paragraphs;
use std::path::PathBuf;

impl OpenCC {
    #[inline]
    fn apply_config_internal(&mut self, config: &str) {
        match OpenccConfig::try_from(config) {
            Ok(cfg) => {
                self.config = cfg.as_str().to_owned();
                self.last_error.clear();
            }
            Err(_) => {
                self.config = OpenccConfig::S2t.as_str().to_owned();
                self.last_error = format!("Invalid config '{}', reverted to 's2t'", config);
            }
        }
    }
}

/// Convert Chinese text using OpenCC dictionaries.
///
/// The config property holds the active canonical code. Assign a new code
/// to change configurations. Unknown codes select "s2t" and record a message
/// accessible through get_last_error().
#[pyclass]
#[pyo3(subclass)]
struct OpenCC {
    opencc: _OpenCC,
    #[pyo3(get)]
    config: String,
    last_error: String,
}

/// Python methods for the OpenCC class.
#[pymethods]
impl OpenCC {
    /// Create a new OpenCC instance.
    ///
    /// # Arguments
    /// - `config` (optional): The conversion configuration string.
    ///   If not provided or invalid, defaults to "s2t".
    /// - `preserve_ids` (optional): Preserve characters inside Unicode IDS
    ///   structures during conversion. Defaults to `false`.
    #[new]
    #[pyo3(signature = (config=None, preserve_ids=false))]
    fn new(config: Option<&str>, preserve_ids: bool) -> Self {
        let mut opencc = _OpenCC::new();
        opencc.set_preserve_ids(preserve_ids);

        let (config_enum, error_str) = parse_config_or_default(config);

        OpenCC {
            opencc,
            config: config_enum.as_str().to_string(),
            last_error: error_str,
        }
    }

    /// Return the active canonical configuration code.
    fn get_config(&self) -> &str {
        &self.config
    }

    /// Set the active configuration using the same rules as apply_config().
    #[setter]
    fn set_config(&mut self, config: &str) -> PyResult<()> {
        self.apply_config_internal(config);
        Ok(())
    }

    /// Set the active configuration from a case-insensitive code.
    ///
    /// An unknown code selects "s2t" and records a message in get_last_error().
    /// Applying a valid code clears that message.
    fn apply_config(&mut self, config: &str) -> PyResult<()> {
        self.apply_config_internal(config);
        Ok(())
    }

    /// Return all supported canonical configuration codes.
    #[staticmethod]
    fn supported_configs() -> Vec<&'static str> {
        OpenccConfig::ALL.iter().map(|c| c.as_str()).collect()
    }

    /// Return whether the configuration code is supported.
    #[staticmethod]
    fn is_valid_config(config: &str) -> bool {
        OpenccConfig::is_valid_config(config)
    }

    /// Return the current configuration error message, or an empty string.
    fn get_last_error(&self) -> &str {
        &self.last_error
    }

    /// Convert input_text with the active configuration.
    ///
    /// Set punctuation=True to apply the configuration's punctuation mappings.
    #[pyo3(signature = (input_text, punctuation=false))]
    fn convert(&self, input_text: &str, punctuation: bool) -> String {
        self.opencc.convert(input_text, &self.config, punctuation)
    }

    /// Estimate the Chinese script used in input_text.
    ///
    /// Return 1 for Traditional Chinese, 2 for Simplified Chinese, or 0
    /// when the script is undetermined.
    fn zho_check(&self, input_text: &str) -> i32 {
        self.opencc.zho_check(input_text)
    }

    /// Create a converter from embedded dictionaries and in-memory mappings.
    ///
    /// Each spec supplies a slot and a list of (source, target) pairs. The mode
    /// defaults to "append". Use "override" to replace the entire slot.
    /// Invalid slots or modes raise ValueError.
    #[classmethod]
    #[pyo3(signature = (config=None, specs=None))]
    fn from_dicts(
        _cls: &Bound<'_, PyType>,
        config: Option<&str>,
        specs: Option<Vec<PyCustomDictSpec>>,
    ) -> PyResult<Self> {
        opencc_from_dicts(config, specs)
    }

    /// Create a converter from embedded dictionaries and UTF-8 dictionary files.
    ///
    /// Each spec supplies a slot and a list of file paths. Each mapping uses
    /// source<TAB>target. The mode defaults to "append". Use "override" to replace
    /// the entire slot. Invalid slots, modes, or dictionary files raise ValueError.
    #[classmethod]
    #[pyo3(signature = (config=None, specs=None))]
    fn from_dict_files(
        _cls: &Bound<'_, PyType>,
        config: Option<&str>,
        specs: Option<Vec<PyCustomDictFileSpec>>,
    ) -> PyResult<Self> {
        opencc_from_dict_files(config, specs)
    }

    /// Return canonical slot names for custom dictionary specifications.
    #[staticmethod]
    fn available_slots() -> Vec<&'static str> {
        DictSlot::ALL
            .iter()
            .map(|slot| slot.canonical_name())
            .collect()
    }

    // CJK Compatibility Ideograph Normalization and DeTofu

    /// Map CJK Compatibility Ideographs to their unified forms.
    ///
    /// For example, "金" becomes "金". Apply this optional pass before convert()
    /// when the input contains compatibility ideographs.
    fn normalize_compat(&self, text: &str) -> String {
        self.opencc.normalize_compat(text)
    }

    /// Apply extended compatibility mappings and normalize CJK ideographs.
    ///
    /// This optional pre-processing pass includes the mappings used by
    /// normalize_compat().
    fn normalize_compat_extended(&self, text: &str) -> String {
        self.opencc.normalize_compat_extended(text)
    }

    /// Replace mapped CJK extension characters with display fallbacks.
    ///
    /// The level selects the first extension block to process. "all" starts
    /// at ExtB. ExtB through ExtI and their short forms B through I are
    /// case-insensitive and accept hyphens or underscores.
    /// An unsupported level raises ValueError.
    #[pyo3(signature = (text, level = "all"))]
    fn detofu(&self, text: &str, level: &str) -> PyResult<String> {
        let level = parse_detofu_level(level)?;
        Ok(DetofuMap::builtin(level).detofu(text))
    }

    /// Apply DeTofu with custom mappings from a UTF-8 file.
    ///
    /// Each line contains a source character, a fallback character, and an
    /// extension label separated by tabs. Custom entries take precedence over
    /// built-in entries for the same source character. File errors raise OSError.
    /// A missing path or an unsupported level raises ValueError.
    #[pyo3(signature = (text, level = "all", path = None))]
    fn detofu_with_custom_file(
        &self,
        text: &str,
        level: &str,
        path: Option<&str>,
    ) -> PyResult<String> {
        let path = path.ok_or_else(|| {
            PyValueError::new_err("detofu_with_custom_file requires a custom file path")
        })?;
        let level = parse_detofu_level(level)?;

        let map = DetofuMap::builtin(level)
            .with_custom_file(path)
            .map_err(|e| PyIOError::new_err(e.to_string()))?;

        Ok(map.detofu(text))
    }

    /// Apply DeTofu with custom (source, fallback) character pairs.
    ///
    /// Supply pairs as a list of tuples containing single-character strings.
    /// Custom entries take precedence over built-in entries for the same source
    /// character. Missing pairs or an unsupported level raises ValueError.
    #[pyo3(signature = (text, level = "all", pairs = None))]
    fn detofu_with_custom_pairs(
        &self,
        text: &str,
        level: &str,
        pairs: Option<Vec<(char, char)>>,
    ) -> PyResult<String> {
        let pairs = pairs.ok_or_else(|| {
            PyValueError::new_err("detofu_with_custom_pairs requires custom character pairs")
        })?;
        let level = parse_detofu_level(level)?;

        Ok(DetofuMap::builtin(level)
            .with_custom_pairs(&pairs)
            .detofu(text))
    }
}

// New: strut + helpers - Custom Dictionary
#[derive(FromPyObject)]
struct PyCustomDictSpec {
    #[pyo3(item)]
    slot: String,

    #[pyo3(item)]
    pairs: Vec<(String, String)>,

    #[pyo3(item)]
    mode: Option<String>,
}

#[derive(FromPyObject)]
struct PyCustomDictFileSpec {
    #[pyo3(item)]
    slot: String,

    #[pyo3(item)]
    files: Vec<String>,

    #[pyo3(item)]
    mode: Option<String>,
}

/// Builds an [`OpenCC`] instance from the default embedded dictionaries plus
/// in-memory custom dictionary specifications.
///
/// This is the shared internal implementation behind the Python
/// `OpenCC.from_dicts()` class method.
///
/// The default dictionaries are loaded first via
/// [`DictionaryMaxlength::from_zstd`], then custom mappings are applied with
/// `with_custom_dicts()`.
///
/// Invalid custom dictionary slots or modes return a Python `ValueError`.
fn opencc_from_dicts(
    config: Option<&str>,
    specs: Option<Vec<PyCustomDictSpec>>,
) -> PyResult<OpenCC> {
    let (config_enum, error_str) = parse_config_or_default(config);

    let rust_specs = specs
        .unwrap_or_default()
        .into_iter()
        .map(|s| {
            Ok(CustomDictSpec {
                slot: parse_slot(&s.slot)?,
                pairs: s.pairs,
                mode: parse_mode(s.mode.as_deref())?,
            })
        })
        .collect::<PyResult<Vec<_>>>()?;

    let dict = DictionaryMaxlength::from_zstd()
        .map_err(to_py_value_error)?
        .with_custom_dicts(&rust_specs)
        .map_err(to_py_value_error)?;

    Ok(OpenCC {
        opencc: _OpenCC::from_dictionary(dict),
        config: config_enum.as_str().to_string(),
        last_error: error_str,
    })
}

/// Builds an [`OpenCC`] instance from the default embedded dictionaries plus
/// external custom dictionary files.
///
/// This is the shared internal implementation behind the Python
/// `OpenCC.from_dict_files()` class method.
///
/// The default dictionaries are loaded first via
/// [`DictionaryMaxlength::from_zstd`], then user-provided dictionary files are
/// applied with `with_custom_dict_files()`.
///
/// Invalid custom dictionary slots, modes, file paths, or malformed dictionary
/// lines return a Python `ValueError`.
fn opencc_from_dict_files(
    config: Option<&str>,
    specs: Option<Vec<PyCustomDictFileSpec>>,
) -> PyResult<OpenCC> {
    let (config_enum, error_str) = parse_config_or_default(config);

    let rust_specs = specs
        .unwrap_or_default()
        .into_iter()
        .map(|s| {
            Ok(CustomDictFileSpec {
                slot: parse_slot(&s.slot)?,
                files: s.files.into_iter().map(PathBuf::from).collect(),
                mode: parse_mode(s.mode.as_deref())?,
            })
        })
        .collect::<PyResult<Vec<_>>>()?;

    let dict = DictionaryMaxlength::from_zstd()
        .map_err(to_py_value_error)?
        .with_custom_dict_files(&rust_specs)
        .map_err(to_py_value_error)?;

    Ok(OpenCC {
        opencc: _OpenCC::from_dictionary(dict),
        config: config_enum.as_str().to_string(),
        last_error: error_str,
    })
}

fn to_py_value_error<E: std::fmt::Display>(err: E) -> PyErr {
    PyValueError::new_err(err.to_string())
}

/// Parses a custom dictionary merge mode.
///
/// Supported modes:
///
/// - `append`
/// - `override`
///
/// Invalid modes return a Python `ValueError`.
fn parse_mode(mode: Option<&str>) -> PyResult<CustomDictMode> {
    match mode
        .unwrap_or("append")
        .trim()
        .to_ascii_lowercase()
        .as_str()
    {
        "append" => Ok(CustomDictMode::Append),
        "override" => Ok(CustomDictMode::Override),
        other => Err(PyValueError::new_err(format!(
            "Invalid custom dict mode: {other}. Expected: append|override"
        ))),
    }
}

/// Parses a custom dictionary slot name into a [`DictSlot`].
///
/// Slot names are ASCII case-insensitive, and surrounding whitespace is
/// ignored. Canonical names such as `STPhrases`, `TWPhrasesRev`, and
/// `HKVariantsRevPhrases` are accepted.
///
/// An optional `.txt` suffix is also accepted for compatibility. Deprecated
/// `JPShinjitai*` aliases are accepted while they remain supported by
/// [`DictSlot::from_name_ignore_ascii_case`].
///
/// Invalid slot names return a Python `ValueError`.
fn parse_slot(slot: &str) -> PyResult<DictSlot> {
    let trimmed = slot.trim();

    let name = if trimmed.to_ascii_lowercase().ends_with(".txt") {
        &trimmed[..trimmed.len() - 4]
    } else {
        trimmed
    };

    DictSlot::from_name_ignore_ascii_case(name).ok_or_else(|| {
        PyValueError::new_err(format!(
            "Invalid custom dictionary slot: {slot}. \
             Expected a canonical slot name like 'STPhrases', \
             'TWPhrasesRev', or 'HKVariantsRevPhrases'."
        ))
    })
}

/// Parses an OpenCC configuration string.
///
/// Invalid configurations automatically fall back to `s2t`
/// and return an associated warning message.
fn parse_config_or_default(config: Option<&str>) -> (OpenccConfig, String) {
    match config {
        Some(c) => match OpenccConfig::try_from(c) {
            Ok(cfg) => (cfg, String::new()),
            Err(_) => (
                OpenccConfig::S2t,
                format!("Invalid config '{}', reverted to 's2t'", c),
            ),
        },
        None => (OpenccConfig::S2t, String::new()),
    }
}

/// Parse a Python/CLI DeTofu level string into the backend `DetofuLevel`.
///
/// This parser is deliberately permissive at the public API boundary:
///
/// - case-insensitive input is accepted;
/// - hyphens and underscores are ignored, so `ext-c`, `ext_c`, and `ExtC`
///   all resolve to `DetofuLevel::ExtC`;
/// - compact block names `B` through `I` are accepted;
/// - `all` is accepted as the public default and resolves to
///   `DetofuLevel::ExtB`, meaning ExtB and every later extension block.
///
/// Invalid values become a Python `ValueError` with the original input
/// preserved in the error message.
fn parse_detofu_level(level: &str) -> PyResult<DetofuLevel> {
    let normalized = level.to_ascii_lowercase().replace(['-', '_'], "");
    match normalized.as_str() {
        "all" => Ok(DetofuLevel::ExtB),
        "b" | "extb" => Ok(DetofuLevel::ExtB),
        "c" | "extc" => Ok(DetofuLevel::ExtC),
        "d" | "extd" => Ok(DetofuLevel::ExtD),
        "e" | "exte" => Ok(DetofuLevel::ExtE),
        "f" | "extf" => Ok(DetofuLevel::ExtF),
        "g" | "extg" => Ok(DetofuLevel::ExtG),
        "h" | "exth" => Ok(DetofuLevel::ExtH),
        "i" | "exti" => Ok(DetofuLevel::ExtI),
        _ => Err(PyValueError::new_err(format!(
            "Invalid detofu level '{}'. Expected all, ExtB, ExtC, ExtD, ExtE, ExtF, ExtG, ExtH, or ExtI.",
            level
        ))),
    }
}

/// Python module definition for opencc_pyo3.
/// Exposes the OpenCC class to Python.
#[pymodule]
fn opencc_pyo3(m: &Bound<PyModule>) -> PyResult<()> {
    m.add_class::<OpenCC>()?;
    m.add_function(wrap_pyfunction!(reflow_cjk_paragraphs, m)?)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test for the zho_check method.
    #[test]
    fn test_zho_check() {
        let opencc = OpenCC::new(Option::from(""), false);
        let text = "春眠不觉晓，处处闻啼鸟";
        let text_code = opencc.zho_check(text);
        let expected = 2;
        assert_eq!(text_code, expected);
    }

    // Test for supported config list
    #[test]
    fn test_get_supported_list() {
        let configs = OpenCC::supported_configs();
        let expected: Vec<_> = OpenccConfig::ALL.iter().map(|c| c.as_str()).collect();
        let actual: Vec<_> = configs.into_iter().collect();
        assert_eq!(actual, expected);
        assert!(OpenccConfig::is_valid_config("t2hkp"));
        assert!(OpenccConfig::is_valid_config("hk2tp"));
    }

    /// Test custom dictionary slot parsing through the upstream DictSlot SSOT.
    #[test]
    fn test_parse_slot() {
        assert_eq!(parse_slot("STPhrases").unwrap(), DictSlot::STPhrases);
        assert_eq!(parse_slot("  stphrases  ").unwrap(), DictSlot::STPhrases);
        assert_eq!(parse_slot("STPhrases.txt").unwrap(), DictSlot::STPhrases);
        assert!(parse_slot("NotASlot").is_err());
    }

    /// Test that the Python-facing slot list exactly mirrors DictSlot::ALL.
    #[test]
    fn test_available_slots() {
        let expected: Vec<_> = DictSlot::ALL
            .iter()
            .map(|slot| slot.canonical_name())
            .collect();

        assert_eq!(OpenCC::available_slots(), expected);
        assert_eq!(expected.len(), 25);
    }

    #[test]
    fn test_reflow_drawing_box() {
        let input = "\
物品准备：
购物帐单（三叔笔记复印）：
名字 数量
──────────────
洛阳铲头 5 个

──────────────
攀山绳 200 米
──────────────
林德大号开山刀 2 把
（加厚的）
──────────────
";

        // Adjust the path if your function lives elsewhere
        // e.g. crate::reflow::reflow_cjk_paragraphs
        let output = crate::reflow_cjk_paragraphs(
            input, /* add_pdf_page_header = */ false, /* compact = */ false,
        )
        .expect("Failed to reflow CJK paragraphs");

        // Print so you can see it when running `cargo test -- --nocapture`
        println!("===== INPUT =====");
        println!("{}", input.replace('\n', "\\n\n"));

        println!("===== REFLOWED =====");
        println!("{}", output.replace('\n', "\\n\n"));

        // Keep test "real"
        assert!(!output.is_empty());
    }

    /// Test custom in-memory dictionary injection for STPhrases.
    #[test]
    fn test_from_dicts_custom_st_phrases_palantir() {
        let specs = vec![PyCustomDictSpec {
            slot: "STPhrases".to_string(),
            pairs: vec![("帕兰蒂尔".to_string(), "柏蘭蒂爾".to_string())],
            mode: Some("append".to_string()),
        }];

        let opencc = opencc_from_dicts(None, Some(specs))
            .expect("from_dicts should create OpenCC with custom STPhrases");

        let output = opencc.convert("帕兰蒂尔是一家公司", false);

        assert_eq!(output, "柏蘭蒂爾是一家公司");
    }

    /// Test custom dictionary file injection for STPhrases.
    #[test]
    fn test_from_dict_files_custom_st_phrases_palantir() {
        let file_path = std::env::temp_dir().join("opencc_pyo3_custom_st_phrases_palantir.txt");

        std::fs::write(&file_path, "帕兰蒂尔\t柏蘭蒂爾\n")
            .expect("failed to write temporary custom dictionary file");

        let specs = vec![PyCustomDictFileSpec {
            slot: "STPhrases".to_string(),
            files: vec![file_path.to_string_lossy().to_string()],
            mode: Some("append".to_string()),
        }];

        let opencc = opencc_from_dict_files(None, Some(specs))
            .expect("from_dict_files should create OpenCC with custom STPhrases");

        let output = opencc.convert("帕兰蒂尔是一家公司", false);

        assert_eq!(output, "柏蘭蒂爾是一家公司");

        let _ = std::fs::remove_file(file_path);
    }
}
