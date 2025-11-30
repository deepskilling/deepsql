/// Python bindings for DeepSQL
/// 
/// Provides a Python-friendly API for all DeepSQL features

#[cfg(feature = "python")]
use pyo3::prelude::*;
#[cfg(feature = "python")]
use pyo3::exceptions::PyException;

#[cfg(feature = "python")]
use crate::engine::Engine;
#[cfg(feature = "python")]
use crate::error::Error as RustError;
#[cfg(feature = "python")]
use crate::types::Value;
#[cfg(feature = "python")]
use crate::storage::record::Record;
#[cfg(feature = "python")]
use crate::planner::plan_cache::PlanCache;
#[cfg(feature = "python")]
use crate::planner::statistics::StatisticsManager;

/// Convert Rust Error to Python exception
#[cfg(feature = "python")]
fn to_pyerr(err: RustError) -> PyErr {
    PyException::new_err(err.to_string())
}

/// Python Database class
#[cfg(feature = "python")]
#[pyclass]
pub struct Database {
    engine: Engine,
    plan_cache: PlanCache,
    stats_manager: StatisticsManager,
}

#[cfg(feature = "python")]
#[pymethods]
impl Database {
    /// Open or create a database
    #[new]
    fn new(path: String) -> PyResult<Self> {
        let engine = Engine::open(&path).map_err(to_pyerr)?;
        Ok(Database {
            engine,
            plan_cache: PlanCache::new(),
            stats_manager: StatisticsManager::new(),
        })
    }
    
    /// Execute a SQL query and return number of affected rows
    fn execute_update(&mut self, _sql: String) -> PyResult<usize> {
        // Placeholder - full implementation would parse and execute SQL
        Ok(0)
    }
    
    /// Execute a query and return all rows
    fn query(&mut self, _sql: String) -> PyResult<Vec<Vec<PyValue>>> {
        // Placeholder - full implementation would parse and execute SQL
        Ok(vec![])
    }
    
    /// Begin a transaction
    fn begin_transaction(&mut self) -> PyResult<()> {
        self.engine.begin_transaction().map_err(to_pyerr)
    }
    
    /// Commit the current transaction
    fn commit(&mut self) -> PyResult<()> {
        self.engine.commit_transaction().map_err(to_pyerr)
    }
    
    /// Rollback the current transaction
    fn rollback(&mut self) -> PyResult<()> {
        self.engine.rollback_transaction().map_err(to_pyerr)
    }
    
    /// Insert a key-value pair
    fn insert(&mut self, key: Vec<u8>, value: Vec<u8>) -> PyResult<()> {
        let record = Record::new(key.clone(), vec![crate::storage::record::Value::Blob(value)]);
        self.engine.insert(record).map_err(to_pyerr)
    }
    
    /// Search for a key
    fn search(&mut self, key: Vec<u8>) -> PyResult<Option<Vec<u8>>> {
        match self.engine.search(&key) {
            Ok(record) => Ok(Some(record.key)),
            Err(RustError::NotFound) => Ok(None),
            Err(e) => Err(to_pyerr(e)),
        }
    }
    
    /// Delete a key
    fn delete(&mut self, key: Vec<u8>) -> PyResult<()> {
        self.engine.delete(&key).map_err(to_pyerr)
    }
    
    /// Bulk load records (sorted)
    fn bulk_load(&mut self, _records: Vec<(Vec<u8>, Vec<u8>)>) -> PyResult<usize> {
        // Placeholder - would use bulk_load function
        Ok(0)
    }
    
    /// Collect statistics for a table
    fn collect_statistics(&mut self, table: String) -> PyResult<()> {
        self.stats_manager
            .collect_stats_for_table(table, 0.1)
            .map_err(to_pyerr)
    }
    
    /// Get cache statistics
    fn get_cache_stats(&self) -> PyResult<CacheStats> {
        let stats = self.plan_cache.stats();
        Ok(CacheStats {
            size: stats.size,
            hits: stats.hits,
            misses: stats.misses,
            hit_rate: stats.hit_rate,
        })
    }
    
    /// Clear the plan cache
    fn clear_cache(&mut self) {
        self.plan_cache.invalidate_all();
    }
    
    /// Close the database
    fn close(&mut self) -> PyResult<()> {
        // Cleanup
        Ok(())
    }
    
    /// Context manager support - enter
    fn __enter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }
    
    /// Context manager support - exit
    #[pyo3(signature = (_exc_type=None, _exc_value=None, _traceback=None))]
    fn __exit__(
        &mut self,
        _exc_type: Option<Bound<'_, PyAny>>,
        _exc_value: Option<Bound<'_, PyAny>>,
        _traceback: Option<Bound<'_, PyAny>>,
    ) -> PyResult<bool> {
        self.close()?;
        Ok(false)
    }
    
    /// String representation
    fn __repr__(&self) -> String {
        "Database()".to_string()
    }
}

/// Python-friendly value type
#[cfg(feature = "python")]
#[pyclass]
#[derive(Clone)]
pub struct PyValue {
    inner: Value,
}

#[cfg(feature = "python")]
impl From<Value> for PyValue {
    fn from(v: Value) -> Self {
        PyValue { inner: v }
    }
}

#[cfg(feature = "python")]
#[pymethods]
impl PyValue {
    /// Check if value is NULL
    fn is_null(&self) -> bool {
        self.inner.is_null()
    }
    
    /// Convert to Python object
    fn to_python(&self, py: Python) -> PyResult<PyObject> {
        match &self.inner {
            Value::Null => Ok(py.None()),
            Value::Integer(i) => Ok(i.to_object(py)),
            Value::Real(f) => Ok(f.to_object(py)),
            Value::Text(s) => Ok(s.to_object(py)),
            Value::Blob(b) => Ok(b.to_object(py)),
        }
    }
    
    /// String representation
    fn __repr__(&self) -> String {
        self.inner.to_text()
    }
}

/// Cache statistics
#[cfg(feature = "python")]
#[pyclass]
#[derive(Clone)]
pub struct CacheStats {
    #[pyo3(get)]
    size: usize,
    
    #[pyo3(get)]
    hits: u64,
    
    #[pyo3(get)]
    misses: u64,
    
    #[pyo3(get)]
    hit_rate: f64,
}

#[cfg(feature = "python")]
#[pymethods]
impl CacheStats {
    fn __repr__(&self) -> String {
        format!(
            "CacheStats(size={}, hits={}, misses={}, hit_rate={:.2}%)",
            self.size, self.hits, self.misses, self.hit_rate * 100.0
        )
    }
}

/// Python module initialization
#[cfg(feature = "python")]
#[pymodule]
fn _deepsql(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<Database>()?;
    m.add_class::<PyValue>()?;
    m.add_class::<CacheStats>()?;
    
    m.add("__version__", "0.1.0")?;
    m.add("__doc__", "DeepSQL - A modern embedded SQL database in Rust")?;
    
    Ok(())
}

#[cfg(not(feature = "python"))]
pub fn init() {
    // Stub when Python feature is disabled
}
