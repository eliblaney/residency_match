use crate::matcher::Matcher;
use crate::parameters::MatchParameters;
use crate::models::{Applicant, Couple, HasCouple, Program};
use crate::ranker::{RankDistribution, RankStrategy, Rankable, ReceiveApplication};
use std::io::{Write, stdout};
use crossterm::{QueueableCommand, cursor, terminal, ExecutableCommand};
use std::time::Instant;
use crate::ranker;

trait AsMinutes {
    fn as_minutes(&self) -> f32;
}
impl AsMinutes for std::time::Duration {
    fn as_minutes(&self) -> f32 {
        self.as_secs_f32() / 60.0
    }
}

pub fn generate_match_parameters(num_applicants: usize, num_programs: usize) -> MatchParameters {
    generate_match_parameters_(num_applicants, num_programs, false)
}

pub fn generate_match_parameters_(num_applicants: usize, num_programs: usize, rank_naive: bool) -> MatchParameters {
    let (mut applicants, mut programs) = generate_population_pool(num_applicants, num_programs);
    let num_applicants = num_applicants + applicants.iter()
        .filter(|a| a.1.is_some()).collect::<Vec<_>>().len();

    if rank_naive {
        generate_naive_rankings(&mut applicants, &mut programs);
    } else {
        generate_rankings(&mut applicants, &mut programs);
    }

    MatchParameters {
        num_applicants,
        num_programs,
        applicants,
        programs
    }
}

pub fn generate_population_pool(num_applicants: usize, num_programs: usize) -> (Vec<Couple<Applicant>>, Vec<Program>) {
    let start = Instant::now();

    let applicants: Vec<Couple<Applicant>> = (0..num_applicants)
        .map(|_| Applicant::sample_applicant().into())
        .collect();
    let programs: Vec<Program> = (0..num_programs)
        .map(|_| Program::sample_program())
        .collect();

    println!("Created sample applicants and programs in {:.2?}.", start.elapsed());

    (applicants, programs)
}

pub fn generate_naive_rankings(applicants: &mut Vec<Couple<Applicant>>, programs: &mut Vec<Program>) {
    let start = Instant::now();

    let mut all_programs = programs.iter().collect::<Vec<_>>();
    animated_process(applicants,
                     |i, len|
                         format!("...Ranked {}/{} applicants ({:.0}%)...",
                                 i, len, i as f64 / len as f64 * 100.0),
                     |c| {
                         c.0.naive_rank(&mut all_programs, c.0.applications);
                         if let Some(couple) = c.1.as_mut() {
                             couple.ranking = c.0.ranking.clone();
                         }
                     }
    );

    let mut all_applicants = applicants.iter().flat_map(|c| match &c.1 {
        Some(couple) => vec![&c.0, &couple],
        None => vec![&c.0]
    }).collect::<Vec<_>>();
    animated_process(programs,
                     |i, len|
                         format!("...Ranked {}/{} programs ({:.0}%)...",
                                 i, len, i as f64 / len as f64 * 100.0),
                     |program| {
                         program.naive_rank(
                             &mut all_applicants,
                             program.capacity * 15
                         );
                     }
    );

    println!("Built rankings in {:.2?}.", start.elapsed());
}

pub fn generate_rankings(applicants: &mut Vec<Couple<Applicant>>, programs: &mut Vec<Program>) {
    let start = Instant::now();

    let strategy = RankStrategy {
        reach_multiplier: 1.05,
        realistic_multiplier: 0.95,
        safety_multiplier: 0.90
    };
    let distribution = RankDistribution {
        reach: 0.3,
        realistic: 0.5,
        safety: 0.2
    };

    programs.sort_by(|a, b| b.competitiveness.total_cmp(&a.competitiveness));
    animated_process(applicants,
                     |i, len|
                         format!("...Ranked {}/{} applicants ({:.0}%)...",
                                 i, len, i as f64 / len as f64 * 100.0),
                     |c|
                         ranker::rank(c, programs, &strategy, &distribution)
    );

    animated_process(programs,
                     |i, len|

                         format!("...Ranked {}/{} applicants ({:.0}%)...",
                                 i, len, i as f64 / len as f64 * 100.0),
                     |p|
                         p.process_applications()
    );

    println!("Built rankings in {:.2?}min.", start.elapsed().as_minutes());
}

pub fn run_simulation(parameters: MatchParameters) {
    let MatchParameters {
        num_applicants,
        num_programs,
        applicants,
        programs
    } = parameters;

    let start = Instant::now();

    let mut matcher = Matcher::new();
    match matcher.run_match(&applicants, &programs) {
        Err(ref e) => {
            eprintln!("Error while matching: {:?}", e.to_string());
            return
        },
        Ok(_) => println!("Finished match in {:.2?}min.", start.elapsed().as_minutes())
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
        println!("{} (#{})", a.id(), sample_program.ranking.iter().position(|i| *i == a.id()).unwrap() + 1);
    });
}

pub fn animated_process<T, S, F>(v: &mut Vec<T>, s: S, mut f: F)
where
    S: Fn(usize, usize) -> String,
    F: FnMut(&mut T)
{
    let len = v.len();
    let mut i = 0usize;
    let mut stdout = stdout();
    for t in v {
        f(t);
        i += 1;
        stdout.queue(cursor::SavePosition).unwrap();
        stdout.write_all(s(i, len).as_ref()).unwrap();
        stdout.queue(cursor::RestorePosition).unwrap();
        stdout.flush().unwrap();
        stdout.queue(cursor::RestorePosition).unwrap();
        stdout.queue(terminal::Clear(terminal::ClearType::FromCursorDown)).unwrap();
    }
    stdout.execute(cursor::Show).unwrap();
}
