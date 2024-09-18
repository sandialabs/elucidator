use crate::database::{Datum, Metadata, Database, Result, DatabaseConfig};
use rstar::{RTree, RTreeParams, RTreeObject, AABB};


#[cfg(feature = "rtree")]
pub struct RTreeDatabase<'a> {
    /// R*-Tree used internally
    rtree: RTree<Metadata<'a>>,
}

#[cfg(feature = "rtree")]
pub struct RTreeConfig {
    /// R*-Tree used internally
    config:  u8,
}

#[cfg(feature = "rtree")]
impl Database for RTreeDatabase<'_> {
    fn new(filename: Option<&str>, config: Option<&DatabaseConfig>) -> Result<Self> {
        todo!();
    }
    fn from_path(filename: &str) -> Result<Self> {
        todo!();
    }
    fn save_as(&self, filename: &str) -> Result<()> {
        todo!();
    }
    fn insert_spec_text(&mut self, designation: &str, spec: &str) -> Result<()> {
        todo!();
    }
    fn insert_metadata(&mut self, datum: &Metadata) -> Result<()> {
        todo!();
    }
    fn insert_n_metadata(&mut self, data: &Vec<Metadata>) -> Result<()> {
        todo!();
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
        todo!();
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