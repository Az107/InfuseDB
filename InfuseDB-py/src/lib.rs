use infusedb::InfuseDB;
use pyo3::{
    exceptions::{PyFileNotFoundError, PyValueError},
    prelude::*,
    types::{PyDict, PyFloat, PyList, PyString},
    IntoPyObjectExt,
};
use std::sync::{Arc, Mutex};

/// Wrapper in Rust to expose InfuseDB for Python
#[pyclass(name = "InfuseDB")]
struct InfuseDBpy {
    inner: Arc<Mutex<InfuseDB>>,
}

#[pyclass]
struct Collection {
    inner: Arc<Mutex<InfuseDB>>,
    name: String,
}

#[pymethods]
impl InfuseDBpy {
    #[new]
    fn new() -> Self {
        InfuseDBpy {
            inner: Arc::new(Mutex::new(InfuseDB::new())),
        }
    }

    #[staticmethod]
    fn load(path: &str) -> PyResult<Self> {
        let db = InfuseDB::load(path)
            .map_err(|_| PyFileNotFoundError::new_err("Path can't be loaded"))?;
        Ok(InfuseDBpy {
            inner: Arc::new(Mutex::new(db)),
        })
    }

    fn get_version(&self) -> &str {
        infusedb::VERSION
    }

    fn create_collection(&mut self, name: &str) -> PyResult<()> {
        let mut db = self.inner.lock().unwrap();
        let r = db.create_collection(name);
        if r.is_ok() {
            return Ok(());
        } else {
            return Err(PyValueError::new_err("Collection already exists"));
        }
    }

    fn list_collections(&self) -> Vec<String> {
        let db = self.inner.lock().unwrap();
        db.get_collection_list()
    }

    fn get_collection(&mut self, name: &str) -> PyResult<Collection> {
        let mut db = self.inner.lock().unwrap();
        let c = db.get_collection(name);
        if c.is_none() {
            return Err(PyValueError::new_err("error getting collection"));
        }
        let col = Collection {
            inner: Arc::clone(&self.inner),
            name: name.to_string(),
        };
        Ok(col)
    }
}

fn convert_data_type(py: Python<'_>, value: &infusedb::DataType) -> PyResult<PyObject> {
    match value {
        infusedb::DataType::Id(uuid) => Ok(PyString::new(py, &uuid.to_string()).into()),
        infusedb::DataType::Text(text) => Ok(PyString::new(py, text).into()),
        infusedb::DataType::Number(n) => Ok(PyFloat::new(py, *n as f64).into()),
        infusedb::DataType::Boolean(b) => {
            let b = if *b { 1.0 } else { 0.0 };
            return Ok(PyFloat::new(py, b).into());
        }
        infusedb::DataType::Array(data_types) => {
            let py_list = PyList::empty(py);
            for item in data_types {
                py_list.append(convert_data_type(py, item)?)?;
            }
            Ok(py_list.into())
        }
        infusedb::DataType::Document(hash_map) => {
            let py_dict = PyDict::new(py);
            for (key, value) in hash_map {
                py_dict.set_item(key, convert_data_type(py, value)?)?;
            }
            Ok(py_dict.into())
        }
    }
}

fn convert_py_to_data_type(py: Python<'_>, value: &PyObject) -> PyResult<infusedb::DataType> {
    if let Ok(s) = value.extract::<String>(py) {
        return Ok(infusedb::DataType::Text(s));
    }
    if let Ok(f) = value.extract::<f32>(py) {
        return Ok(infusedb::DataType::Number(f));
    }
    if let Ok(b) = value.extract::<bool>(py) {
        return Ok(infusedb::DataType::Boolean(b));
    }

    if let Ok(list) = value.extract::<Py<PyList>>(py) {
        let mut items = Vec::new();
        let list = list.bind_borrowed(py);
        for item in list.iter() {
            items.push(convert_py_to_data_type(py, &item.into_py_any(py)?)?);
        }
        return Ok(infusedb::DataType::Array(items));
    }
    if let Ok(dict) = value.extract::<Py<PyDict>>(py) {
        let mut map = std::collections::HashMap::new();
        let dict = dict.bind_borrowed(py);
        for (key, val) in dict.iter() {
            let key: String = key.extract()?;
            let val = convert_py_to_data_type(py, &val.into_py_any(py)?)?;
            map.insert(key, val);
        }
        return Ok(infusedb::DataType::Document(map));
    }

    Err(PyValueError::new_err("Unsupported data type"))
}

#[pymethods]
impl Collection {
    fn get(&mut self, py: Python<'_>, k: &str) -> PyResult<PyObject> {
        let mut db = self.inner.lock().unwrap();
        let c = db.get_collection(self.name.as_str());
        if c.is_none() {
            return Err(PyValueError::new_err("error getting collection"));
        }
        let c = c.unwrap();
        let r = c.get(k);
        if r.is_none() {
            return Err(PyValueError::new_err("error getting key"));
        }
        let r = r.unwrap();
        let value = convert_data_type(py, r);
        return Ok(value.unwrap());
    }

    fn set(&mut self, py: Python<'_>, k: &str, v: PyObject) -> PyResult<()> {
        let mut db = self.inner.lock().unwrap();
        let c = db.get_collection(self.name.as_str());
        if c.is_none() {
            return Err(PyValueError::new_err("error getting collection"));
        }
        let c = c.unwrap();
        let v = convert_py_to_data_type(py, &v)?;
        c.add(k, v);
        return Ok(());
    }

    fn del(&mut self, _py: Python<'_>, key: &str) -> PyResult<()> {
        let mut db = self.inner.lock().unwrap();
        let c = db.get_collection(self.name.as_str());
        if c.is_none() {
            return Err(PyValueError::new_err("error getting collection"));
        }
        let c = c.unwrap();
        c.rm(key);
        Ok(())
    }

    fn list(&self, py: Python<'_>) -> PyResult<Py<PyList>> {
        let list = PyList::empty(py);
        let mut db = self.inner.lock().unwrap();
        let c = db.get_collection(self.name.as_str());
        if c.is_none() {
            return Err(PyValueError::new_err("error getting collection"));
        }
        let c = c.unwrap();
        for (k, _) in c.list() {
            list.add(k)?;
        }
        return Ok(list.unbind());
    }
}

#[pymodule]
fn infusedb_py(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<InfuseDBpy>()?;
    Ok(())
}
