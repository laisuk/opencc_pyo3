// Import OpenCC crate and PyO3 for Python bindings
use opencc_fmmseg;
use opencc_fmmseg::OpenCC as _OpenCC;
use pyo3::prelude::*;

/// List of supported OpenCC conversion configurations.
/// These correspond to different Chinese conversion schemes, e.g.:
/// - "s2t": Simplified to Traditional
/// - "t2s": Traditional to Simplified
/// - "s2tw": Simplified to Traditional (Taiwan Standard)
/// - ...and others (see README for full list)
const CONFIG_LIST: [&str; 16] = [
    "s2t", "t2s", "s2tw", "tw2s", "s2twp", "tw2sp", "s2hk", "hk2s", "t2tw", "tw2t", "t2twp",
    "tw2tp", "t2hk", "hk2t", "t2jp", "jp2t",
];

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
        let config_str = match config {
            Some(c) if CONFIG_LIST.contains(&c) => c.to_string(),
            _ => "s2t".to_string(),
        };
        OpenCC {
            opencc,
            config: config_str,
        }
    }

    /// Convert input text using the current configuration.
    ///
    /// # Arguments
    /// - `input`: The input string to convert.
    /// - `punctuation`: Whether to also convert punctuation.
    ///
    /// # Returns
    /// The converted string.
    fn convert(&self, input: &str, punctuation: bool) -> String {
        self.opencc.convert(input, &self.config, punctuation)
    }

    /// Detect the code of the input text.
    ///
    /// # Arguments
    /// - `input`: The input string to check.
    ///
    /// # Returns
    /// An integer code representing the detected text type.
    fn zho_check(&self, input: &str) -> i32 {
        self.opencc.zho_check(input)
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
}
