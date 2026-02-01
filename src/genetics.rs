use std::collections::HashSet;
use std::ops::Mul;
use std::time::{Duration, Instant};
use crate::planet_stats::PlanetStats;
use common_explorer::{ExplorerBag, ExplorerBagContent};
use common_game::components::resource::{
    BasicResourceType, ComplexResourceRequest, ComplexResourceType, GenericResource,
};
use common_game::utils::ID;
use rand::Rng;

const BASIC_RESOURCE_WEIGHT: u32 = 10;
const COMPLEX_RESOURCE_WEIGHT: u32 = 60;

#[derive(Debug)]
pub(crate) enum Intention {
    Generate(Option<BasicResourceType>),
    Combine(Option<ComplexResourceRequest>),
    Move(Option<ID>),
}

pub(crate) struct Brain {
    genome: Vec<u8>,
    gene_step: usize,
    performance: u32,
    resources_amount: u8,
    bag: ExplorerBag,
    move_intention: bool,
    blocked: bool,
    idle_timeout : Duration,
    last_success : Instant,
}

impl Brain {
    pub(crate) fn new(game_step : Duration) -> Self {
        let mut rng = rand::rng();
        let genome: Vec<u8> = (0..64).map(|_| rng.random_range(0..128)).collect();
        //let genome = vec![59, 46, 53, 4, 0, 38, 9, 51, 61, 22, 25, 44, 12, 17, 0, 38, 37, 59, 32, 40];

        Self {
            genome,
            gene_step: 0,
            performance: 0,
            resources_amount: 0,
            bag: ExplorerBag::new(),
            move_intention: false,
            blocked: false,
            idle_timeout : game_step.mul(6),
            last_success: Instant::now(),
        }
    }
    #[allow(dead_code)]
    pub(crate) fn get_genome(&self) -> Vec<&u8> {
        self.genome.iter().collect()
    }
    pub(crate) fn get_performance(&self) -> u32 {
        self.performance
    }
    pub(crate) fn insert_resource(&mut self, resource: GenericResource) {
        self.performance += match resource {
            GenericResource::BasicResources(_) => {
                self.resources_amount += 1;
                BASIC_RESOURCE_WEIGHT
            }
            GenericResource::ComplexResources(_) => {
                self.resources_amount -= 1;
                COMPLEX_RESOURCE_WEIGHT
            }
        };
        self.bag.insert(resource);
        self.last_success = Instant::now();
    }
    pub(crate) fn reinsert_resource(&mut self, resource: GenericResource) {
        self.bag.insert(resource);
    }

    pub(crate) fn get_bag_content(&self) -> ExplorerBagContent {
        self.bag.to_content()
    }
    pub(crate) fn current_gene(&self) -> u8 {
        self.genome[self.gene_step % self.genome.len()]
    }
    pub(crate) fn try_combination_request(
        &mut self,
        complex: ComplexResourceType,
    ) -> Option<ComplexResourceRequest> {
        self.bag.create_combination_request(complex)
    }
    pub(crate) fn on_move(&mut self) {
        self.move_intention = false;
        self.last_success = Instant::now();
    }
    pub(crate) fn on_no_action(&mut self) {
        // Sets move intention to true based on the gene
        let gene = self.current_gene();
        if gene % 3 != 0 {
            self.move_intention = true;
        }
    }
    pub(crate) fn got_blocked(&mut self) {
        self.blocked = true;
    }
    pub(crate) fn unblock(&mut self){
        self.blocked = false;
    }
}

// Decisions
impl Brain {
    pub(crate) fn decide(&mut self, planet_stats: &mut PlanetStats) -> Intention {
        let gene = self.genome[self.gene_step % self.genome.len()];
        self.gene_step += 1;

        // If last action was not successful, the Explorer will want to move
        // Also better to move if too much time elapsed since last successful action
        if !self.blocked && (self.move_intention || (self.last_success.elapsed() > self.idle_timeout)) {
            return Intention::Move(Brain::decide_move(gene, planet_stats))
        }

        // Check if Planet has any combination matching our resources
        let matches = self.combinations_matchings(planet_stats.combinations());

        // Adding self.blocked prevents a Move Intention if Explorer is blocked here
        let mut action = gene % (10 + self.resources_amount) + u8::from(self.blocked);

        // If #matches is zero, don't try to combine
        // Otherwise make it more probable
        if action > 10 && !matches{
            action = (gene % 10) + u8::from(self.blocked);
        } else if matches {
            action += 6;
        }
        match action {
            0..2 => Intention::Move(Brain::decide_move(gene, planet_stats)),
            2..11 => Intention::Generate(Brain::decide_generation(gene, planet_stats)),
            _ => Intention::Combine(self.decide_combination(gene, planet_stats)),
        }
    }

    fn decide_generation(gene: u8, planet_stats: &mut PlanetStats) -> Option<BasicResourceType> {
        planet_stats.resources().and_then(|resources| {
            if resources.is_empty() {
                None
            } else {
                resources.iter().nth(gene as usize % resources.len()).copied()
            }
        })
    }
    fn decide_combination(
        &mut self,
        gene: u8,
        planet_stats: &mut PlanetStats,
    ) -> Option<ComplexResourceRequest> {
        let combo_vec: Vec<&ComplexResourceType> = planet_stats.combinations()?.iter().collect();
        if gene % 2 == 0 {
            combo_vec.into_iter().rev().find_map(|&complex| self.bag.create_combination_request(complex))
        } else {
            combo_vec.into_iter().find_map(|&complex| self.bag.create_combination_request(complex))
        }
    }
    fn decide_move(gene: u8, planet_stats: &mut PlanetStats) -> Option<ID> {
        planet_stats.neighbors().and_then(|neighbors| {
            if neighbors.is_empty() {
                None
            } else {
                Some(neighbors[gene as usize % neighbors.len()])
            }
        })
    }

    fn combinations_matchings(&self, combinations : Option<&HashSet<ComplexResourceType>>) -> bool {
        if let Some(combinations) = combinations {
            combinations.iter().any(|&complex| self.bag.can_craft(complex))
        } else {
            false
        }
    }
}
