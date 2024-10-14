mod generator;

use std::sync::atomic::{AtomicU32, Ordering};
use serde::{Deserialize, Serialize};
use crate::ranker::{Competitive, Rankable, ReceiveApplication};

static APPLICANT_COUNTER: AtomicU32 = AtomicU32::new(0);
static PROGRAM_COUNTER: AtomicU32 = AtomicU32::new(0);

pub trait HasCouple {
    fn get_couple(&self) -> Option<u32>;
}

pub trait HasApplications {
    fn applications(&self) -> u8;
}

pub trait HasCapacity {
    fn capacity(&self) -> u8;
}
/*
#[derive(Clone)]
pub enum MatchStatus {
    UNMATCHED, MATCHED, PENDING
}
*/
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Applicant {
    pub id: u32,
    // pub name: String,
    // pub age: u8,
    // pub email: String,
    // pub phone: String,
    // pub status: MatchStatus,
    pub applications: u8,
    pub competitiveness: f32,
    pub couple: Option<u32>,
    pub ranking: Vec<u32>,
}

impl Applicant {
/*    pub fn new(name: String, age: u8, email: String, phone: String, applications: u8, competitiveness: f32, couple: Option<u32>) -> Applicant {
        Applicant {
            id: APPLICANT_COUNTER.fetch_add(1, Ordering::SeqCst),
            name,
            age,
            email,
            phone,
            applications,
            competitiveness,
            couple,
            ranking: Vec::new(),
        }
    }
*/
    fn sample_applicant_(can_couple: bool) -> (Applicant, Option<Applicant>) {
        let id = APPLICANT_COUNTER.fetch_add(1, Ordering::SeqCst);
        let mut competitiveness = generator::random_competitiveness();
        let mut couple: Option<Applicant> = match can_couple && generator::is_coupled() {
            true => Some(Applicant::sample_applicant_(false).0),
            false => None
        };
        if let Some(a) = &mut couple {
            a.couple = Some(id);
            competitiveness = (competitiveness + a.competitiveness) / 2.0;
            a.competitiveness = competitiveness;
            a.applications = (competitiveness * 100.0) as u8 + 1;
        }
        (
            Applicant {
                id,
                // name: generator::random_name(),
                // age: generator::random_age(),
                // email: generator::random_email(),
                // phone: generator::random_phone(),
                // status: MatchStatus::PENDING,
                applications: (competitiveness * 100.0) as u8 + 1,
                competitiveness,
                couple: couple.clone().map(|a| a.id),
                ranking: Vec::new(),
            },
            couple
        )
    }

    pub fn sample_applicant() -> (Applicant, Option<Applicant>) {
        Self::sample_applicant_(true)
    }
}

impl Competitive for Applicant {
    fn competitiveness(&self) -> f32 {
        self.competitiveness
    }
}

impl HasCouple for Applicant {
    fn get_couple(&self) -> Option<u32> {
        self.couple
    }
}

impl Rankable<Program> for Applicant {
    fn id(&self) -> u32 {
        self.id
    }

    fn ranking(&self) -> Vec<u32> {
        self.ranking.clone()
    }

    fn add_ranking(&mut self, to_add: &Program) {
        self.ranking.push(to_add.id)
    }
}

impl HasApplications for Applicant {
    fn applications(&self) -> u8 {
        self.applications
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Program {
    pub id: u32,
    // pub name: String,
    // pub deadline: DateTime<Utc>,
    pub capacity: u8,
    pub competitiveness: f32,
    pub ranking: Vec<(u32, f32)>,
}

impl Program {
/*    pub fn new(name: String, deadline: DateTime<Utc>, capacity: u8, competitiveness: f32) -> Program {
        Program {
            id: PROGRAM_COUNTER.fetch_add(1, Ordering::SeqCst),
            name,
            // deadline,
            capacity,
            competitiveness,
            ranking: Vec::new(),
        }
    }
*/
    pub fn sample_program() -> Program {
        Program {
            id: PROGRAM_COUNTER.fetch_add(1, Ordering::SeqCst),
            // name: generator::random_name(),
            // deadline: generator::random_deadline(),
            capacity: generator::random_capacity(),
            competitiveness: generator::random_competitiveness(),
            ranking: Vec::new(),
        }
    }
}

impl Competitive for Program {
    fn competitiveness(&self) -> f32 {
        self.competitiveness
    }
}

impl Rankable<Applicant> for Program {
    fn id(&self) -> u32 {
        self.id
    }

    fn ranking(&self) -> Vec<u32> {
        self.ranking.iter().map(|(a, _)| *a).collect()
    }

    fn add_ranking(&mut self, to_add: &Applicant) {
        self.ranking.push((to_add.id, to_add.competitiveness))
    }
}

impl HasCapacity for Program {
    fn capacity(&self) -> u8 {
        self.capacity
    }
}

impl ReceiveApplication<Applicant> for Program {
    fn receive_application(&mut self, applicant: &Applicant) {
        self.add_ranking(&applicant);
        self.ranking.sort_by(|(_, comp_a), (_, comp_b)| comp_b.total_cmp(comp_a));
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Couple<A>(pub A, pub Option<A>);

impl<A> From<(A, A)> for Couple<A> {
    fn from(couple: (A, A)) -> Couple<A> {
        Couple(couple.0, Some(couple.1))
    }
}

impl<A> From<(A, Option<A>)> for Couple<A> {
    fn from(couple: (A, Option<A>)) -> Couple<A> {
        Couple(couple.0, couple.1)
    }
}

