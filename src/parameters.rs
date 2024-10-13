use serde::{Deserialize, Serialize};
use crate::models::{Applicant, Couple, Program};

#[derive(Clone, Serialize, Deserialize)]
pub struct MatchParameters {
    pub applicants: Vec<Couple<Applicant>>,
    pub programs: Vec<Program>,
    pub num_programs: usize,
    pub num_applicants: usize,
}

impl MatchParameters {
    pub fn save(&self, path: &str) -> bincode::Result<()> {
        let file = std::fs::File::create(path)?;
        let writer = std::io::BufWriter::new(file);
        bincode::serialize_into(writer, self)
    }
    
    pub fn open(path: &str) -> bincode::Result<Self> {
        let file = std::fs::File::open(path)?;
        let reader = std::io::BufReader::new(file);
        bincode::deserialize_from(reader)
    }
}