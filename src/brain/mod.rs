pub(crate) mod intention;
pub(crate) mod math;

use std::{
    collections::HashSet,
    time::{Duration, Instant},
};

use crate::brain::intention::Intention;
use crate::brain::math::ResourceVector;
use crate::config::{INITIAL_NEEDS, LAST_SUCCESS_TIMEOUT_MULTIPLIER, SOFTMAX_TEMPERATURE};
use crate::galaxy::galaxy_map::GalaxyMap;
use crate::galaxy::resources::build_bag_vector;
use crate::galaxy::resources::{build_crafting_vector, resource_value};
use common_explorer::{ExplorerBag, ExplorerBagContent};
use common_game::components::resource::ResourceType;
use common_game::{
    components::resource::{
        BasicResourceType, ComplexResourceRequest, ComplexResourceType, GenericResource,
    },
    utils::ID,
};

pub struct Brain {
    bag: ExplorerBag,
    needs: ResourceVector,
    galaxy_map: GalaxyMap,
    performance: u64,
    game_step: Duration,
    last_success: Instant,
}

impl Brain {
    /// Creates an empty Brain
    pub(crate) fn new(game_step: Duration) -> Self {
        Brain {
            bag: ExplorerBag::new(),
            needs: ResourceVector::new(INITIAL_NEEDS),
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
    pub(crate) fn performance(&self) -> u64 {
        self.performance
    }

    pub(crate) fn supported_resources(&mut self, planet : ID) -> HashSet<BasicResourceType> {
        self.galaxy_map.planet_supported_resources(planet)
    }

    pub(crate) fn supported_combinations(&mut self, planet : ID) -> HashSet<ComplexResourceType> {
        self.galaxy_map.planet_supported_combinations(planet)
    }

    /// Sets the needs `ResourceVector` to reflect the given `ResourceType`
    pub(crate) fn set_needs(&mut self, res: ResourceType) {
        self.needs = ResourceVector::generate_resource_needs(res);
    }

    /// Resets the needs `ResourceVector` to the default one.
    pub(crate) fn reset_needs(&mut self) {
        self.needs = ResourceVector::new(INITIAL_NEEDS);
    }

    /// Inserts a resource into the bag and updates the performance score and the needs
    pub(crate) fn insert_resource(&mut self, resource: GenericResource) {
        self.performance += resource_value(resource.get_type());
        // Update needs based on the inserted resource
        self.needs.decrease_need(&resource);
        // Update last success
        //self.last_success = Instant::now();
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

    /// Updates Planet basic resources in the `GalaxyMap`
    pub(crate) fn set_planet_basic_resources(
        &mut self,
        planet_id: ID,
        resources: HashSet<BasicResourceType>,
    ) {
        self.galaxy_map
            .set_planet_basic_resources(planet_id, resources);
    }

    /// Updates Planet complex resources in the `GalaxyMap`
    pub(crate) fn set_planet_complex_resources(
        &mut self,
        planet_id: ID,
        resources: HashSet<ComplexResourceType>,
    ) {
        self.galaxy_map
            .set_planet_complex_resources(planet_id, resources);
    }

    /// Updates Planet neighbors in the `GalaxyMap`
    pub(crate) fn set_planet_neighbors(&mut self, planet_id: ID, neighbors: Vec<ID>) {
        self.galaxy_map.set_planet_neighbors(planet_id, neighbors);
    }

    /// Resets last success on move
    pub(crate) fn on_move(&mut self) {
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

        let planet_capabilities = self.galaxy_map.planet_capabilities(current_planet);
        let bag_vector = build_bag_vector(&self.bag);
        let crafting_vector = build_crafting_vector(&bag_vector);

        let possibilities = self.needs * planet_capabilities * crafting_vector;

        if possibilities.is_zero() {
            return self.generate_move_intention(current_planet);
        }

        let choice = possibilities.softmax_sample(SOFTMAX_TEMPERATURE);

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
        let neighbors = self.galaxy_map.planet_neighbors(current_planet).clone();
        if neighbors.is_empty() {
            return Intention::Move(None);
        }

        let capabilities: Vec<(ID, ResourceVector)> = neighbors
            .iter()
            .map(|id| (*id, self.galaxy_map.planet_capabilities(*id)))
            .collect();

        if let Some(&(id, _)) = capabilities.iter().find(|(_, cap)| cap.is_zero()) {
            return Intention::Move(Some(id));
        }

        #[allow(clippy::cast_precision_loss)]
        let scores: Vec<(ID, f64)> = capabilities
            .iter()
            .map(|(id, cap)| (*id, cap.dot(&self.needs) as f64))
            .collect();

        // Softmax, avoiding Planets with score of 0
        let max = scores
            .iter()
            .map(|(_, s)| s)
            .copied()
            .fold(f64::NEG_INFINITY, f64::max);
        let exps: Vec<f64> = scores
            .iter()
            .map(|(_, s)| ((s - max) / SOFTMAX_TEMPERATURE).exp())
            .collect();
        let sum: f64 = exps.iter().sum();
        let probs: Vec<f64> = exps.iter().map(|e| e / sum).collect();

        // Sample an ID using the softmax distribution
        let mut r = rand::random::<f64>();
        let id = scores
            .iter()
            .zip(probs.iter())
            .find(|(_, p)| {
                r -= **p;
                r <= 0.0
            })
            .map_or(scores[0].0, |((id, _), _)| *id);

        Intention::Move(Some(id))
    }

    /// Checks if the last success was too long ago
    fn is_timed_out(&self) -> bool {
        self.last_success.elapsed() > self.game_step * LAST_SUCCESS_TIMEOUT_MULTIPLIER
    }
}
