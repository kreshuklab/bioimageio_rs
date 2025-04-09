use pyo3::prelude::*;

use crate::rdf::Author2;

/// Formats the sum of two numbers as string.
#[pyfunction]
fn sum_as_string(a: usize, b: usize) -> PyResult<String> {
    Ok((a + b).to_string())
}

/// A Python module implemented in Rust.
#[pymodule]
fn bioimg_spec(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(sum_as_string, m)?)?;
    register_author_submodule(m)?;
    Ok(())
}

pub fn register_author_submodule(parent_module: &Bound<'_, PyModule>) -> PyResult<()> {
    let author_module = PyModule::new(parent_module.py(), "author")?;
    author_module.add_class::<Author2>()?;
    parent_module.add_submodule(&author_module)
}

#[pymethods]
impl Author2{
    #[new]
    pub fn new() -> PyResult<Self>{
        Ok(Author2{
            name: "Blerbs".to_owned().try_into().unwrap(),
            affiliation: None,
            email: None,
            github_user: None,
            orcid: None,
        })
    }
}
