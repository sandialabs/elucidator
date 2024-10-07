use std::{collections::HashMap, io::Read};

use rusqlite::Connection;

use crate::{
    backends::rtree::MetadataClone,
    database::{Config, Database, DatabaseConfig, Datum, Metadata, Result},
    error::DatabaseError,
};
use elucidator::designation::DesignationSpecification;

use serde::{Deserialize, Serialize};
use serde_json;

use std::fs::File;
use std::io::Write;
use std::sync::{Arc, Mutex};

pub struct SqlDatabase {
    /// Active database connection
    conn: Arc<Mutex<Connection>>,
    /// Mapping of designations
    designations: HashMap<String, DesignationSpecification>,
    /// Extra configuration settings for the database
    config: SqliteConfig,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct SqliteConfig {
    use_rtree: bool,
    use_wal: bool,
    page_size: u16,
    synchronous_off: bool,
    use_memory_temp_store: bool,
    threads: u32,
    cached_pages: u32,
}

impl Config for SqliteConfig {
    fn new() -> Self {
        SqliteConfig {
            use_rtree: true,
            use_wal: false,
            page_size: 4096,
            synchronous_off: false,
            use_memory_temp_store: false,
            threads: 0,
            cached_pages: 0,
        }
    }
    fn from_json_file(filename: &str) -> Result<Self> {
        let mut file = File::open(filename)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        Ok(serde_json::from_str(&contents).unwrap())
    }
    fn to_json_file(&self, filename: &str) -> Result<()> {
        let mut file = File::create(filename)?;
        let json = serde_json::to_string(&self).unwrap();
        write!(file, "{json}")?;
        Ok(())
    }
}

impl SqliteConfig {
    pub fn use_wal(&mut self) -> Self {
        self.use_wal = true;
        self.clone()
    }
    pub fn page_size(&mut self, page_size: u16) -> Self {
        self.page_size = page_size;
        self.clone()
    }
    pub fn no_sync(&mut self) -> Self {
        self.synchronous_off = true;
        self.clone()
    }
}

impl SqlDatabase {
    const MIN_VERSION: [u32; 3] = [3, 7, 0];
    fn initialize(&self) -> Result<()> {
        self.verify_version()?;
        let conn = self.conn.lock()?;
        if self.config.use_wal {
            conn.execute("PRAGMA journal_mode = WAL", [])?;
        }
        conn.execute(&format!("PRAGMA page_size = {}", self.config.page_size), [])?;
        if self.config.synchronous_off {
            conn.execute("PRAGMA synchronous = OFF", [])?;
        }
        if self.config.use_memory_temp_store {
            conn.execute("PRAGMA temp_store = MEMORY", [])?;
        }
        if self.config.threads > 0 {
            conn.execute(&format!("PRAGMA threads = {}", self.config.threads), [])?;
        }
        if self.config.cached_pages > 0 {
            conn.execute(
                &format!("PRAGMA cache_size = {}", self.config.cached_pages),
                [],
            )?;
        }
        conn.execute(
            "CREATE TABLE designation_spec (
                designation  TEXT NOT NULL PRIMARY KEY,
                spec  TEXT NOT NULL
            )",
            (), // empty list of parameters.
        )?;
        conn.execute(
            "CREATE VIRTUAL TABLE MetadataLocations USING rtree(
                id INTEGER PRIMARY KEY,
                xmin, xmax, ymin, ymax, zmin, zmax, tmin, tmax
            )",
            (), // empty list of parameters.
        )?;
        conn.execute(
            "CREATE TABLE Metadata (
                id INTEGER PRIMARY KEY,
                designation TEXT,
                buffer BLOB
            )",
            [],
        )?;
        conn.execute("PRAGMA optimize", [])?;
        Ok(())
    }
    fn verify_version(&self) -> Result<(), DatabaseError> {
        let conn = self.conn.lock()?;
        let version = conn
            .query_row("SELECT SQLITE_VERSION();", [], |row| {
                row.get::<usize, String>(0)
            })?
            .split(".")
            .map(|n| n.parse::<u32>().unwrap_or(0))
            .collect::<Vec<u32>>();

        for (curr_v, min_v) in version.iter().zip(SqlDatabase::MIN_VERSION.iter()) {
            if curr_v > min_v {
                break;
            }
            if curr_v < min_v {
                let version_error = DatabaseError::VersionError {
                    reason: format!(
                        "Expected sqlite version of at least {:?}, found {:?}",
                        SqlDatabase::MIN_VERSION,
                        version,
                    ),
                };
                Err(version_error)?;
            }
        }
        Ok(())
    }
    pub fn get_designations(&self) -> HashMap<String, DesignationSpecification> {
        self.designations.clone()
    }
    pub fn get_all_metadata<'a>(&self) -> Result<Vec<MetadataClone>> {
        let mut data = Vec::new();
        let conn = self.conn.lock()?;
        let mut stmt = conn.prepare_cached(
            "SELECT 
                ml.id, ml.xmin, ml.xmax, ml.ymin, ml.ymax, ml.zmin, ml.zmax, ml.tmin, ml.tmax,
                m.designation, m.buffer
            FROM 
                Metadata AS m
            JOIN 
                MetadataLocations AS ml
            ON 
                ml.id = m.id
            ",
        )?;
        let mut rows = stmt.raw_query();
        while let Some(row) = rows.next()? {
            let xmin = row.get_ref(1)?.as_f64()?;
            let xmax = row.get_ref(2)?.as_f64()?;
            let ymin = row.get_ref(3)?.as_f64()?;
            let ymax = row.get_ref(4)?.as_f64()?;
            let zmin = row.get_ref(5)?.as_f64()?;
            let zmax = row.get_ref(6)?.as_f64()?;
            let tmin = row.get_ref(7)?.as_f64()?;
            let tmax = row.get_ref(8)?.as_f64()?;
            let designation = row.get_ref(9)?.as_str()?;
            let buffer = match row.get_ref(10)? {
                rusqlite::types::ValueRef::Blob(b) => b,
                _ => unreachable!("We should always retrieve blobs!"),
            };
            data.push(MetadataClone {
                xmin,
                xmax,
                ymin,
                ymax,
                zmin,
                zmax,
                tmin,
                tmax,
                designation: designation.to_string(),
                buffer: buffer.into(),
            });
        }
        Ok(data)
    }
}

impl Database for SqlDatabase {
    fn new(filename: Option<&str>, config: Option<&DatabaseConfig>) -> Result<Self> {
        let config = match config {
            Some(dbcfg) => match &dbcfg {
                DatabaseConfig::SqliteConfig(sqlcfg) => sqlcfg.clone(),
                _ => Err(DatabaseError::ConfigError {
                    reason: "Sqlite given config for incorrect backend.".to_string(),
                })?,
            },
            None => SqliteConfig::new(),
        };
        let db = if let Some(name) = filename {
            SqlDatabase {
                conn: Arc::new(Mutex::new(Connection::open(name)?)),
                designations: HashMap::new(),
                config,
            }
        } else {
            SqlDatabase {
                conn: Arc::new(Mutex::new(Connection::open_in_memory()?)),
                designations: HashMap::new(),
                config,
            }
        };
        db.initialize()?;
        Ok(db)
    }
    fn from_path(filename: &str) -> Result<Self> {
        let conn = Connection::open(filename)?;
        let mut designations = HashMap::new();
        {
            let mut stmt =
                conn.prepare_cached("SELECT designation, spec FROM designation_spec;")?;
            let mut rows = stmt.query([])?;
            while let Some(row) = rows.next()? {
                let designation: String = row.get(0)?;
                let spec_text: String = row.get(1)?;
                let spec = DesignationSpecification::from_text(&spec_text).unwrap();
                designations.insert(designation, spec);
            }
        }
        Ok(SqlDatabase {
            conn: Arc::new(Mutex::new(conn)),
            designations,
            config: SqliteConfig::new(),
        })
    }
    fn save_as(&self, filename: &str) -> Result<()> {
        let conn = self.conn.lock()?;
        conn.backup(rusqlite::DatabaseName::Main, filename, None)?;
        Ok(())
    }

    fn insert_spec_text(&mut self, designation: &str, spec: &str) -> Result<()> {
        let designation_spec = DesignationSpecification::from_text(spec)?;
        let conn = self.conn.lock()?;
        conn.execute(
            "INSERT INTO designation_spec (designation, spec) VALUES (?1, ?2)",
            (designation, spec),
        )?;
        self.designations
            .insert(designation.to_string(), designation_spec);
        Ok(())
    }
    fn insert_metadata(&mut self, datum: &Metadata) -> Result<()> {
        let mut conn = self.conn.lock()?;
        let tx = conn.transaction()?;
        {
            let mut stmt = tx.prepare_cached(
                "INSERT INTO MetadataLocations (xmin, xmax, ymin, ymax, zmin, zmax, tmin, tmax) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            )?;
            stmt.execute([
                datum.xmin, datum.xmax, datum.ymin, datum.ymax, datum.zmin, datum.zmax, datum.tmin,
                datum.tmax,
            ])?;
            let mut stmt = tx.prepare_cached(
                "INSERT INTO Metadata (id, designation, buffer) VALUES (last_insert_rowid(), ?1, ?2)",
            )?;
            stmt.raw_bind_parameter(1, datum.designation)?;
            stmt.raw_bind_parameter(2, datum.buffer)?;
            stmt.raw_execute()?;
        }

        tx.commit()?;

        Ok(())
    }
    fn insert_n_metadata(&mut self, data: &Vec<Metadata>) -> Result<()> {
        let mut conn = self.conn.lock()?;
        let tx = conn.transaction()?;

        for datum in data {
            let mut stmt = tx.prepare_cached(
                "INSERT INTO MetadataLocations (xmin, xmax, ymin, ymax, zmin, zmax, tmin, tmax) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            )?;
            stmt.execute([
                datum.xmin, datum.xmax, datum.ymin, datum.ymax, datum.zmin, datum.zmax, datum.tmin,
                datum.tmax,
            ])?;
            let mut stmt = tx.prepare_cached(
                "INSERT INTO Metadata (id, designation, buffer) VALUES (last_insert_rowid(), ?1, ?2)",
            )?;
            stmt.raw_bind_parameter(1, datum.designation)?;
            stmt.raw_bind_parameter(2, datum.buffer)?;
            stmt.raw_execute()?;
        }

        tx.commit()?;

        Ok(())
    }
    fn get_metadata_in_bb(
        &self,
        xmin: f64,
        xmax: f64,
        ymin: f64,
        ymax: f64,
        zmin: f64,
        zmax: f64,
        tmin: f64,
        tmax: f64,
        designation: &str,
        epsilon: Option<f64>,
    ) -> Result<Vec<Datum>> {
        let eps = epsilon.unwrap_or(0.0);
        let xmin = xmin - eps;
        let xmax = xmax + eps;
        let ymin = ymin - eps;
        let ymax = ymax + eps;
        let zmin = zmin - eps;
        let zmax = zmax + eps;
        let tmin = tmin - eps;
        let tmax = tmax + eps;

        let conn = self.conn.lock()?;
        let mut stmt = conn.prepare_cached(
            "SELECT 
                ml.id, ml.xmin, ml.xmax, ml.ymin, ml.ymax, ml.zmin, ml.zmax, ml.tmin, ml.tmax,
                m.designation, m.buffer
            FROM 
                Metadata AS m
            JOIN 
                MetadataLocations AS ml
            ON 
                ml.id = m.id
            WHERE 
                ml.xmin >= ?1 AND ml.xmax <= ?2 AND
                ml.ymin >= ?3 AND ml.ymax <= ?4 AND
                ml.zmin >= ?5 AND ml.zmax <= ?6 AND
                ml.tmin >= ?7 AND ml.tmax <= ?8 AND
                m.designation = ?9
            ",
        )?;

        stmt.raw_bind_parameter(1, xmin)?;
        stmt.raw_bind_parameter(2, xmax)?;
        stmt.raw_bind_parameter(3, ymin)?;
        stmt.raw_bind_parameter(4, ymax)?;
        stmt.raw_bind_parameter(5, zmin)?;
        stmt.raw_bind_parameter(6, zmax)?;
        stmt.raw_bind_parameter(7, tmin)?;
        stmt.raw_bind_parameter(8, tmax)?;
        stmt.raw_bind_parameter(9, designation)?;

        let mut rows = stmt.raw_query();
        let mut data = Vec::new();
        while let Some(row) = rows.next()? {
            let buffer = match row.get_ref(10)? {
                rusqlite::types::ValueRef::Blob(b) => b,
                _ => unreachable!("We should always retrieve blobs!"),
            };
            let d = self.designations.get(designation).unwrap();
            data.push(d.interpret_enum(buffer).unwrap());
        }
        Ok(data)
    }

    fn get_metadata_blobs_in_bb(
        &self,
        _xmin: f64,
        _xmax: f64,
        _ymin: f64,
        _ymax: f64,
        _zmin: f64,
        _zmax: f64,
        _tmin: f64,
        _tmax: f64,
        _designation: &str,
        _epsilon: Option<f64>,
    ) -> Result<Vec<&Vec<u8>>> {
        todo!();
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use rand;
    use rand::Rng;

    struct TempFile {
        pub filepath: String,
    }

    impl TempFile {
        pub fn new() -> Result<Self> {
            let random_filename = random_identifier(10);
            Self::from(&random_filename)
        }
        pub fn from(filename: &str) -> Result<Self> {
            let temp_dir = std::env::temp_dir();
            let file_dir = temp_dir.join(random_identifier(10));
            let filepath = file_dir.join(filename);
            let filepath = filepath.to_str().unwrap();
            let _ = std::fs::create_dir_all(file_dir);
            Ok(TempFile {
                filepath: filepath.to_string(),
            })
        }
    }

    impl std::ops::Drop for TempFile {
        fn drop(&mut self) {
            let _ = std::fs::remove_file(&self.filepath);
        }
    }

    fn random_identifier(size: u8) -> String {
        let mut rng = rand::thread_rng();
        (0..size)
            .map(|_| (rng.gen_range(b'a'..=b'z') as char))
            .collect()
    }

    mod config {
        use super::*;
        

        #[test]
        fn to_and_from_json_ok() {
            let cfg = SqliteConfig::new().use_wal().page_size(2048);
            let temp_file = TempFile::from("temp.json").unwrap();
            let _ = cfg.to_json_file(&temp_file.filepath);
            let recovered_cfg = SqliteConfig::from_json_file(&temp_file.filepath).unwrap();
            pretty_assertions::assert_eq!(cfg, recovered_cfg);
        }
    }

    mod database {
        use super::*;
        use crate::error::DatabaseError;
        use elucidator::value::DataValue;
        use std::{collections::HashSet, ops::Deref};

        #[test]
        fn create_in_memory_ok() {
            assert!(SqlDatabase::new(None, None).is_ok());
        }

        #[test]
        fn create_with_filepath_ok() {
            let tempfile = TempFile::from("temp.db").unwrap();
            let db = SqlDatabase::new(Some(&tempfile.filepath), None);
            assert!(db.is_ok());
        }

        #[test]
        fn from_empty_ok() {
            let tempfile = TempFile::from("temp.db").unwrap();
            let db = SqlDatabase::new(Some(&tempfile.filepath), None);
            drop(db);
            let loaded_db = SqlDatabase::from_path(&tempfile.filepath);
            assert!(loaded_db.is_ok());
        }

        #[test]
        fn insert_designation_ok() {
            let mut db = SqlDatabase::new(None, None).unwrap();
            let designation = "Foo";
            let spec = "foo: u8";
            let result = db.insert_spec_text(designation, spec);
            pretty_assertions::assert_eq!(result, Ok(()));
            let keys = db
                .designations
                .keys()
                .map(String::deref)
                .collect::<HashSet<&str>>();
            pretty_assertions::assert_eq!(keys, HashSet::from(["Foo"]));
        }

        #[test]
        fn insert_1_ok() {
            let mut db = SqlDatabase::new(None, None).unwrap();

            let buffer: &[u8; 1] = &[100; 1];
            let designation = "Foo";
            let spec = "foo: u8";
            let md = Metadata {
                xmin: 0.0,
                xmax: 0.0,
                ymin: 0.0,
                ymax: 0.0,
                zmin: 0.0,
                zmax: 0.0,
                tmin: 0.0,
                tmax: 0.0,
                designation,
                buffer,
            };

            let _ = db.insert_spec_text(designation, spec);
            let result = db.insert_metadata(&md);

            pretty_assertions::assert_eq!(result, Ok(()));
        }

        #[test]
        fn insert_bad_designation_fails() {
            let mut db = SqlDatabase::new(None, None).unwrap();
            let designation = "Foo";
            let spec = "foo u8";
            let result = db.insert_spec_text(designation, spec);
            let expected = DesignationSpecification::from_text(spec);
            assert!(
                expected.is_err(),
                "Expected an error from bad designation spec, but got ok instead."
            );
            pretty_assertions::assert_eq!(
                result,
                Err(DatabaseError::ElucidatorError {
                    reason: expected.unwrap_err()
                })
            );
        }

        #[test]
        fn insert_n_ok() {
            let mut db = SqlDatabase::new(None, None).unwrap();

            let designation = "Foo";
            let spec = "foo: u8";
            let buffer: &[u8; 1] = &[100; 1];
            let md1 = Metadata {
                xmin: 0.0,
                xmax: 0.0,
                ymin: 0.0,
                ymax: 0.0,
                zmin: 0.0,
                zmax: 0.0,
                tmin: 0.0,
                tmax: 0.0,
                designation,
                buffer,
            };

            let buffer: &[u8; 1] = &[150; 1];
            let md2 = Metadata {
                xmin: 0.0,
                xmax: 0.0,
                ymin: 0.0,
                ymax: 0.0,
                zmin: 0.0,
                zmax: 0.0,
                tmin: 0.0,
                tmax: 0.0,
                designation,
                buffer,
            };

            let buffer: &[u8; 1] = &[200; 1];
            let md3 = Metadata {
                xmin: 0.0,
                xmax: 0.0,
                ymin: 0.0,
                ymax: 0.0,
                zmin: 0.0,
                zmax: 0.0,
                tmin: 0.0,
                tmax: 0.0,
                designation,
                buffer,
            };

            let metadata: Vec<Metadata> = vec![md1, md2, md3];

            let _ = db.insert_spec_text(designation, spec);
            let result = db.insert_n_metadata(&metadata);

            pretty_assertions::assert_eq!(result, Ok(()));
        }

        #[test]
        fn bb_search_ok() {
            let mut db = SqlDatabase::new(None, None).unwrap();

            let designation = "Foo";
            let spec = "foo: u8, bar: f32";
            let buffer: &[u8; 5] = &[100, 0, 0, 128, 63];
            let md1 = Metadata {
                xmin: 0.0,
                xmax: 0.0,
                ymin: 0.0,
                ymax: 0.0,
                zmin: 0.0,
                zmax: 0.0,
                tmin: 0.0,
                tmax: 0.0,
                designation,
                buffer,
            };

            let buffer: &[u8; 5] = &[150, 0, 36, 116, 73];
            let md2 = Metadata {
                xmin: 0.0,
                xmax: 1.0,
                ymin: 0.0,
                ymax: 1.0,
                zmin: 0.0,
                zmax: 1.0,
                tmin: 0.0,
                tmax: 1.0,
                designation,
                buffer,
            };

            let buffer: &[u8; 5] = &[200, 0, 0, 200, 194];
            let md3 = Metadata {
                xmin: 0.0,
                xmax: 2.0,
                ymin: 0.0,
                ymax: 2.0,
                zmin: 0.0,
                zmax: 2.0,
                tmin: 0.0,
                tmax: 2.0,
                designation,
                buffer,
            };

            let metadata: Vec<Metadata> = vec![md1, md2, md3];

            let _ = db.insert_spec_text(designation, spec);
            let _ = db.insert_n_metadata(&metadata);

            let result = db.get_metadata_in_bb(0.0, 1.0, 0.0, 1.0, 0.0, 1.0, 0.0, 1.0, "Foo", None);

            let expected: Vec<HashMap<&str, DataValue>> = vec![
                HashMap::from([
                    ("foo", DataValue::Byte(100)),
                    ("bar", DataValue::Float32(1.0)),
                ]),
                HashMap::from([
                    ("foo", DataValue::Byte(150)),
                    ("bar", DataValue::Float32(1000000.0)),
                ]),
            ];
            pretty_assertions::assert_eq!(result, Ok(expected),);
        }
    }
}
