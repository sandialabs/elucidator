use std::collections::HashMap;

use rusqlite::Connection;

use elucidator::designation::DesignationSpecification;
use crate::database::{Database, Metadata, Datum, Result, DatabaseConfig};

struct SqlDatabase {
    /// Active database connection
    conn: Connection,
    /// Mapping of designations
    designations: HashMap<String, DesignationSpecification>,
    /// Extra configuration settings for the database
    config: Option<SqliteConfig>,
}

pub struct SqliteConfig {}

impl Database for SqlDatabase {
    fn new(filename: Option<&str>, config: Option<&DatabaseConfig>) -> Result<Self> {
        let db = if let Some(name) = filename {
            SqlDatabase {
                conn: Connection::open(name)?,
                designations: HashMap::new(),
                config: None,
            }
        } else {
            SqlDatabase {
                conn: Connection::open_in_memory()?,
                designations: HashMap::new(),
                config: None,
            }
        };
        db.initialize()?;
        Ok(db)
    }
    fn from_path(filename: &str) -> Result<Self> {
        let conn = Connection::open(filename)?;
        let mut designations = HashMap::new();
        {
            let mut stmt = conn.prepare_cached(
                "SELECT designation, spec FROM designation_spec;"
            )?;
            let mut rows = stmt.query([])?;
            while let Some(row) = rows.next()? {
                let designation: String = row.get(0)?;
                let spec_text: String = row.get(1)?;
                let spec = DesignationSpecification::from_text(&spec_text).unwrap();
                designations.insert(designation, spec);
            }
        }
        Ok(SqlDatabase { 
            conn,
            designations,
            config: None,
        })
    }
    fn save_as(&self, filename: &str) -> Result<()> {
        self.conn.backup(rusqlite::DatabaseName::Main, filename, None)?;
        Ok(())
    }
    fn initialize(&self) -> Result<()> {
        self.conn.execute(
            "CREATE TABLE designation_spec (
                designation  TEXT NOT NULL PRIMARY KEY,
                spec  TEXT NOT NULL
            )",
            (), // empty list of parameters.
        )?;
        self.conn.execute(
            "CREATE TABLE Metadata (
                xmin REAL,
                xmax REAL,
                ymin REAL,
                ymax REAL,
                zmin REAL,
                zmax REAL,
                tmin REAL,
                tmax REAL,
                designation  TEXT NOT NULL,
                buffer  BLOB
            )",
            (), // empty list of parameters.
        )?;
        Ok(())
    }
    fn insert_spec_text(&mut self, designation: &str, spec: &str) -> Result<()> {
        let designation_spec = DesignationSpecification::from_text(spec)?;
        self.conn.execute(
            "INSERT INTO designation_spec (designation, spec) VALUES (?1, ?2)",
            (designation, spec),
        )?;
        self.designations.insert(designation.to_string(), designation_spec);
        Ok(())
    }
    fn insert_metadata(&self, datum: &Metadata) -> Result<usize> {
        let result = self.conn.execute(
            "INSERT INTO Metadata (xmin, xmax, ymin, ymax, zmin, zmax, tmin, tmax, designation, buffer) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            (
                &datum.xmin,
                &datum.xmax,
                &datum.ymin,
                &datum.ymax,
                &datum.zmin,
                &datum.zmax,
                &datum.tmin,
                &datum.tmax, 
                &datum.designation,
                &datum.buffer, 
            ),
        )?;
        Ok(result)
    }
    fn insert_n_metadata(&self, data: &Vec<Metadata>) -> Result<usize> {
        const N_FIELDS: usize = 10;
        let unbound = (0..data.len()).map(|idx| {
            let unbound_value = (idx*N_FIELDS + 1..=(idx+1)*N_FIELDS)
                .map(|i| format!("?{i}"))
                .collect::<Vec<String>>()
                .join(", ");
            format!("({unbound_value})")
        })
        .collect::<Vec<String>>()
        .join(", ");

        let sql_statement = format!(
            "INSERT INTO Metadata (xmin, xmax, ymin, ymax, zmin, zmax, tmin, tmax, designation, buffer) VALUES {unbound};"
        );

        let mut stmt = self.conn.prepare_cached(&sql_statement)?;
        for (i, m) in data.iter().enumerate() {
            stmt.raw_bind_parameter(1 + i*N_FIELDS, &m.xmin)?;
            stmt.raw_bind_parameter(2 + i*N_FIELDS, &m.xmax)?;
            stmt.raw_bind_parameter(3 + i*N_FIELDS, &m.ymin)?;
            stmt.raw_bind_parameter(4 + i*N_FIELDS, &m.ymax)?;
            stmt.raw_bind_parameter(5 + i*N_FIELDS, &m.zmin)?;
            stmt.raw_bind_parameter(6 + i*N_FIELDS, &m.zmax)?;
            stmt.raw_bind_parameter(7 + i*N_FIELDS, &m.tmin)?;
            stmt.raw_bind_parameter(8 + i*N_FIELDS, &m.tmax)?;
            stmt.raw_bind_parameter(9 + i*N_FIELDS, &m.designation)?;
            stmt.raw_bind_parameter(10 + i*N_FIELDS, &m.buffer)?;
        }
        let result = stmt.raw_execute()?;
        Ok(result)
    }
    fn get_metadata_in_bb(
        &self,
        xmin: f64, xmax: f64,
        ymin: f64, ymax: f64,
        zmin: f64, zmax: f64,
        tmin: f64, tmax: f64,
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
        let mut stmt = self.conn.prepare_cached(
            "SELECT buffer
            FROM Metadata
            WHERE
                xmin >= ?1 AND
                xmax <= ?2 AND
                ymin >= ?3 AND
                ymax <= ?4 AND
                zmin >= ?5 AND
                zmax <= ?6 AND
                tmin >= ?7 AND
                tmax <= ?8 AND
                designation = ?9
            ;
            "
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
            let buffer = match row.get_ref(0)? {
                rusqlite::types::ValueRef::Blob(b) => b,
                _ => unreachable!("We should always retrieve blobs!"),
            };
            let d = self.designations.get(designation).unwrap();
            data.push(d.interpret_enum(&buffer).unwrap());
        }
        Ok(data)
    }
}


#[cfg(test)]
mod test {
    use super::*;
    use rand::Rng;
    use rand;

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
                filepath: filepath.to_string()
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
    
    mod database {
        use super::*;
        use std::{collections::HashSet, ops::Deref};
        use elucidator::value::DataValue;
        use crate::error::DatabaseError;

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
            let keys = db.designations.keys()
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

            pretty_assertions::assert_eq!(result, Ok(1 as usize));
        }

        #[test]
        fn insert_bad_designation_fails() {
            let mut db = SqlDatabase::new(None, None).unwrap();
            let designation = "Foo";
            let spec = "foo u8";
            let result = db.insert_spec_text(designation, spec);
            let expected = DesignationSpecification::from_text(spec);
            assert!(expected.is_err(), "Expected an error from bad designation spec, but got ok instead.");
            pretty_assertions::assert_eq!(
                result,
                Err(DatabaseError::ElucidatorError { reason: expected.unwrap_err() })
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

            pretty_assertions::assert_eq!(result, Ok(3 as usize));
        }

        
        #[test]
        fn bb_search_ok() {
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

            let buffer: &[u8; 1] = &[200; 1];
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
            
            let result = db.get_metadata_in_bb(
                0.0, 1.0,
                0.0, 1.0,
                0.0, 1.0,
                0.0, 1.0,
                "Foo", 
                None,
            );

            let expected: Vec<HashMap<&str, DataValue>> = vec![
                HashMap::from(
                    [("foo", DataValue::Byte(100))]
                ),
                HashMap::from(
                    [("foo", DataValue::Byte(150))]
                ),
            ];
            pretty_assertions::assert_eq!(
                result, 
                Ok(expected),
            );
        }
    }
}