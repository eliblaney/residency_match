use std::cmp::max;
use crate::models::{Rankable, HasCouple, HasCapacity, Couple};

#[derive(Debug)]
pub enum MatchError {
    ProgramNotFound(String),
    ApplicantNotFound(String),
}

impl MatchError {
    pub fn to_string(&self) -> String {
        match self {
            MatchError::ProgramNotFound(s) => format!("Program not found: {}", s),
            MatchError::ApplicantNotFound(s) => format!("Applicant not found: {}", s),
        }
    }
}

#[derive(Clone)]
pub struct Matcher<'a, A, P>
where A: Rankable<P> + HasCouple + Clone,
      P: Rankable<A> + HasCapacity
{
    pub matches: Vec<(&'a P, Vec<&'a A>)>,
    pub unmatched_a: Vec<&'a A>,
    pub unmatched_p: Vec<&'a P>,
}

impl<'a, A, P> Matcher<'a, A, P>
where
    A: Rankable<P> + HasCouple + Clone,
    P: Rankable<A> + HasCapacity
{
    pub fn new() -> Matcher<'a, A, P> {
        Matcher {
            matches: Vec::new(),
            unmatched_a: Vec::new(),
            unmatched_p: Vec::new(),
        }
    }

    fn clear(&mut self) {
        self.matches.clear();
        self.unmatched_a.clear();
        self.unmatched_p.clear();
    }

    fn attempt_single_match(&mut self, applicant: &'a A) -> Result<(), MatchError>
    {
        // assumes applicant is not couples-matching
        assert!(applicant.get_couple().is_none());
        let ranking = applicant.ranking();
        for program_id in ranking.iter() {
            let program = self.matches.iter_mut()
                .find(|m| m.0.id() == *program_id)
                .ok_or(MatchError::ProgramNotFound(format!("single: program: program_id {} in ranking.iter()", program_id)))?;

            let program_ranking = program.0.ranking();
            let rank_position = program_ranking.iter().position(|&a| a == applicant.id());
            if rank_position.is_none() || program.0.capacity() == 0 {
                // if program did not rank applicant, try the next program
                continue;
            }
            if program.0.capacity() > program.1.len() as u8 {
                // if program has an opening, tentatively match applicant to program
                program.1.push(&applicant);
                assert!(program.1.len() as u8 <= program.0.capacity());
                return Ok(());
            }
            let rank_map: Vec<(usize, usize)> = program.1.iter().enumerate().map(|a| (
                a.0, // tentative match index
                program_ranking.iter()
                    .position(|&b| b == a.1.id())
                    .ok_or(MatchError::ApplicantNotFound(format!("single: rank_map: a.1.id() {} in program.1.iter()", a.1.id()))) // rank index
            )).map(|(a, b)| match b {
                Ok(b) => Ok((a, b)),
                Err(e) => Err(e)
            }).collect::<Result<Vec<(usize, usize)>, MatchError>>()?;
            let (weakest_index, weakest_rank) = rank_map.iter().max_by(|a, b| a.1.cmp(&b.1)).unwrap();
            if rank_position.unwrap() < *weakest_rank {
                // if program prefers applicant to someone already matched,
                // then applicant takes their spot & we re-attempt the displaced applicant
                let weakest_applicant: &A = program.1.swap_remove(*weakest_index);
                program.1.push(&applicant);
                assert!(program.1.len() as u8 <= program.0.capacity());
                return match weakest_applicant.get_couple() {
                    None => self.retry_match(weakest_applicant),
                    Some(c) => {
                        if let Some(i) = program.1.iter().position(|a| a.id() == c) {
                            // skip some work and retry couple directly
                            let weakest_applicant_couple = program.1.swap_remove(i);
                            self.attempt_couples_match(weakest_applicant, Some(weakest_applicant_couple))
                        } else {
                            // sorry mario, your princess is another castle
                            // (go find her)
                            self.retry_match(weakest_applicant)
                        }
                    }
                }
            }
        }
        // unmatched applicant
        self.unmatched_a.push(&applicant);
        Ok(())
    }

    fn attempt_couples_match(&mut self, applicant: &'a A, couple: Option<&'a A>) -> Result<(), MatchError>
    {
        let couple = match &couple {
            None => return self.attempt_single_match(&applicant),
            Some(b) => b
        };
        assert!(applicant.get_couple().is_some() && couple.get_couple().is_some());
        let ranking: Vec<(&u32, &u32)> = applicant.ranking().iter().zip(couple.ranking()).collect();
        for program_pair in ranking.iter() {
            let p0 = self.matches.iter()
                .find(|m| m.0.id() == *program_pair.0)
                .ok_or(MatchError::ProgramNotFound(format!("couples: p0: *program_pair.0 {} in matches.iter()", *program_pair.0)))?;
            let mut p1 = p0;
            let same_program = *program_pair.0 == *program_pair.1;
            if !same_program {
                p1 = self.matches.iter()
                    .find(|m| m.0.id() == *program_pair.1)
                    .ok_or(MatchError::ProgramNotFound(format!("couples: p1: *program_pair.1 {} in matches.iter()", *program_pair.1)))?;
            }

            // program ranklists
            let p_ranks = (p0.0.ranking(), p1.0.ranking());
            // position of each applicant on their respective program's ranklists
            let rank0 = p_ranks.0.iter().position(|&a| a == applicant.id());
            let rank1 = p_ranks.1.iter().position(|&a| a == couple.id());
            if rank0.is_none() || p0.0.capacity() == 0
                || rank1.is_none() || p1.0.capacity() == 0
                || (same_program && p0.0.capacity() == 1) {
                // if program did not rank applicant, try the next program
                continue;
            }
            let rank0 = rank0.unwrap();
            let rank1 = rank1.unwrap();

            // program 0's ranklist as (tentative-rank index, applicant rank)
            let mut r0_map: Vec<(usize, usize)> = p0.1.iter().enumerate().map(|a| (
                a.0, // tentative match index
                p_ranks.0.iter()
                    .position(|&b| b == a.1.id())
                    .ok_or(MatchError::ApplicantNotFound(format!("couples: p_ranks: a.1.id() {} in matches.iter()", a.1.id()))) // applicant rank index
            )).map(|(a, b)| match b {
                Ok(b) => Ok((a, b)),
                Err(e) => Err(e)
            }).collect::<Result<Vec<(usize, usize)>, MatchError>>()?;
            // sort by descending applicant rank
            r0_map.sort_by(|a, b| b.1.cmp(&a.1));
            // weakest tentative match
            let mut r0_worst_iter = r0_map.iter();

            if same_program {
                let available_spots = max(p0.0.capacity() - p0.1.len() as u8, 0);
                match available_spots {
                    0 => {
                        // if program has space for no applicants, replace the two weakest
                        // tentatively matched applicants to make room for both
                        let (weak_index0, weak_rank) = r0_worst_iter.next().unwrap();
                        if rank0 < *weak_rank && rank1 < *weak_rank {
                            let (mut weak_index1, weak_rank) = r0_worst_iter.next().unwrap();
                            if rank0 < *weak_rank && rank1 < *weak_rank {
                                // both applicants are preferred to both currently weakest matched applicants
                                // we re-attempt both displaced applicants
                                let p0 = self.matches.iter_mut()
                                    .find(|m| m.0.id() == *program_pair.0)
                                    .ok_or(MatchError::ProgramNotFound(format!("couples: &mut p0 (1): *program_pair.0 {} in matches.iter()", *program_pair.0)))?;
                                let weakest_applicant0: &A = p0.1.swap_remove(*weak_index0);
                                if *weak_index0 < weak_index1 {
                                    // adjust index for second removal
                                    weak_index1 = weak_index1 - 1;
                                }
                                let weakest_applicant1: &A = p0.1.swap_remove(weak_index1);
                                p0.1.push(applicant);
                                p0.1.push(couple);
                                assert!(p0.1.len() as u8 <= p0.0.capacity());
                                return match weakest_applicant0.get_couple() {
                                    Some(c) => {
                                        return if weakest_applicant0.get_couple().eq(&Some(weakest_applicant1.id())) {
                                            // we displaced both partners of the couple
                                            return self.attempt_couples_match(weakest_applicant0, Some(weakest_applicant1))
                                        } else if let Some(i) = p0.1.iter().position(|a| a.id() == c) {
                                            // we displaced one partner of the couple (and a rando)
                                            let extra = p0.1.swap_remove(i);
                                            self.attempt_couples_match(weakest_applicant0, Some(extra))?;
                                            self.retry_match(weakest_applicant1)
                                        } else {
                                            // neither displaced applicant is coupled at this same program
                                            self.retry_match(weakest_applicant1)?;
                                            self.retry_match(weakest_applicant0)
                                        };
                                    },
                                    None => {
                                        match weakest_applicant1.get_couple() {
                                            None => self.retry_match(weakest_applicant1)?,
                                            Some(c) => {
                                                if let Some(i) = p0.1.iter().position(|a| a.id() == c) {
                                                    // we displaced one partner of the couple (and a rando)
                                                    let extra = p0.1.swap_remove(i);
                                                    self.retry_match(weakest_applicant0)?;
                                                    return self.attempt_couples_match(weakest_applicant1, Some(extra));
                                                } else {
                                                    // neither displaced applicant is coupled at this same program
                                                    self.retry_match(weakest_applicant1)?
                                                }
                                            }
                                        };
                                        self.retry_match(weakest_applicant0)
                                    }
                                }
                            }
                        }
                    },
                    1 => {
                        // if program has space for one applicant, tentatively match both
                        // by replacing the weakest tentatively matched applicant to make more room
                        let (weak_index, weak_rank) = r0_worst_iter.next().unwrap();
                        if rank0 < *weak_rank && rank1 < *weak_rank {
                            // if program prefers applicant to someone already matched,
                            // then applicant takes their spot & we re-attempt the displaced applicant
                            let p0 = self.matches.iter_mut()
                                .find(|m| m.0.id() == *program_pair.0)
                                .ok_or(MatchError::ProgramNotFound(format!("couples: &mut p0 (2): *program_pair.0 {} in matches.iter()", *program_pair.0)))?;
                            let weakest_applicant: &A = p0.1.swap_remove(*weak_index);
                            p0.1.push(applicant);
                            p0.1.push(couple);
                            assert!(p0.1.len() as u8 <= p0.0.capacity());
                            return match weakest_applicant.get_couple() {
                                None => self.retry_match(weakest_applicant),
                                Some(c) => {
                                    if let Some(i) = p0.1.iter().position(|a| a.id() == c) {
                                        // skip some work and retry couple directly
                                        let weakest_applicant_couple = p0.1.swap_remove(i);
                                        self.attempt_couples_match(weakest_applicant, Some(weakest_applicant_couple))
                                    } else {
                                        // sorry mario, your princess is another castle
                                        // (go find her)
                                        self.retry_match(weakest_applicant)
                                    }
                                }
                            }
                        }
                    },
                    _ => {
                        assert!(p0.0.capacity() - p0.1.len() as u8 >= 2,
                                "couples: available_spots {} not >= 2; {} - {}",
                                available_spots, p0.0.capacity(), p0.1.len()
                        );
                        // if program has space for both applicants, tentatively match both
                        let p0 = self.matches.iter_mut()
                            .find(|m| m.0.id() == *program_pair.0)
                            .ok_or(MatchError::ProgramNotFound(format!("couples: &mut p0 (3): *program_pair.0 {} in matches.iter()", *program_pair.0)))?;
                        p0.1.push(applicant);
                        p0.1.push(couple);
                        assert!(p0.1.len() as u8 <= p0.0.capacity());
                        let test_p = self.matches.iter()
                            .find(|m| m.1.iter().any(|a| a.get_couple().eq(&Some(applicant.get_couple().unwrap()))));
                        assert!(&test_p.is_some(), "couples: program: any(couple) {} in matches.iter()", applicant.get_couple().unwrap());
                        return Ok(());
                    },
                };
            } else {
                assert_ne!(p0.0.id(), p1.0.id());
                if p0.0.capacity() > p0.1.len() as u8
                    && p1.0.capacity() > p1.1.len() as u8 {
                    // if programs both have an opening, tentatively match applicant to programs
                    {
                        let p0 = self.matches.iter_mut()
                            .find(|m| m.0.id() == *program_pair.0)
                            .ok_or(MatchError::ProgramNotFound(format!("couples: &mut p0 (4): *program_pair.0 {} in matches.iter()", *program_pair.0)))?;
                        p0.1.push(applicant);
                        assert!(p0.1.len() as u8 <= p0.0.capacity());
                    }
                    let p1 = self.matches.iter_mut()
                        .find(|m| m.0.id() == *program_pair.1)
                        .ok_or(MatchError::ProgramNotFound(format!("couples: &mut p1 (1): *program_pair.0 {} in matches.iter()", *program_pair.0)))?;
                    p1.1.push(couple);
                    assert!(p1.1.len() as u8 <= p1.0.capacity(), "couples: program {} p1.1.len() {} <= p1.0.capacity() {}", p1.0.id(), p1.1.len(), p1.0.capacity());
                    let test_p = self.matches.iter()
                        .find(|m| m.1.iter().any(|a| a.get_couple().eq(&Some(applicant.get_couple().unwrap()))));
                    assert!(&test_p.is_some(), "couples: program: any(couple) {} in matches.iter()", applicant.get_couple().unwrap());
                    return Ok(());
                }

                // program 1's ranklist as (tentative-rank index, applicant rank)
                let mut r1_map: Vec<(usize, usize)> = p1.1.iter().enumerate().map(|a| (
                    a.0, // tentative match index
                    p_ranks.1.iter()
                        .position(|&b| b == a.1.id())
                        .ok_or(MatchError::ApplicantNotFound(format!("couples: r1_map: a.1.id() {} in p_ranks.1.iter()", a.1.id()))) // applicant rank index
                )).map(|(a, b)| match b {
                    Ok(b) => Ok((a, b)),
                    Err(e) => Err(e)
                }).collect::<Result<Vec<(usize, usize)>, MatchError>>()?;
                // sort by descending applicant rank
                r1_map.sort_by(|a, b| b.1.cmp(&a.1));
                // weakest tentative match
                let (r0_worst_index, r0_worst_rank) = r0_worst_iter.next().unwrap();
                let (mut r1_worst_index, r1_worst_rank) = r1_map.iter().next().unwrap();

                if rank0 < *r0_worst_rank && rank1 < *r1_worst_rank {
                    // if both applicants are preferred to both weakest tentatively matched applicants
                    // we re-attempt the two displaced applicants
                    let weakest_applicant0: &A;
                    {
                        let p0 = self.matches.iter_mut()
                            .find(|m| m.0.id() == *program_pair.0)
                            .ok_or(MatchError::ProgramNotFound(format!("couples: &mut p0 (5): *program_pair.0 {} in matches.iter()", *program_pair.0)))?;
                        weakest_applicant0 = p0.1.swap_remove(*r0_worst_index);
                        p0.1.push(applicant);
                        assert!(p0.1.len() as u8 <= p0.0.capacity());
                    }
                    let p1 = self.matches.iter_mut()
                        .find(|m| m.0.id() == *program_pair.1)
                        .ok_or(MatchError::ProgramNotFound(format!("couples: &mut p1 (2): *program_pair.0 {} in matches.iter()", *program_pair.0)))?;
                    if *r0_worst_index < r1_worst_index {
                        // adjust index for second removal
                        r1_worst_index = r1_worst_index - 1;
                    }
                    let weakest_applicant1: &A = p1.1.swap_remove(r1_worst_index);
                    p1.1.push(couple);
                    assert!(p1.1.len() as u8 <= p1.0.capacity());
                    let test_p = self.matches.iter()
                        .find(|m| m.1.iter().any(|a| a.get_couple().eq(&Some(applicant.get_couple().unwrap()))));
                    assert!(&test_p.is_some(), "couples: program: any(couple) {} in matches.iter()", applicant.get_couple().unwrap());

                    self.retry_match(weakest_applicant0)?;
                    self.retry_match(weakest_applicant1)?;

                    return Ok(());
                }
            }
        }

        self.unmatched_a.push(applicant);
        self.unmatched_a.push(couple);
        Ok(())
    }

    fn retry_match(&mut self, applicant: &'a A) -> Result<(), MatchError> {
        match applicant.get_couple() {
            None => self.attempt_single_match(applicant),
            Some(couple) => {
                let program = self.matches.iter_mut()
                    .find(|m| m.1.iter().any(|a| a.get_couple().eq(&Some(couple))));
                assert!(&program.is_some(), "retry: program: any(couple) {} in matches.iter()", couple);
                let program = program.unwrap();
                let index = program.1.iter()
                    .position(|a| a.get_couple() == Some(couple))
                    .ok_or(MatchError::ApplicantNotFound(format!("retry: index: couple {} in program.1.iter()", couple)))?;
                let couple = program.1.swap_remove(index);
                self.attempt_couples_match(applicant, Some(couple))
            }
        }
    }

    pub fn run_match(&mut self, a: &'a mut Vec<Couple<A>>, p: &'a Vec<P>) -> Result<(), MatchError> {
        self.clear();
        self.matches = p.into_iter().map(|p| (p, Vec::new())).collect();

        a.iter_mut().map(|i| self.attempt_couples_match(&i.0, i.1.as_ref()))
            .collect::<Vec<_>>()
            .into_iter().collect::<Result<Vec<()>, MatchError>>()?;

        self.finalize();
        Ok(())
    }

    fn finalize(&mut self) {
        self.unmatched_p = self.matches.iter()
            .filter(|m| m.1.len() == 0)
            .map(|m| m.0)
            .collect();
        self.matches = self.matches.iter()
            .filter(|m| m.1.len() > 0)
            .map(|m| (m.0, m.1.clone()))
            .collect();
    }

    pub fn unfilled_positions(&self) -> u32 {
        self.unmatched_p.iter().map(|p| p.capacity() as u32).sum::<u32>()
            + self.matches.iter().map(|m| {
            let diff = m.0.capacity() as i32 - m.1.len() as i32;
            assert!(diff >= 0, "unfilled_positions: program {} has overfilled capacity: {}/{} (extra = {})", m.0.id(), m.1.len(), m.0.capacity(), -diff);
            diff as u32
        }).sum::<u32>()
    }

}