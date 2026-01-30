use crate::planet_stats::PlanetStats;
use common_explorer::{ExplorerBag, ExplorerBagContent};
use common_game::components::resource::{
    BasicResourceType, ComplexResourceRequest, ComplexResourceType, GenericResource,
};
use common_game::utils::ID;
use rand::RngExt;

const BASIC_RESOURCE_WEIGHT: u32 = 10;
const COMPLEX_RESOURCE_WEIGHT: u32 = 60;

#[derive(Debug)]
pub(crate) enum Intention {
    Generate(Option<BasicResourceType>),
    Combine(Option<ComplexResourceType>),
    Move(Option<ID>),
}

pub(crate) struct Brain {
    genome: Vec<u8>,
    gene_step: usize,
    performance: u32,
    move_chance_increment: u8,
    bag: ExplorerBag,
}

impl Brain {
    pub(crate) fn new() -> Self {
        let mut rng = rand::rng();
        let genome: Vec<u8> = (0..20).map(|_| rng.random_range(0..32)).collect();
        //let genome = vec![11, 9, 11, 8, 2, 28, 2, 14, 24, 6, 8, 26, 31, 2, 9, 23, 24, 29, 0, 16];

        Self {
            genome,
            gene_step: 0,
            performance: 0,
            move_chance_increment: 0,
            bag: ExplorerBag::new(),
        }
    }
    pub(crate) fn get_genome(&self) -> Vec<&u8> {
        self.genome.iter().collect()
    }
    pub(crate) fn get_performance(&self) -> u32 {
        self.performance
    }

    pub(crate) fn insert_resource(&mut self, resource: GenericResource) {
        self.performance += match resource {
            GenericResource::BasicResources(_) => BASIC_RESOURCE_WEIGHT,
            GenericResource::ComplexResources(_) => COMPLEX_RESOURCE_WEIGHT,
        };
        if self.move_chance_increment > 0 {
            self.move_chance_increment -= 1;
        }
        self.bag.insert(resource);
    }
    pub(crate) fn reinsert_resource(&mut self, resource: GenericResource) {
        self.bag.insert(resource);
    }

    pub(crate) fn get_bag_content(&self) -> ExplorerBagContent {
        self.bag.to_content()
    }
    pub(crate) fn try_combination_request(
        &mut self,
        complex: ComplexResourceType,
    ) -> Option<ComplexResourceRequest> {
        self.bag.create_combination_request(complex)
    }
    pub(crate) fn on_move(&mut self) {
        self.move_chance_increment = 0;
    }
    pub(crate) fn on_no_action(&mut self) {
        self.move_chance_increment += 1;
    }
}

// Decisions
impl Brain {
    pub(crate) fn decide(&mut self, planet_stats: &mut PlanetStats) -> Intention {
        let gene = self.genome[self.gene_step % self.genome.len()];
        self.gene_step += 1;

        match gene % (10 + (self.move_chance_increment)) {
            0..5 => Intention::Generate(Brain::decide_generation(gene, planet_stats)),
            5..8 => Intention::Combine(Brain::decide_combination(gene, planet_stats)),
            _ => Intention::Move(self.decide_move(gene, planet_stats)),
        }
    }

    fn decide_generation(gene: u8, planet_stats: &mut PlanetStats) -> Option<BasicResourceType> {
        planet_stats
            .resources()
            .filter(|resources| !resources.is_empty())
            .map(|resources| {
                *resources.iter().collect::<Vec<&BasicResourceType>>()
                    [gene as usize % resources.len()]
            })
    }
    fn decide_combination(gene: u8, planet_stats: &mut PlanetStats) -> Option<ComplexResourceType> {
        planet_stats
            .combinations()
            .filter(|resources| !resources.is_empty())
            .map(|resources| {
                *resources.iter().collect::<Vec<&ComplexResourceType>>()
                    [gene as usize % resources.len()]
            })
    }
    fn decide_move(&mut self, gene: u8, planet_stats: &mut PlanetStats) -> Option<ID> {
        let id = planet_stats
            .neighbors()
            .filter(|neighbors| !neighbors.is_empty())
            .map(|neighbors| neighbors[gene as usize % neighbors.len()]);
        planet_stats.remove_neighbor(id);
        if id.is_some() {
            self.move_chance_increment = 0;
        }
        id
    }
}
