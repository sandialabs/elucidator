use pyo3::{exceptions::PyValueError, prelude::*, types::PyDict};

use elucidator::{error::ElucidatorError, value::DataValue};

use elucidator_db::{
    backends::rtree::RTreeDatabase,
    database::{Database, Metadata},
    error::DatabaseError,
};

use std::collections::HashMap;

fn value2obj<'py>(
    py: Python<'py>,
    dv: &HashMap<&str, DataValue>,
) -> PyResult<Bound<'py, PyDict>> {
    let d = PyDict::new_bound(py);

    for (k, v) in dv {
        match v {
            DataValue::Byte(v) => d.set_item(k, v)?,
            DataValue::UnsignedInteger16(v) => d.set_item(k, v)?,
            DataValue::UnsignedInteger32(v) => d.set_item(k, v)?,
            DataValue::UnsignedInteger64(v) => d.set_item(k, v)?,
            DataValue::SignedInteger8(v) => d.set_item(k, v)?,
            DataValue::SignedInteger16(v) => d.set_item(k, v)?,
            DataValue::SignedInteger32(v) => d.set_item(k, v)?,
            DataValue::SignedInteger64(v) => d.set_item(k, v)?,
            DataValue::Float32(v) => d.set_item(k, v)?,
            DataValue::Float64(v) => d.set_item(k, v)?,
            DataValue::Str(v) => d.set_item(k, v)?,
            DataValue::ByteArray(v) => d.set_item(k, v)?,
            DataValue::UnsignedInteger16Array(v) => d.set_item(k, v)?,
            DataValue::UnsignedInteger32Array(v) => d.set_item(k, v)?,
            DataValue::UnsignedInteger64Array(v) => d.set_item(k, v)?,
            DataValue::SignedInteger8Array(v) => d.set_item(k, v)?,
            DataValue::SignedInteger16Array(v) => d.set_item(k, v)?,
            DataValue::SignedInteger32Array(v) => d.set_item(k, v)?,
            DataValue::SignedInteger64Array(v) => d.set_item(k, v)?,
            DataValue::Float32Array(v) => d.set_item(k, v)?,
            DataValue::Float64Array(v) => d.set_item(k, v)?,
        }
    }
    Ok(d)
}

enum ApiError {
    Eluci(ElucidatorError),
    Database(DatabaseError),
}

impl From<ApiError> for PyErr {
    fn from(val: ApiError) -> Self {
        let msg = match &val {
            ApiError::Eluci(e) => format!("ElucidatorError: {e}"),
            ApiError::Database(e) => format!("DatabaseError: {e}"),
        };
        PyValueError::new_err(msg)
    }
}

impl From<DatabaseError> for ApiError {
    fn from(item: DatabaseError) -> Self {
        ApiError::Database(item)
    }
}

impl From<&DatabaseError> for ApiError {
    fn from(item: &DatabaseError) -> Self {
        ApiError::Database(item.clone())
    }
}

#[pyclass]
#[derive(Clone, Debug)]
struct Point {
    x: f64,
    y: f64,
    z: f64,
    t: f64,
}

#[pymethods]
impl Point {
    #[new]
    fn new(x: f64, y: f64, z: f64, t: f64) -> Self {
        Point { x, y, z, t }
    }
}

#[pyclass]
#[derive(Clone, Debug)]
struct BoundingBox {
    a: Point,
    b: Point,
}

#[pymethods]
impl BoundingBox {
    #[new]
    fn new(a: &Point, b: &Point) -> Self {
        BoundingBox {
            a: a.clone(),
            b: b.clone(),
        }
    }
}

#[pyclass]
#[derive(Debug)]
struct Session {
    db: RTreeDatabase,
}

#[pymethods]
impl Session {
    #[new]
    fn new() -> PyResult<Self> {
        let db = match RTreeDatabase::new(None, None) {
            Ok(o) => o,
            Err(e) => Err(Into::<PyErr>::into(ApiError::from(e)))?,
        };
        Ok(Session { db })
    }
    fn add_designation(&mut self, name: &str, spec: &str) -> PyResult<()> {
        match self.db.insert_spec_text(name, spec) {
            Ok(()) => Ok(()),
            Err(e) => Err(Into::<PyErr>::into(ApiError::from(e)))?,
        }
    }
    fn insert_metadata(
        &mut self,
        designation: &str,
        bb: &BoundingBox,
        buffer: &[u8],
    ) -> PyResult<()> {
        let m = Metadata {
            xmin: bb.a.x,
            xmax: bb.b.x,
            ymin: bb.a.y,
            ymax: bb.b.y,
            zmin: bb.a.z,
            zmax: bb.b.z,
            tmin: bb.a.t,
            tmax: bb.b.t,
            designation,
            buffer,
        };
        match self.db.insert_metadata(&m) {
            Ok(()) => Ok(()),
            Err(e) => Err(Into::<PyErr>::into(ApiError::from(e)))?,
        }
    }
    fn get_metadata<'py>(
        &self,
        py: Python<'py>,
        designation: &str,
        bb: &BoundingBox,
        eps: Option<f64>,
    ) -> PyResult<Vec<Bound<'py, PyDict>>> {
        let r = self.db.get_metadata_in_bb(
            bb.a.x,
            bb.b.x,
            bb.a.y,
            bb.b.y,
            bb.a.z,
            bb.b.z,
            bb.a.t,
            bb.b.t,
            designation,
            eps,
        );
        match &r {
            Ok(o) => Ok(o.iter().map(|x| value2obj(py, x).unwrap()).collect()),
            Err(e) => Err(Into::<PyErr>::into(ApiError::from(e)))?,
        }
    }
    fn print(&self) {
        println!("{self:#?}");
    }
}

/// A Python module implemented in Rust.
#[pymodule]
fn pyelucidator(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<Session>()?;
    m.add_class::<Point>()?;
    m.add_class::<BoundingBox>()?;
    Ok(())
}
