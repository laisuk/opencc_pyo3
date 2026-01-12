mod pdf_extract_helper;
mod reflow_helper;

// Import OpenCC crate and PyO3 for Python bindings
use once_cell::sync::Lazy;
use opencc_fmmseg;
use opencc_fmmseg::OpenCC as _OpenCC;
use pyo3::prelude::*;
use std::collections::HashSet;

use pdf_extract_helper::{
    extract_pdf_pages_with_callback, extract_pdf_text, extract_pdf_text_pages,
};
use reflow_helper::reflow_cjk_paragraphs;

/// List of supported OpenCC conversion configurations.
/// These correspond to different Chinese conversion schemes, e.g.:
/// - "s2t": Simplified to Traditional
/// - "t2s": Traditional to Simplified
/// - "s2tw": Simplified to Traditional (Taiwan Standard)
/// - ...and others (see README for full list)
pub static CONFIG_SET: Lazy<HashSet<&'static str>> = Lazy::new(|| {
    [
        "s2t", "t2s", "s2tw", "tw2s", "s2twp", "tw2sp", "s2hk", "hk2s", "t2tw", "tw2t", "t2twp",
        "tw2tp", "t2hk", "hk2t", "t2jp", "jp2t",
    ]
    .iter()
    .copied()
    .collect()
});

/// Python class wrapping the Rust OpenCC struct.
///
/// # Fields
/// - `opencc`: The internal Rust OpenCC instance.
/// - `config`: The conversion configuration string (e.g. "s2t").
///
/// # Python Attributes
/// - `config`: Get/set the current conversion configuration.
#[pyclass]
#[pyo3(subclass)]
struct OpenCC {
    opencc: _OpenCC,
    #[pyo3(get, set)]
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
    #[new]
    #[pyo3(signature = (config=None))]
    fn new(config: Option<&str>) -> Self {
        let opencc = _OpenCC::new();
        let (config_str, error_str) = match config {
            Some(c) if CONFIG_SET.contains(&c) => (c.to_string(), String::new()),
            Some(c) => (
                "s2t".to_string(),
                format!("Invalid config '{}', reverted to 's2t'", c),
            ),
            None => ("s2t".to_string(), String::new()),
        };
        OpenCC {
            opencc,
            config: config_str,
            last_error: error_str,
        }
    }

    /// Convert input text using the current configuration.
    ///
    /// # Arguments
    /// - `input_text`: The input string to convert.
    /// - `punctuation`: Whether to also convert punctuation.
    ///
    /// # Returns
    /// The converted string.
    fn convert(&self, input_text: &str, punctuation: bool) -> String {
        self.opencc.convert(input_text, &self.config, punctuation)
    }

    /// Detect the code of the input text.
    ///
    /// # Arguments
    /// - `input_text`: The input string to check.
    ///
    /// # Returns
    /// An integer code representing the detected text type.
    /// 1 - Traditional Chinese, 2 - Simplified Chinese, 0 - Others
    fn zho_check(&self, input_text: &str) -> i32 {
        self.opencc.zho_check(input_text)
    }

    /// Get the current configuration name.
    ///
    /// # Returns
    /// A string slice representing the currently active OpenCC config,
    /// such as `"s2t"`, `"t2s"`, etc.
    fn get_config(&self) -> &str {
        &self.config
    }

    /// Set the configuration name.
    ///
    /// # Arguments
    /// * `config` - A string slice representing the new configuration to set.
    ///              If the value is not in the supported list, it will fall back
    ///              to `"s2t"` and set an error message.
    ///
    /// # Behavior
    /// - If the config is valid, it will update the current config and clear any previous error.
    /// - If the config is invalid, it sets the config to `"s2t"` and stores an error in `last_error`.
    fn apply_config(&mut self, config: &str) {
        if CONFIG_SET.contains(config) {
            self.config = config.to_string();
            self.last_error.clear();
        } else {
            self.config = "s2t".to_string();
            self.last_error = format!("Invalid config '{}', reverted to 's2t'", config);
        }
    }

    /// Get the most recent error string (if any).
    ///
    /// # Returns
    /// A string slice containing the most recent error message.
    /// If no error occurred, returns an empty string.
    fn get_last_error(&self) -> &str {
        &self.last_error
    }

    /// Get a list of all supported OpenCC configuration codes.
    ///
    /// # Returns
    /// A vector of string slices representing valid configuration codes,
    /// such as `"s2t"`, `"t2s"`, `"s2tw"`, etc.
    #[staticmethod]
    fn supported_configs() -> Vec<&'static str> {
        CONFIG_SET.iter().copied().collect()
    }

    /// Check if a configuration name is valid.
    ///
    /// # Arguments
    /// * `config` - A string slice representing the config to check.
    ///
    /// # Returns
    /// `true` if the config is supported, otherwise `false`.
    #[staticmethod]
    fn is_valid_config(config: &str) -> bool {
        CONFIG_SET.contains(config)
    }
}

/// Python module definition for opencc_pyo3.
/// Exposes the OpenCC class to Python.
#[pymodule]
fn opencc_pyo3(m: &Bound<PyModule>) -> PyResult<()> {
    m.add_class::<OpenCC>()?;
    m.add_function(wrap_pyfunction!(reflow_cjk_paragraphs, m)?)?;
    m.add_function(wrap_pyfunction!(extract_pdf_text, m)?)?;
    m.add_function(wrap_pyfunction!(extract_pdf_text_pages, m)?)?;
    m.add_function(wrap_pyfunction!(extract_pdf_pages_with_callback, m)?)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test for the zho_check method.
    #[test]
    fn test_zho_check() {
        let opencc = OpenCC::new(Option::from(""));
        let text = "春眠不觉晓，处处闻啼鸟";
        let text_code = opencc.zho_check(text);
        let expected = 2;
        assert_eq!(text_code, expected);
    }

    // Test for supported config list
    #[test]
    fn test_get_supported_list() {
        let configs = OpenCC::supported_configs();
        let expected: HashSet<&str> = CONFIG_SET.iter().copied().collect();
        let actual: HashSet<&str> = configs.into_iter().collect();
        assert_eq!(actual, expected);
    }

    /// Test PDF text extraction using a known CJK PDF.
    /// Saves *reflowed* text to `tests/简体字_output.txt` for manual inspection.
    #[test]
    fn test_extract_pdf_text() {
        use std::fs;
        use std::io::Write;
        use std::path::Path;

        // PDF input (relative to crate root)
        let input_path = "tests/简体字.pdf";

        assert!(
            Path::new(input_path).exists(),
            "Test PDF not found at: {}. Make sure the file exists.",
            input_path
        );

        // Extract text
        let text = extract_pdf_text(input_path).expect("Failed to extract text from test PDF");

        // Sanity check: extracted text should not be empty
        assert!(
            !text.trim().is_empty(),
            "PDF extraction returned empty text"
        );

        // Ensure some CJK characters appear (adjust if your sample PDF differs)
        assert!(
            text.contains("字") || text.contains("简") || text.contains("体"),
            "Extracted text does not contain expected CJK characters.\nGot: {}",
            text
        );

        // 🔹 Reflow CJK paragraphs before saving
        // add_pdf_page_header = false, compact = false (blank line between paragraphs)
        let reflowed =
            reflow_cjk_paragraphs(&text, false, false).expect("Failed to reflow CJK paragraphs");

        // Save output to file for manual review
        let output_path = "tests/简体字_output.txt";
        let mut file = fs::File::create(output_path).expect("Failed to create output .txt file");

        file.write_all(reflowed.as_bytes())
            .expect("Failed to write extracted text to output file");

        // Optional: check output file exists and is non-empty
        let out_meta = fs::metadata(output_path).expect("Failed to stat output file");
        assert!(out_meta.len() > 0, "Output text file is empty");
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
}
