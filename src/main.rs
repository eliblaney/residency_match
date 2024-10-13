use std::time::Instant;
use crate::matcher::Matcher;
use crate::parameters::MatchParameters;
use crate::models::{Applicant, Couple, HasCouple, Program, Rankable};

mod models;
mod matcher;
mod parameters;

const NUM_APPLICANTS: usize = 50000;
const NUM_PROGRAMS: usize = 10000;


fn main() {
    let datafile = "data.bin".into();

    let parameters = load_params(Some(datafile));

    // save_params(&parameters, datafile);

    run_simulation(parameters);
}

fn load_params(datafile: Option<String>) -> MatchParameters {
    match datafile {
        None => {
            generate_match_parameters(NUM_APPLICANTS, NUM_PROGRAMS)
        },
        Some(datafile) => {
            let start = Instant::now();
            let parameters = MatchParameters::open(&datafile).unwrap();
            println!("Loaded data from {} in {:.2?}.", &datafile, start.elapsed());
            parameters
        }
    }
}

fn save_params(parameters: &MatchParameters, datafile: String) {
    let start = Instant::now();
    parameters.save(&datafile).unwrap();
    println!("Saved data to {} in {:.2?}.", &datafile, start.elapsed());
}

fn run_simulation(parameters: MatchParameters) {
    let MatchParameters {
        num_applicants,
        num_programs,
        mut applicants,
        programs
    } = parameters;

    let start = Instant::now();

    let mut matcher = Matcher::new();
    match matcher.run_match(&mut applicants, &programs) {
        Err(ref e) => {
            eprintln!("Error while matching: {:?}", e.to_string());
            return
        },
        Ok(_) => println!("Finished match in {:.2?}.", start.elapsed())
    };

    let matched_programs = matcher.matches.len();
    let unfilled_positions = matcher.unfilled_positions();
    let matched_applicants = matcher.matches.iter()
        .flat_map(|m| &m.1)
        .collect::<Vec<&&Applicant>>()
        .len();
    let unmatched_applicants = matcher.unmatched_a.len();
    let matched_couples = matcher.matches.iter().flat_map(|m| &m.1).filter(|a| a.get_couple().is_some()).collect::<Vec<_>>().len();
    let unmatched_couples = matcher.unmatched_a.iter().filter(|a| a.get_couple().is_some()).collect::<Vec<_>>().len();

    println!("Matched programs: {} ({:.1}%), Unfilled positions: {}",
             matched_programs, matched_programs as f32 / num_programs as f32 * 100.0,
             unfilled_positions
    );
    println!("Matched applicants: {} ({:.1}%), Unmatched applicants: {} ({:.1}%)",
                matched_applicants, matched_applicants as f32 / num_applicants as f32 * 100.0,
                unmatched_applicants, unmatched_applicants as f32 / num_applicants as f32 * 100.0,
    );
    println!("Matched couples: {} ({:.1}%), Unmatched couples: {} ({:.1}%)",
             matched_couples, matched_couples as f32 / (matched_couples + unmatched_couples) as f32 * 100.0,
             unmatched_couples, unmatched_couples as f32 / (matched_couples + unmatched_couples) as f32 * 100.0,
    );

    let first_choicers = matcher.matches.iter()
        .filter(|m| m.1.iter().any(|a| a.ranking[0] == m.0.id()))
        .flat_map(|m| &m.1)
        .collect::<Vec<_>>();
    let all_first_choice = first_choicers.len();

    println!("Number of applicants that matched their first choice: {} ({:.1}%)",
             all_first_choice, all_first_choice as f32 / matched_applicants as f32 * 100.0
    );

    let couples_first_choice = first_choicers.iter()
        .filter(|a| a.get_couple().is_some())
        .collect::<Vec<_>>()
        .len();

    println!("Number of couples that matched their first choice: {} ({:.1}%)",
             couples_first_choice, couples_first_choice as f32 / matched_couples as f32 * 100.0
    );

    println!();

    let sample_match = matcher.matches.iter().find(|m| m.1.len() > 0).unwrap();
    let sample_program = sample_match.0;
    let sample_applicant = sample_match.1[0];

    println!("Sample applicant ({}) with competitiveness={} ranked {} programs and matched their #{} choice.",
             sample_applicant.id(), sample_applicant.competitiveness, sample_applicant.applications,
             sample_applicant.ranking.iter().position(|&p| p == sample_program.id()).unwrap() + 1);
    println!("Sample program ({}) with capacity {} and competitiveness={} ranked {} applicants and matched with {} applicants.",
             sample_program.id(), sample_program.capacity, sample_program.competitiveness,
             sample_program.ranking.len(), sample_match.1.len());
    println!("Sample program's matriculates and their position in program's rank list:");
    sample_match.1.iter().for_each(|a| {
        println!("{} (#{})", a.id(), sample_program.ranking.iter().position(|&i| i == a.id()).unwrap() + 1);
    });
}

fn generate_match_parameters(num_applicants: usize, num_programs: usize) -> MatchParameters {
    let start = Instant::now();

    let mut programs: Vec<Program> = (0..num_programs)
        .map(|_| Program::sample_program())
        .collect();
    let mut applicants: Vec<Couple<Applicant>> = (0..num_applicants)
        .map(|_| Applicant::sample_applicant().into())
        .collect();
    let num_applicants = num_applicants + applicants.iter()
        .filter(|a| a.1.is_some()).collect::<Vec<_>>().len();

    println!("Created sample applicants and programs in {:.2?}.", start.elapsed());
    let start = Instant::now();

    let mut all_programs = programs.iter().collect::<Vec<_>>();
    applicants.iter_mut().for_each(|c| {
        c.0.sample_ranking(&mut all_programs, c.0.applications);
        if let Some(couple) = c.1.as_mut() {
            couple.ranking = c.0.ranking.clone();
        };
    });
    let mut all_applicants = applicants.iter().flat_map(|c| match &c.1 {
        Some(couple) => vec![&c.0, &couple],
        None => vec![&c.0]
    }).collect::<Vec<_>>();

    programs.iter_mut().for_each(|program| {
        program.sample_ranking(
            &mut all_applicants,
            program.capacity * 15);
    });

    println!("Built rankings in {:.2?}.", start.elapsed());

    MatchParameters {
        num_applicants,
        num_programs,
        applicants,
        programs
    }
}
