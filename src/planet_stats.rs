use common_game::components::resource::{BasicResourceType, ComplexResourceType};
use common_game::utils::ID;
use std::collections::HashSet;

pub(crate) struct PlanetStats {
    resources: Option<HashSet<BasicResourceType>>,
    combinations: Option<HashSet<ComplexResourceType>>,
    neighbors: Option<Vec<ID>>,
}

impl PlanetStats {
    pub(crate) fn new() -> Self {
        Self {
            resources: None,
            combinations: None,
            neighbors: None,
        }
    }
    pub(crate) fn update_resources(&mut self, resources: HashSet<BasicResourceType>) {
        self.resources = Some(resources);
    }
    pub(crate) fn update_combinations(&mut self, combinations: HashSet<ComplexResourceType>) {
        self.combinations = Some(combinations);
    }
    pub(crate) fn update_neighbors(&mut self, neighbors: Vec<ID>) {
        self.neighbors = Some(neighbors);
    }
    pub(crate) fn get_resources(&self) -> Option<&HashSet<BasicResourceType>> {
        self.resources.as_ref()
    }
    pub(crate) fn get_combinations(&self) -> Option<&HashSet<ComplexResourceType>> {
        self.combinations.as_ref()
    }
    pub(crate) fn get_neighbors(&self) -> Option<&Vec<ID>> {
        self.neighbors.as_ref()
    }
}
