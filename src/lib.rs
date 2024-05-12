// Import OpenCC crate
use opencc_fmmseg;
use opencc_fmmseg::OpenCC as _OpenCC;
use pyo3::prelude::*;

const CONFIG_LIST: [&str; 16] = [
    "s2t", "t2s", "s2tw", "tw2s", "s2twp", "tw2sp", "s2hk", "hk2s", "t2tw", "tw2t", "t2twp",
    "tw2tp", "t2hk", "hk2t", "t2jp", "jp2t",
];
// Wrap the OpenCC struct in PyO3
#[pyclass(subclass)]
struct OpenCC {
    opencc: _OpenCC,
    #[pyo3(get, set)]
    config: String,
}
// Implement methods for the OpenCCWrapper struct
#[pymethods]
impl OpenCC {
    #[new]
    fn new(config: Option<&str>) -> Self {
        let opencc = _OpenCC::new();
        let config_str = match config {
            Some(c) if CONFIG_LIST.contains(&c) => c.to_string(),
            _ => "s2t".to_string(),
        };
        OpenCC {
            opencc,
            config: config_str.to_string(),
        }
    }

    fn convert(&self, input: &str, punctuation: bool) -> String {
        self.opencc.convert(input, &self.config, punctuation)
    }

    fn zho_check(&self, input: &str) -> i32 {
        self.opencc.zho_check(input)
    }
}

#[pymodule]
fn opencc_pyo3(_py: Python, m: &Bound<PyModule>) -> PyResult<()> {
    m.add_class::<OpenCC>()?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zho_check() {
        let opencc = OpenCC::new(Option::from(""));
        let text = "春眠不觉晓，处处闻啼鸟";
        let text_code = opencc.zho_check(text);
        let expected = 2;
        assert_eq!(text_code, expected);
    }
}
