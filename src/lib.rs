// Import OpenCC crate and PyO3 for Python bindings
use once_cell::sync::Lazy;
use opencc_fmmseg;
use opencc_fmmseg::OpenCC as _OpenCC;
use pyo3::prelude::*;
use std::collections::HashSet;

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
    ///              If the value is not in the supported list, it will fallback
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
}
