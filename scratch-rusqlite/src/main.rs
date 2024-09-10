use std::{borrow::Borrow, collections::HashMap};
use elucidator::{designation::{self, DesignationSpecification}, value::DataValue};
use rusqlite::{Connection, Result};

type Datum<'a> = HashMap<&'a str, DataValue>;

#[derive(Debug)]
struct DesignationSpec {
    designation: String,
    spec: String,
}

#[derive(Debug, Clone)]
struct Metadata<'a> {
    pub xmin: f64,
    pub xmax: f64,
    pub ymin: f64,
    pub ymax: f64,
    pub zmin: f64,
    pub zmax: f64,
    pub tmin: f64,
    pub tmax: f64,
    pub designation: &'a str,
    pub buffer: &'a [u8],
}

struct Database {
    /// Active database connection
    conn: Connection,
    /// Mapping of designations
    designations: HashMap<String, DesignationSpecification>,
}

impl Database {
    fn new(filename: Option<&str>) -> Result<Self> {
        let db = if let Some(name) = filename {
            Database {
                conn: Connection::open(name)?,
                designations: HashMap::new(),
            }
        } else {
            Database {
                conn: Connection::open_in_memory()?,
                designations: HashMap::new(),
            }
        };
        db.initialize()?;
        Ok(db)
    }
    fn from(filename: &str) -> Result<Self> {
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
        Ok(Database { 
            conn,
            designations,
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

        self.conn.execute(
            "INSERT INTO designation_spec (designation, spec) VALUES (?1, ?2)",
            (designation, spec),
        )?;
        let spec = DesignationSpecification::from_text(spec).unwrap();
        self.designations.insert(designation.to_string(), spec);
        Ok(())
    }
    fn insert_metadata(&self, datum: &Metadata) -> Result<()> {
        self.conn.execute(
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
        Ok(())
    }
    fn insert_n_metadata(&self, data: &Vec<Metadata>) -> Result<()> {
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
        stmt.raw_execute()?;
        Ok(())
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

fn main() -> Result<()> {
    let mut db = Database::new(None)?;
    let designation = "FooBar";
    let spec = "foo: u32, bar: i8";
    let buffer = &[7, 0, 0, 0, unsafe { std::mem::transmute::<i8, u8>(-5) }];
    db.insert_spec_text(designation, spec)?;
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
    db.insert_metadata(&md)?;
    let data = db.get_metadata_in_bb(
        -1.0, 1.0,
        -1.0, 1.0,
        -1.0, 1.0,
        -1.0, 1.0,
        designation,
        None
    ).unwrap();
    println!("{data:#?}");

    let _ = db.save_as("my_database.db");
    let mydb = Database::from("my_database.db")?;

    let my_data = mydb.get_metadata_in_bb(
        -9.9, 9.9,
        -9.9, 9.9,
        -9.9, 9.9,
        -9.9, 9.9,
        designation,
       Some(1.0) 
    ).unwrap();
    println!("{my_data:#?}"); 

    println!("----  ----  ----  ----  ----");
    let buffer = &[123, 0, 0, 0, unsafe { std::mem::transmute::<i8, u8>(-123) }];
    let md1 = Metadata {
        xmin: 0.1,
        xmax: 0.1,
        ymin: 0.1,
        ymax: 0.1,
        zmin: 0.1,
        zmax: 0.1,
        tmin: 0.1,
        tmax: 0.1, 
        designation,
        buffer,
    };
    let buffer = &[42, 0, 0, 0, unsafe { std::mem::transmute::<i8, u8>(-42) }];
    let md2 = Metadata {
        xmin: 10.0,
        xmax: 10.0,
        ymin: 10.0,
        ymax: 10.0,
        zmin: 10.0,
        zmax: 10.0,
        tmin: 10.0,
        tmax: 10.0, 
        designation,
        buffer,
    };
    let buffer = &[255, 255, 0, 0, unsafe { std::mem::transmute::<i8, u8>(127) }];
    let md3 = Metadata {
        xmin: 123.456,
        xmax: 123.456,
        ymin: 123.456,
        ymax: 123.456,
        zmin: 123.456,
        zmax: 123.456,
        tmin: 123.456,
        tmax: 123.456, 
        designation,
        buffer,
    };
    let data = vec![md1, md2, md3];
    mydb.insert_n_metadata(&data)?;

    let my_data = mydb.get_metadata_in_bb(
        -9.9, 9.9,
        -9.9, 9.9,
        -9.9, 9.9,
        -9.9, 9.9,
        designation,
       Some(1.0) 
    ).unwrap();
    println!("{my_data:#?}"); 

    Ok(())
}
