use clap::Parser;
use elucidator::designation::DesignationSpecification;

/// Verify that a given designation string is valid
#[derive(Parser)]
struct Args {
    /// String to validate
    input: String,
}

fn main() {
    let args = Args::parse();

    match DesignationSpecification::from_str(&args.input) {
        Ok(o) => println!("All good!"),
        Err(e) => print!("{e}"),
    }
}
