use crate::models::{Couple, HasApplications};

pub trait Competitive {
    fn competitiveness(&self) -> f32;
}

pub trait ReceiveApplication<A>
where A: Competitive
{
    fn receive_application(&mut self, applicant: &A);
}

pub struct RankStrategy {
    pub reach_multiplier: f32,
    pub realistic_multiplier: f32,
    pub safety_multiplier: f32
}

pub struct RankDistribution {
    pub reach: f32,
    pub realistic: f32,
    pub safety: f32
}

pub trait Rankable<T: Competitive>
where Self: Competitive
{
    fn id(&self) -> u32;
    fn ranking(&self) -> Vec<u32>;
    fn add_ranking(&mut self, to_add: &T);

    fn naive_rank(&mut self, to_rank: &mut Vec<&T>, num: u8) {
        to_rank.sort_by(|a, b| {
            (a.competitiveness() - self.competitiveness())
                .abs()
                .partial_cmp(&(b.competitiveness() - self.competitiveness()).abs())
                .unwrap()
        });

        for program in to_rank.iter().take(num.into()) {
            self.add_ranking(program);
        }
    }
}

pub fn rank<A, P>(applicant: &mut Couple<A>, programs: &mut Vec<P>, strategy: &RankStrategy, distribution: &RankDistribution)
where A: Rankable<P> + HasApplications + Competitive,
      P: Rankable<A> + ReceiveApplication<A> + Competitive
{
    let a = &mut applicant.0;
    let reach: Vec<u32> = programs.iter()
        .filter(|p|
            p.competitiveness() >= f32::min(0.99, strategy.reach_multiplier * a.competitiveness())
        )
        .take((a.applications() as f32 * distribution.reach) as usize)
        .map(|p| p.id())
        .collect();
    let realistic: Vec<u32> = programs.iter()
        .filter(|p|
            p.competitiveness() < strategy.reach_multiplier * a.competitiveness() &&
                p.competitiveness() >= f32::min(0.99, strategy.realistic_multiplier * a.competitiveness())
        )
        .take((a.applications() as f32 * distribution.realistic) as usize)
        .map(|p| p.id())
        .collect();
    let safety: Vec<u32> = programs.iter()
        .filter(|p|
            p.competitiveness() < strategy.realistic_multiplier * a.competitiveness() &&
                p.competitiveness() >= f32::min(0.95, strategy.safety_multiplier * a.competitiveness())
        )
        .take((a.applications() as f32 * distribution.safety) as usize)
        .map(|p| p.id())
        .collect();
    let all = [reach, realistic, safety].concat();
    for i in &all {
        let p = programs.iter_mut().find(|p| p.id() == *i).unwrap();
        a.add_ranking(p);
        p.receive_application(&a);
    }
    if let Some(b) = applicant.1.as_mut() {
        for i in all {
            let p = programs.iter_mut().find(|p| p.id() == i).unwrap();
            b.add_ranking(p);
            p.receive_application(&b);
        }
    }
}