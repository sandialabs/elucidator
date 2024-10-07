use crate::{backends::sqlite::SqlDatabase, database::{Database, DatabaseConfig, Datum, Metadata, Result}};
use rstar::{RTree, RTreeObject, AABB};

use std::collections::HashMap;
use elucidator::designation::DesignationSpecification;


#[derive(Debug)]
pub struct RTreeDatabase {
    /// R*-Tree used internally
    rtree: RTree<MetadataClone>,
    designations: HashMap<String, DesignationSpecification>,
}

pub struct RTreeConfig {
    /// R*-Tree used internally
    _config:  u8,
}
#[derive(Debug, Clone, PartialEq)]
pub struct MetadataClone {
    pub xmin: f64,
    pub xmax: f64,
    pub ymin: f64,
    pub ymax: f64,
    pub zmin: f64,
    pub zmax: f64,
    pub tmin: f64,
    pub tmax: f64,
    pub designation: String,
    pub buffer: Vec<u8>,
}

impl From<Metadata<'_>> for MetadataClone {
    fn from(m: Metadata) -> Self {
        MetadataClone {
            xmin: m.xmin,
            xmax: m.xmax,
            ymin: m.ymin,
            ymax: m.ymax,
            zmin: m.zmin,
            zmax: m.zmax,
            tmin: m.tmin,
            tmax: m.tmax,
            designation: m.designation.to_string(),
            buffer: m.buffer.into()
        }
    }
}

impl From<&Metadata<'_>> for MetadataClone {
    fn from(m: &Metadata) -> Self {
        MetadataClone {
            xmin: m.xmin,
            xmax: m.xmax,
            ymin: m.ymin,
            ymax: m.ymax,
            zmin: m.zmin,
            zmax: m.zmax,
            tmin: m.tmin,
            tmax: m.tmax,
            designation: m.designation.to_string(),
            buffer: m.buffer.into()
        }
    }
}

impl<'a> RTreeObject for &MetadataClone {
    type Envelope = AABB<[f64; 4]>;

    fn envelope(&self) -> Self::Envelope
    {
        AABB::from_corners(
            (self.xmin, self.ymin, self.zmin, self.tmin).into(),
            (self.xmax, self.ymax, self.zmax, self.tmax).into(),
        )
    }
}

impl<'a> RTreeObject for MetadataClone {
    type Envelope = AABB<[f64; 4]>;

    fn envelope(&self) -> Self::Envelope
    {
        AABB::from_corners(
            (self.xmin, self.ymin, self.zmin, self.tmin).into(),
            (self.xmax, self.ymax, self.zmax, self.tmax).into(),
        )
    }
}


impl Database for RTreeDatabase {
    fn new(_: Option<&str>, _: Option<&DatabaseConfig>) -> Result<Self> {
        Ok(Self {
            rtree: RTree::new(),
            designations: HashMap::new(),
        })
    }
    fn from_path(filename: &str) -> Result<Self> {
        let sqlite = SqlDatabase::from_path(filename)?;
        let designations = sqlite.get_designations();
        let mds = sqlite.get_all_metadata()?;
        let rtree = RTree::bulk_load(mds);
        Ok(RTreeDatabase {
            rtree,
            designations,
        })
    }
    fn save_as(&self, filename: &str) -> Result<()> {
        let mut sqlite = SqlDatabase::new(Some(filename), None)?;

        for (designation, designation_spec) in self.designations.iter() {
            sqlite.insert_spec_text(&designation, &designation_spec.to_string())?;
        }
        let md_results: Result<Vec<()>, crate::error::DatabaseError> = self.rtree.iter()
            .map(|m| {
                let md = Metadata {
                    xmin: m.xmin,
                    xmax: m.xmax,
                    ymin: m.ymin,
                    ymax: m.ymax,
                    zmin: m.zmin,
                    zmax: m.zmax,
                    tmin: m.tmin,
                    tmax: m.tmax,
                    designation: &m.designation,
                    buffer: &m.buffer,
                };
                sqlite.insert_metadata(&md)
            })
            .collect();
        md_results?;
        Ok(())
    }
    fn insert_spec_text(&mut self, designation: &str, spec: &str) -> Result<()> {
        let designation_spec = DesignationSpecification::from_text(spec)?;
        self.designations.insert(designation.to_string(), designation_spec);
        Ok(())
    }
    fn insert_metadata(&mut self, datum: &Metadata) -> Result<()> {
        self.rtree.insert(datum.into());
        Ok(())
    }
    fn insert_n_metadata(&mut self, data: &Vec<Metadata>) -> Result<()> {
        for datum in data {
            self.rtree.insert(datum.into());
        }
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
        let d = self.designations.get(designation).unwrap();
        let blobs = self.get_metadata_blobs_in_bb(xmin, xmax, ymin, ymax, zmin, zmax, tmin, tmax, designation, epsilon)?;
        Ok(blobs.iter()
            .map(|b| d.interpret_enum(b).unwrap())
            .collect()
        )

    }

    fn get_metadata_blobs_in_bb(
        &self,
        xmin: f64, xmax: f64,
        ymin: f64, ymax: f64,
        zmin: f64, zmax: f64,
        tmin: f64, tmax: f64,
        designation: &str,
        epsilon: Option<f64>,
    ) -> Result<Vec<&Vec<u8>>> {
        let eps = epsilon.unwrap_or(0.0);
        let mins = [xmin - eps, ymin - eps, zmin - eps, tmin - eps];
        let maxs = [xmax + eps, ymax + eps, zmax + eps, tmax + eps];
        
        let bb = AABB::from_corners(mins, maxs);
        Ok(
            self.rtree.locate_in_envelope(&bb)
                .filter(|m| m.designation == designation)
                .map(|m| &m.buffer)
                .collect()
        )
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
    
    mod config {
        use super::*;
        use pretty_assertions::assert_eq;
    }

    mod database {
        use super::*;
        use std::{collections::HashSet, ops::Deref};
        use elucidator::value::DataValue;
        use crate::error::DatabaseError;

        #[test]
        fn create_in_memory_ok() {
            assert!(RTreeDatabase::new(None, None).is_ok());
        }

        #[test]
        fn from_empty_ok() {
            let tempfile = TempFile::from("temp.db").unwrap();
            let db = SqlDatabase::new(Some(&tempfile.filepath), None);
            drop(db);
            let loaded_db = RTreeDatabase::from_path(&tempfile.filepath);
            assert!(loaded_db.is_ok());
        }

        #[test]
        fn insert_designation_ok() {
            let mut db = RTreeDatabase::new(None, None).unwrap();
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
            let mut db = RTreeDatabase::new(None, None).unwrap();

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
            let mut db = RTreeDatabase::new(None, None).unwrap();
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
            let mut db = RTreeDatabase::new(None, None).unwrap();

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
            let mut db = RTreeDatabase::new(None, None).unwrap();

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
                    [("foo", DataValue::Byte(100)),
                     ("bar", DataValue::Float32(1.0))]
                ),
                HashMap::from(
                    [("foo", DataValue::Byte(150)),
                     ("bar", DataValue::Float32(1000000.0))]
                ),
            ];
            assert!(result.is_ok());
            let result = result.unwrap();
            assert_eq!(result.len(), expected.len());
            for x in expected.iter() {
                assert!(result.contains(x));
            }
        }

        #[test]
        fn test_save_and_recover_ok() {
            let mut db = RTreeDatabase::new(None, None).unwrap();

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
             
            let tempfile = TempFile::from("temp.db").unwrap(); 
            let _ = db.save_as(&tempfile.filepath);
            
            let recovered = RTreeDatabase::from_path(&tempfile.filepath).unwrap();
            pretty_assertions::assert_eq!(db.designations, recovered.designations);
            let db_md = db.rtree.iter().collect::<Vec<&MetadataClone>>();
            let recovered_md = recovered.rtree.iter().collect::<Vec<&MetadataClone>>();
            assert_eq!(db_md.len(), recovered_md.len());
            for element in db_md.iter() {
                assert!(recovered_md.contains(element));
            }
        }
    }
}
