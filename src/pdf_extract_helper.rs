use pdf_extract::Document;
use pyo3::{exceptions, pyfunction, Py, PyAny, PyResult, Python};

/// Extracts plain text from a PDF file.
///
/// This uses the pure-Rust `pdf-extract` crate. It works well for many PDFs,
/// but for tricky CJK encodings or missing ToUnicode maps you may want to
/// switch to a PDFium-based backend later.
///
/// Parameters
/// ----------
/// path : str
///     Path to the PDF file on disk.
///
/// Returns
/// -------
/// str
///     Concatenated text of all pages.
#[pyfunction]
pub fn extract_pdf_text(path: &str) -> PyResult<String> {
    let text = pdf_extract::extract_text(path).map_err(|e| {
        exceptions::PyRuntimeError::new_err(format!(
            "Failed to extract text from PDF '{}': {e}",
            path
        ))
    })?;
    Ok(text)
}

/// Extracts plain text from a PDF file, split by pages.
///
/// This uses the pure-Rust `pdf-extract` crate. It returns one string per page,
/// in reading order. This is useful if you want to show a progress bar while
/// processing each page sequentially in Python.
///
/// Parameters
/// ----------
/// path : str
///     Path to the PDF file on disk.
///
/// Returns
/// -------
/// List[str]
///     A list of page texts. `result[i]` is the text of page `i + 1`.
#[pyfunction]
pub fn extract_pdf_text_pages(path: &str) -> PyResult<Vec<String>> {
    let pages = pdf_extract::extract_text_by_pages(path).map_err(|e| {
        exceptions::PyRuntimeError::new_err(format!(
            "Failed to extract text by pages from PDF '{}': {e}",
            path
        ))
    })?;
    Ok(pages)
}

/// Extracts PDF text page-by-page and reports progress to a Python callback.
///
/// For PDFs where `pdf-extract` can see the page tree:
///   - iterates real pages, including blank ones (blank → "").
/// For PDFs where `get_pages()` returns empty:
///   - falls back to `extract_text(path)` and calls the callback once as 1/1.
///
/// callback signature: callback(page_number, total_pages, text)
#[pyfunction]
pub fn extract_pdf_pages_with_callback(path: &str, callback: Py<PyAny>) -> PyResult<()> {
    use pyo3::exceptions;

    let doc = match Document::load(path) {
        Ok(d) => d,

        Err(e) => {
            // Detect file-not-found specifically
            let msg = e.to_string();

            let is_not_found =
                msg.contains("No such file")
                    || msg.contains("cannot find the file")
                    || msg.contains("os error 2");

            if is_not_found {
                return Err(exceptions::PyFileNotFoundError::new_err(path.to_string()));
            }

            // All other errors are real PDF/load errors
            return Err(exceptions::PyRuntimeError::new_err(format!(
                "Failed to open PDF '{}': {e}",
                path
            )));
        }
    };

    let pages = doc.get_pages();
    let total_pages = pages.len();

    // fn normalize_page_text(mut s: String) -> String {
    //     if s.contains('\r') {
    //         s = s.replace("\r\n", "\n").replace('\r', "\n");
    //     }
    //     if s.trim().is_empty() {
    //         return "\n".to_string();
    //     }
    //     let t = s.trim().to_string();
    //     format!("{t}\n\n")
    // }
    fn normalize_page_text(mut s: String) -> String {
        // Normalize newlines
        if s.contains('\r') {
            s = s.replace("\r\n", "\n").replace('\r', "\n");
        }

        // If page is truly empty-ish, keep a blank page marker
        if s.trim_matches('\n').trim().is_empty() {
            return "\n\n".to_string();
        }

        // IMPORTANT: do NOT trim the page text; only trim trailing newlines
        while s.ends_with('\n') {
            s.pop();
        }

        s.push_str("\n\n");
        s
    }

    // Fallback: 0-page tree => single chunk
    if total_pages == 0 {
        eprintln!(
            "Warning: pdf-extract reports 0 pages for '{}'; falling back to single-chunk extract_text().",
            path
        );

        let text = pdf_extract::extract_text(path).map_err(|e| {
            exceptions::PyRuntimeError::new_err(format!(
                "Failed to extract text from PDF '{}': {e}",
                path
            ))
        })?;

        if text.trim().is_empty() {
            return Err(exceptions::PyRuntimeError::new_err(format!(
                "Pure-Rust pdf-extract could not extract any text from '{}'. This PDF likely requires a PDFium-based engine.",
                path
            )));
        }

        let text = normalize_page_text(text);

        return Python::attach(|py| {
            callback.call1(py, (1usize, 1usize, text))?;
            Ok(())
        });
    }

    // Normal path
    let page_numbers: Vec<u32> = pages.keys().copied().collect();

    Python::attach(move |py| -> PyResult<()> {
        for (idx, page_number) in page_numbers.iter().copied().enumerate() {
            let raw = match doc.extract_text(&[page_number]) {
                Ok(t) => t,
                Err(e) => {
                    eprintln!(
                        "Warning: failed to extract text from page {} of '{}': {} — treating as blank page.",
                        page_number, path, e
                    );
                    String::new()
                }
            };

            let text = normalize_page_text(raw);

            // 1-based page index for callback, consistent with your PDFium ctypes
            let page_1_based = idx + 1;
            callback.call1(py, (page_1_based, total_pages, text))?;
        }
        Ok(())
    })
}