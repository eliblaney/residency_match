use std::time::Instant;
use crate::driver::{generate_match_parameters, run_simulation};
use crate::parameters::MatchParameters;

mod models;
mod matcher;
mod parameters;
mod ranker;
mod driver;

const NUM_APPLICANTS: usize = 50000;
const NUM_PROGRAMS: usize = 10000;

fn main() {
    let datafile: Option<&str> = None;

    let parameters = load_params(&datafile);

    if datafile.is_none() {
        save_params(&parameters, "data.bin");
    }

    run_simulation(parameters);
}

fn load_params(datafile: &Option<&str>) -> MatchParameters {
    match datafile {
        None => {
            generate_match_parameters(NUM_APPLICANTS, NUM_PROGRAMS)
        },
        Some(datafile) => {
            let start = Instant::now();
            let parameters = MatchParameters::open(*datafile).unwrap();
            println!("Loaded data from {} in {:.2?}.", datafile, start.elapsed());
            parameters
        }
    }
}

fn save_params(parameters: &MatchParameters, datafile: &str) {
    let start = Instant::now();
    parameters.save(datafile).unwrap();
    println!("Saved data to {} in {:.2?}.", datafile, start.elapsed());
}
