use clap::Parser;
use elucidator::{
    representable::Representable
};
use elucidator_db::{
    database::{Config, Database, DatabaseConfig, Metadata},
    backends::rtree::RTreeDatabase,
    backends::sqlite::{SqliteConfig, SqlDatabase},
};
use rand::random;
use std::{
    fs::{File, OpenOptions},
    io::{prelude::*, Write},
    path::Path,
    time::Instant,
};

/// Create PDFs and time them
#[derive(Parser)]
struct Args {
    /// File to save result to; appends if exists
    #[arg(short, long)]
    savename: Option<String>,
    /// Number of PDFs
    count: usize,
    /// Number of PDF bins
    size: usize,
    /// Number of queries
    queries: usize,
}

fn rand_pair() -> (f64, f64) {
    let a: f64 = random();
    let b: f64 = random();
    if a > b {
        (b, a)
    } else {
        (a, b)
    }
}

static designation: &'static str = "pdf";

type Bb = (f64, f64, f64, f64, f64, f64, f64, f64);
fn random_bb() -> Bb {
    let (xmin, xmax) = rand_pair();
    let (ymin, ymax) = rand_pair();
    let (zmin, zmax) = rand_pair();
    let (tmin, tmax) = rand_pair();
    (
        xmin,
        xmax,
        ymin,
        ymax,
        zmin,
        zmax,
        tmin,
        tmax,
    )
}

fn metadata_from(buffer: &[u8]) -> Metadata {
    let (xmin, xmax) = rand_pair();
    let (ymin, ymax) = rand_pair();
    let (zmin, zmax) = rand_pair();
    let (tmin, tmax) = rand_pair();

    Metadata {
        xmin,
        xmax,
        ymin,
        ymax,
        zmin,
        zmax,
        tmin,
        tmax,
        designation,
        buffer
    }
}

fn main() {
    let Args {count, size, queries, savename} = Args::parse();
    let pdf_size = size * std::mem::size_of::<u32>();
    let spec = format!("pdf: u32[{}]", size);
    let random_vals: Vec<Vec<u8>> = (0..count)
        .map(|_| {
            (0..size)
                .map(|_| random::<u32>())
                .collect::<Vec<u32>>()
                .as_buffer()
        })
        .collect();
    let random_metadata: Vec<Metadata> = random_vals.iter()
        .map(|x| metadata_from(x))
        .collect();
    let start_time = Instant::now();
    let mut db = RTreeDatabase::new(None, None).unwrap();
    db.insert_spec_text("pdf", &spec).unwrap();
    for datum in &random_metadata {
        db.insert_metadata(datum).unwrap();
    }
    let elapsed_insertion = start_time.elapsed();
    drop(random_metadata);
    drop(random_vals);
    let random_bbs: Vec<Bb> = (0..queries).map(|_| random_bb()).collect();
    let eps = 1e-16;
    let start_time = Instant::now();
    for x in random_bbs {
        db.get_metadata_in_bb(x.0, x.1, x.2, x.3, x.4, x.5, x.6, x.7, "pdf", Some(eps)).unwrap();
    }
    let elapsed_queries = start_time.elapsed();
    if let Some(fname) = savename {
        let p = Path::new(&fname);
        let mut file = if !p.exists() {
            let mut f = File::create(&p).unwrap();
            write!(&mut f, "count,size,queries,insertion,query\n").unwrap();
            f
        } else {
            OpenOptions::new()
                .append(true)
                .open(&fname)
                .unwrap()
        };

        let s = format!(
            "{count},{size},{queries},{},{}\n",
            elapsed_insertion.as_secs_f32(),
            elapsed_queries.as_secs_f32(),
        );
        write!(&mut file, "{s}").unwrap();
    } else {
        println!("Inserted {count} objects of size {pdf_size}, and performed {queries} queries.");
        println!("Insertion time: {elapsed_insertion:#?}");
        println!("Query time: {elapsed_queries:#?}");
    }
}
