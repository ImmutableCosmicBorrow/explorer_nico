use std::collections::{HashMap, HashSet};
use crate::brain::math::Vec10;
use common_game::{
    components::resource::{BasicResourceType, ComplexResourceType},
    utils::ID,
};

pub struct PlanetInfo {
    capabilities: Vec10,
    neighbors: Vec<ID>,
}

pub struct GalaxyMap {
    planets: HashMap<ID, PlanetInfo>,
}

impl PlanetInfo {
    pub(crate) fn new() -> Self {
        PlanetInfo {
            capabilities: Vec10::zeros(),
            neighbors: Vec::new(),
        }
    }

    pub(crate) fn capabilities(&self) -> Vec10 {
        self.capabilities
    }

    pub(crate) fn neighbors(&self) -> &Vec<ID> {
        &self.neighbors
    }

    pub(crate) fn set_basic_resources(&mut self, resources: &HashSet<BasicResourceType>) {
        self.capabilities.set_basic(resources);
    }

    pub(crate) fn set_complex_resources(&mut self, resources: &HashSet<ComplexResourceType>) {
        self.capabilities.set_complex(resources);
    }

    pub(crate) fn set_neighbors(&mut self, neighbors: Vec<ID>) {
        self.neighbors = neighbors;
    }
}

impl GalaxyMap {
    pub(crate) fn new() -> Self {
        GalaxyMap {
            planets: HashMap::new(),
        }
    }

    pub(crate) fn planet_capabilities(&mut self, planet_id: ID) -> Vec10 {
        self.planets
            .entry(planet_id)
            .or_insert(PlanetInfo::new())
            .capabilities()
    }

    pub(crate) fn planet_neighbors(&mut self, planet_id: ID) -> &Vec<ID> {
        self.planets
            .entry(planet_id)
            .or_insert(PlanetInfo::new())
            .neighbors()
    }

    pub(crate) fn set_planet_basic_resources(
        &mut self,
        planet_id: ID,
        resources: &HashSet<BasicResourceType>,
    ) {
        let planet = self.planets.entry(planet_id).or_insert(PlanetInfo::new());
        planet.set_basic_resources(resources);
    }

    pub(crate) fn set_planet_complex_resources(
        &mut self,
        planet_id: ID,
        resources: &HashSet<ComplexResourceType>,
    ) {
        let planet = self.planets.entry(planet_id).or_insert(PlanetInfo::new());
        planet.set_complex_resources(resources);
    }

    pub(crate) fn set_planet_neighbors(&mut self, planet_id: ID, neighbors: Vec<ID>) {
        let planet = self.planets.entry(planet_id).or_insert(PlanetInfo::new());
        planet.set_neighbors(neighbors);
    }
}
