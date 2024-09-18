use std::collections::HashMap;

use elucidator::value::DataValue;
use crate::error::*;

pub type Datum<'a> = HashMap<&'a str, DataValue>;
pub type Result<T, E = DatabaseError> = std::result::Result<T, E>;

#[cfg(feature = "rtree")]
use rstar::{RTreeObject, AABB, Point};

#[derive(Debug, Clone)]
pub struct Metadata<'a> {
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

pub trait Database {
    fn new(filename: Option<&str>, config: Option<&DatabaseConfig>) -> Result<Self> where Self: Sized;
    fn from_path(filename: &str) -> Result<Self> where Self: Sized;
    fn save_as(&self, filename: &str) -> Result<()>;
    fn insert_spec_text(&mut self, designation: &str, spec: &str) -> Result<()>;
    fn insert_metadata(&mut self, datum: &Metadata) -> Result<()>;
    fn insert_n_metadata(&mut self, data: &Vec<Metadata>) -> Result<()>;
    fn get_metadata_in_bb(
        &self,
        xmin: f64, xmax: f64,
        ymin: f64, ymax: f64,
        zmin: f64, zmax: f64,
        tmin: f64, tmax: f64,
        designation: &str,
        epsilon: Option<f64>,
    ) -> Result<Vec<Datum>>;
}

pub trait Config {
    fn new() -> Self where Self: Sized;
    fn from_json_file(filename: &str) -> Result<Self> where Self: Sized;
    fn to_json_file(&self, filename: &str) -> Result<()> where Self: Sized;
}

pub enum DatabaseConfig {
    #[cfg(feature = "rtree")]
    RTreeConfig(crate::backends::rtree::RTreeConfig),
    SqliteConfig(crate::backends::sqlite::SqliteConfig),
}


#[cfg(feature = "rtree")]
impl<'a> RTreeObject for Metadata<'a> {
    type Envelope = AABB<[f64; 4]>;

    fn envelope(&self) -> Self::Envelope
    {
        AABB::from_corners(
            (self.xmin, self.ymin, self.zmin, self.tmin).into(),
            (self.xmax, self.ymax, self.zmax, self.tmax).into(),
        )
    }
}