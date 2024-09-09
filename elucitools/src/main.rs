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

    match DesignationSpecification::from_text(&args.input) {
        Ok(_) => println!("All good!"),
        Err(e) => print!("{e}"),
    }
}
