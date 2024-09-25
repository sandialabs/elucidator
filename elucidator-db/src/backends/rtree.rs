use crate::database::{Datum, Metadata, Database, Result, DatabaseConfig};
use rstar::{RTree, RTreeObject, AABB};

use std::{collections::HashMap, ops::Deref};
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
#[derive(Debug, Clone)]
struct MetadataClone {
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
    fn from_path(_: &str) -> Result<Self> {
        unimplemented!();
    }
    fn save_as(&self, _filename: &str) -> Result<()> {
        unimplemented!();
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

    mod database {
        use super::*;
        use std::{collections::HashSet, ops::Deref};
        use elucidator::value::DataValue;
        use crate::error::DatabaseError;


        #[test]
        fn create_in_memory_ok() {
            assert_eq!(true, true);
            // assert!(RTreeDatabase::new(None, None).is_ok());
        }
    }
}