use std::{
    collections::HashSet,
    time::{Duration, Instant},
};

use crate::{galaxy_map::GalaxyMap, payload, resources::{BASIC_RESOURCE_WEIGHT, COMPLEX_RESOURCE_WEIGHT, build_bag_vector}, vector::Vec10};
use common_explorer::{ExplorerBag, ExplorerBagContent};
use common_game::{
    components::resource::{
        BasicResourceType, ComplexResourceRequest, ComplexResourceType, GenericResource,
    },
    utils::ID,
};
use crate::logging_utils::log_error;
use crate::resources::build_crafting_vector;

#[derive(Debug)]
pub(crate) enum Intention {
    Generate(Option<BasicResourceType>),
    Combine(Option<ComplexResourceRequest>),
    Move(Option<ID>),
}

const INITIAL_NEEDS: [u64; 10] = [
    4,  // Carbon
    4,  // Hydrogen
    4,  // Oxygen
    4,  // Silicon
    5,  // Diamond
    6,  // Water
    7,  // Life
    8,  // Robot
    9,  // Dolphin
    10, // AI Partner
];

const LAST_SUCCESS_TIMEOUT_MULTIPLIER: u32 = 3;

pub struct Brain {
    bag: ExplorerBag,
    needs: Vec10,
    galaxy_map: GalaxyMap,
    performance: u32,
    game_step: Duration,
    last_success: Instant,
}

impl Brain {
    /// Creates an empty Brain
    pub(crate) fn new(game_step: Duration) -> Self {
        Brain {
            bag: ExplorerBag::new(),
            needs: Vec10::new(INITIAL_NEEDS),
            galaxy_map: GalaxyMap::new(),
            performance: 0,
            game_step,
            last_success: Instant::now(),
        }
    }

    /// Returns the content of the bag
    pub(crate) fn bag_content(&self) -> ExplorerBagContent {
        self.bag.to_content()
    }

    /// Returns the performance score
    pub(crate) fn performance(&self) -> u32 {
        self.performance
    }

    /// Inserts a resource into the bag and updates the performance score and the needs
    pub(crate) fn insert_resource(&mut self, resource: GenericResource) {
        self.performance += match resource {
            GenericResource::BasicResources(_) => BASIC_RESOURCE_WEIGHT,
            GenericResource::ComplexResources(_) => COMPLEX_RESOURCE_WEIGHT,
        };
        // Update needs based on the inserted resource
        self.needs.decrease_need(&resource);
        // Update last success
        self.last_success = Instant::now();

        self.bag.insert(resource);
    }

    /// Reinserts a resource into the bag without updating the performance score, but updates the needs
    pub(crate) fn reinsert_resource(&mut self, resource: GenericResource) {
        // Update needs based on the inserted resource
        self.needs.decrease_need(&resource);

        self.bag.insert(resource);
    }

    /// Tries go create a combination request for a complex resource, also increases the needs for the resources used in the combination
    pub(crate) fn try_combination_request(
        &mut self,
        complex: ComplexResourceType,
    ) -> Option<ComplexResourceRequest> {
        let request = self.bag.create_combination_request(complex);
        if let Some(r) = &request {
            // Update needs based on the resources used in the combination
            self.needs.increase_needs(r);
        }
        request
    }

    /// Updates Planet basic resources in the GalaxyMap
    pub(crate) fn set_planet_basic_resources(
        &mut self,
        planet_id: ID,
        resources: HashSet<BasicResourceType>,
    ) {
        self.galaxy_map
            .set_planet_basic_resources(planet_id, resources);
    }

    /// Updates Planet complex resources in the GalaxyMap
    pub(crate) fn set_planet_complex_resources(
        &mut self,
        planet_id: ID,
        resources: HashSet<ComplexResourceType>,
    ) {
        self.galaxy_map
            .set_planet_complex_resources(planet_id, resources);
    }

    /// Updates Planet neighbors in the GalaxyMap
    pub(crate) fn set_planet_neighbors(&mut self, planet_id: ID, neighbors: Vec<ID>) {
        self.galaxy_map.set_planet_neighbors(planet_id, neighbors);
    }

    /// Resets last success on move
    pub(crate) fn on_move(&mut self, id: ID) {
        self.last_success = Instant::now();
    }

    /// Thinks at what action to perform
    pub(crate) fn think(&mut self, current_planet: ID) -> Intention {
        // TODO: does something like:
        // 1. [needs] * [planet.capabilities] * [combinations] DONE
        // 2. Picks the max DONE
        // 3. Creates the Intention to perform it DONE
        // There should be a chance that we decide to move instead
        // Might model last success, and when we think we check when last success happened
        // If it is too long ago, we might want to move instead
        // Must also update needs

        if self.is_timed_out() {
            return self.generate_move_intention(current_planet);
        }

        let planet_capabilies = self.galaxy_map.planet_capabilities(current_planet);
        let bag_vector = build_bag_vector(&self.bag);
        let crafting_vector = build_crafting_vector(&bag_vector);

        let possibilites = self.needs * planet_capabilies * crafting_vector;
        let choice = possibilites.max_index();

        self.generate_intention(choice)
    }

    /// Generates the Intention given the choice
    fn generate_intention(&mut self, choice: usize) -> Intention {
        match choice {
            0 => Intention::Generate(Some(BasicResourceType::Carbon)),
            1 => Intention::Generate(Some(BasicResourceType::Hydrogen)),
            2 => Intention::Generate(Some(BasicResourceType::Oxygen)),
            3 => Intention::Generate(Some(BasicResourceType::Silicon)),
            4 => Intention::Combine(self.try_combination_request(ComplexResourceType::Diamond)),
            5 => Intention::Combine(self.try_combination_request(ComplexResourceType::Water)),
            6 => Intention::Combine(self.try_combination_request(ComplexResourceType::Life)),
            7 => Intention::Combine(self.try_combination_request(ComplexResourceType::Robot)),
            8 => Intention::Combine(self.try_combination_request(ComplexResourceType::Dolphin)),
            9 => Intention::Combine(self.try_combination_request(ComplexResourceType::AIPartner)),
            _ => Intention::Generate(None), // TODO: placeholder, change to Move probably
        }
    }

    /// Generates a Move Intention by choosing where to move
    fn generate_move_intention(&mut self, current_planet: ID) -> Intention {
        let neighbors = self.galaxy_map.planet_neighbors(current_planet);
        if neighbors.is_empty() {
            return Intention::Move(None);
        }
        let next_planet = neighbors[0]; // TODO: choose a random neighbor
        Intention::Move(Some(next_planet))
    }

    /// Checks if the last success was too long ago
    fn is_timed_out(&self) -> bool {
        self.last_success.elapsed() > self.game_step * LAST_SUCCESS_TIMEOUT_MULTIPLIER
    }
}
