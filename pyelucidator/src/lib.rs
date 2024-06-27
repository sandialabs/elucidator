use elucidator::MemberSpecification;
use elucidator::error::*;
use pyo3::prelude::*;
use pyo3::exceptions::PyValueError;

#[pyclass(name = "MemberSpec")]
struct MemberSpecPy {
    inner: MemberSpecification,
}

#[pymethods]
impl MemberSpecPy {
    #[staticmethod]
    fn from_string(s: &str) -> PyResult<Self> {
        match MemberSpecification::from(s) {
            Ok(o) => Ok(Self { inner: o }),
            Err(e) => {
                Err(PyValueError::new_err("encountered a parsing error"))
            }
        }
    }
}


/// Elucidating my metadata
#[pymodule]
fn pyelucidator(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<MemberSpecPy>()?;
    Ok(())
}
