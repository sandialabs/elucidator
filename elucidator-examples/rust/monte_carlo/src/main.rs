use std::iter::Enumerate;

use elucidator::value::DataValue;
use elucidator::Representable;
use elucidator_db::backends::rtree::RTreeDatabase;
use elucidator_db::database::{Database, Metadata};
use elucidator_db::error::DatabaseError;

use rand::prelude::*;

use std::vec::Vec;

struct StepSummary {
    hits: u64,
    misses: u64,
}

struct AnalysisResult {
    timestep: usize,
    total_hits: u64,
    total_misses: u64,
    pi_estimate: f64,
    pi_95_ci: (f64, f64),
}

fn simulate_step(trials: usize) -> StepSummary {
    let mut rng = rand::thread_rng();
    let mut hits = 0;
    let mut misses = 0;
    for _ in 0..trials {
        let x: f32 = rng.gen_range(-1.0..=1.0);
        let y: f32 = rng.gen_range(-1.0..=1.0);
        if x.powi(2) + y.powi(2) <= 1.0 {
            hits += 1;
        } else {
            misses += 1;
        }
    }
    StepSummary { hits, misses }
}

fn run_experiment(
    db: &mut dyn Database,
    n_steps: usize,
    samples_per_step: usize,
) -> Result<(), DatabaseError> {
    for idx in 0..n_steps {
        let step = simulate_step(samples_per_step);
        let mut buffer: Vec<u8> = Vec::with_capacity(8);
        buffer.extend(step.hits.as_buffer());
        buffer.extend(step.misses.as_buffer());
        let md = Metadata {
            xmin: -1.0,
            xmax: 1.0,
            ymin: -1.0,
            ymax: 1.0,
            zmin: -1.0,
            zmax: 1.0,
            tmin: idx as f64,
            tmax: idx as f64,
            designation: "state",
            buffer: &buffer,
        };
        db.insert_metadata(&md)?;
    }
    Ok(())
}

fn calc_confidence_interval(hits: f64, misses: f64, zscore: f64) -> (f64, f64) {
    let p = hits / (hits + misses);
    let se = (p * (1.0 - p) / (hits + misses)).sqrt();
    let pi_upper_bound = 4.0 * (p + zscore * se);
    let pi_lower_bound = 4.0 * (p - zscore * se);
    (pi_lower_bound, pi_upper_bound)
}

fn calc_pi_estimate(hits: f64, misses: f64) -> f64 {
    if hits + misses == 0.0 {
        return 0.0;
    }
    hits as f64 / (hits + misses) as f64 * 4.0
}

fn analyze(db: &mut dyn Database, timestep: usize) -> Result<AnalysisResult, DatabaseError> {
    const Z_SCORE_95_CI: f64 = 1.959963984540054;
    let mut total_hits: u64 = 0;
    let mut total_misses: u64 = 0;
    let pi_estimate: f64;
    let pi_95_ci: (f64, f64);
    let data = db.get_metadata_in_bb(
        -1.0,
        1.0,
        -1.0,
        1.0,
        -1.0,
        1.0,
        0.0,
        timestep as f64,
        "state",
        None,
    )?;
    for metadata in data {
        let hits = metadata
            .get("hits")
            .unwrap_or(&DataValue::UnsignedInteger64(0));
        let misses = metadata
            .get("misses")
            .unwrap_or(&DataValue::UnsignedInteger64(0));
        match hits {
            DataValue::UnsignedInteger64(h) => total_hits += h,
            _ => {
                unreachable!()
            }
        }
        match misses {
            DataValue::UnsignedInteger64(m) => total_misses += m,
            _ => {
                unreachable!()
            }
        }
    }

    pi_estimate = calc_pi_estimate(total_hits as f64, total_misses as f64);
    pi_95_ci = calc_confidence_interval(total_hits as f64, total_misses as f64, Z_SCORE_95_CI);

    Ok(AnalysisResult {
        timestep,
        total_hits,
        total_misses,
        pi_estimate,
        pi_95_ci,
    })
}

fn main() {
    const N_STEPS: usize = 100000;
    const SAMPLES_PER_STEP: usize = 50;
    const DISPLAY_INTERVAL: usize = 5000;

    let mut db: RTreeDatabase = Database::new(None, None).unwrap();
    db.insert_spec_text("state", "hits: u64, misses: u64")
        .unwrap();

    run_experiment(&mut db, N_STEPS, SAMPLES_PER_STEP).unwrap();
    for t in (DISPLAY_INTERVAL..=N_STEPS).step_by(DISPLAY_INTERVAL) {
        let analysis = analyze(&mut db, t).unwrap();
        println!(
            "Timestep {t}: Pi ~= {}, 95% CI ({}, {})",
            analysis.pi_estimate, analysis.pi_95_ci.0, analysis.pi_95_ci.1
        );
    }
}
