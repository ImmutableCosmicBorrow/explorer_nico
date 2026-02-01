use common_game::components::resource::{BasicResourceType, ComplexResourceType};
use common_game::protocols::planet_explorer::ExplorerToPlanet;
use common_game::utils::ID;
use crossbeam_channel::Sender;
use std::collections::HashSet;

pub(crate) struct PlanetStats {
    id: Option<ID>,
    sender: Option<Sender<ExplorerToPlanet>>,
    resources: Option<HashSet<BasicResourceType>>,
    combinations: Option<HashSet<ComplexResourceType>>,
    neighbors: Option<Vec<ID>>,
}

impl PlanetStats {
    pub(crate) fn new() -> Self {
        Self {
            id: None,
            sender: None,
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
    pub(crate) fn update_planet(&mut self, id: ID, sender: Sender<ExplorerToPlanet>) {
        self.id = Some(id);
        self.sender = Some(sender);
        self.resources = None;
        self.combinations = None;
        self.neighbors = None;
    }
    pub(crate) fn resources(&self) -> Option<&HashSet<BasicResourceType>> {
        self.resources.as_ref()
    }
    pub(crate) fn combinations(&self) -> Option<&HashSet<ComplexResourceType>> {
        self.combinations.as_ref()
    }
    pub(crate) fn neighbors(&self) -> Option<&Vec<ID>> {
        self.neighbors.as_ref()
    }
    pub(crate) fn id(&self) -> Option<ID> {
        self.id
    }
    pub(crate) fn has_neighbors(&self) -> bool {
        if let Some(ref neighbors) = self.neighbors {
            !neighbors.is_empty()
        } else {
            false
        }
    }
    pub(crate) fn has_resources(&self) -> bool {
        if let Some(ref resources) = self.resources {
            !resources.is_empty()
        } else {
            false
        }
    }
    pub(crate) fn has_combinations(&self) -> bool {
        if let Some(ref combinations) = self.combinations {
            !combinations.is_empty()
        } else {
            false
        }
    }
    pub(crate) fn sender(&self) -> Option<Sender<ExplorerToPlanet>> {
        self.sender.clone()
    }
    #[allow(dead_code)]
    pub(crate) fn remove_neighbor(&mut self, to_remove: Option<ID>) {
        if let Some(id) = to_remove
            && let Some(ref mut neighbors) = self.neighbors
        {
            neighbors.retain(|x| *x != id);
        }
    }
}
