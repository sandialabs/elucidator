use rusqlite::{Connection, Result};

#[derive(Debug)]
struct DesignationSpec {
    designation: String,
    spec: String,
}

#[derive(Debug)]
struct Metadata<'a> {
    xmin: f64,
    xmax: f64,
    ymin: f64,
    ymax: f64,
    zmin: f64,
    zmax: f64,
    tmin: f64,
    tmax: f64,
    designation: &'a str,
    buffer: &'a [u8],
}

fn main() -> Result<()> {
    let conn = Connection::open_in_memory()?;

    conn.execute(
        "CREATE TABLE designation_spec (
            designation  TEXT NOT NULL PRIMARY KEY,
            spec  TEXT NOT NULL
        )",
        (), // empty list of parameters.
    )?;
    conn.execute(
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
     
    let entry  = DesignationSpec {
        designation: "FooBar".to_string(),
        spec: "foo: u32, bar: i8".to_string(),
    };
    conn.execute(
        "INSERT INTO designation_spec (designation, spec) VALUES (?1, ?2)",
        (&entry.designation, &entry.spec),
    )?;

    let mut stmt = conn.prepare("SELECT designation, spec FROM designation_spec")?;
    let designation_iter = stmt.query_map([], |row| {
        Ok(DesignationSpec {
            designation: row.get(0)?,
            spec: row.get(1)?,
        })
    })?;
    let designation = "FooBar";
    let buffer = &[0; 9];
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
    conn.execute(
        "INSERT INTO Metadata (xmin, xmax, ymin, ymax, zmin, zmax, tmin, tmax, designation, buffer) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
        (
            &md.xmin,
            &md.xmax,
            &md.ymin,
            &md.ymax,
            &md.zmin,
            &md.zmax,
            &md.tmin,
            &md.tmax, 
            &md.designation,
            &md.buffer, 
        ),
    )?;


    for designation in designation_iter {
        println!("Found designation {:?}", designation.unwrap());
    }
    
    println!("Here");
    let mut stmt = conn.prepare("SELECT xmin, xmax, ymin, ymax, zmin, zmax, tmin, tmax, designation, buffer FROM Metadata")?;
    println!("Now Here");
    let mut rows = stmt.query([])?;
    while let Some(row) = rows.next()? {
        println!("And Now Here");
        let x = Metadata {
            xmin: row.get_unwrap(0),
            xmax: row.get_unwrap(1),
            ymin: row.get_unwrap(2),
            ymax: row.get_unwrap(3),
            zmin: row.get_unwrap(4),
            zmax: row.get_unwrap(5),
            tmin: row.get_unwrap(6),
            tmax: row.get_unwrap(7), 
            designation: row.get_ref_unwrap(8).as_str().unwrap(),
            buffer: row.get_ref_unwrap(9).as_blob().unwrap(),
        };
        println!("{x:?}");
    }
    Ok(())
}