use rand;
use rand::Rng;

/*
fn random_string(n: usize) -> String {
    let chars: Vec<char> = (0..n).map(|_| rand::thread_rng().gen_range(65..91) as u8 as char).collect();
    chars.iter().collect()
}

pub fn random_email() -> String {
    format!("{}@{}.com", random_string(10), random_string(5))
}

pub fn random_phone() -> String {
    format!("{}-{}-{}", random_string(3), random_string(3), random_string(4))
}

pub fn random_age() -> u8 {
    rand::thread_rng().gen_range(25..=35)
}

pub fn random_name() -> String {
    let first = random_string(1).to_uppercase();
    let rest = random_string(6);
    format!("{}{}", first, rest)
}
*/

pub fn random_capacity() -> u8 {
    rand::thread_rng().gen_range(1..=10)
}

pub fn random_competitiveness() -> f32 {
    rand::thread_rng().gen_range(0f32..1f32)
}

/*
pub fn random_deadline() -> DateTime<Utc> {
    chrono::DateTime::default().add(
        chrono::Duration::days(rand::thread_rng().gen_range(14..=90))
    )
}
*/

pub fn is_coupled() -> bool {
    rand::thread_rng().gen_bool(0.02)
}