use std::{collections::HashSet, hash::Hash};

use common_explorer::ExplorerBag;
use common_game::components::resource::{BasicResourceType, ComplexResourceType, ResourceType};

use crate::vector::Vec10;

pub(crate) const BASIC_RESOURCE_WEIGHT: u32 = 10;
pub(crate) const COMPLEX_RESOURCE_WEIGHT: u32 = 60;

/// All basic resources, ordered.
pub const BASIC_RESOURCES: [BasicResourceType; 4] = [
    BasicResourceType::Carbon,
    BasicResourceType::Hydrogen,
    BasicResourceType::Oxygen,
    BasicResourceType::Silicon,
];

/// All complex resources, ordered.
pub const COMPLEX_RESOURCES: [ComplexResourceType; 6] = [
    ComplexResourceType::Diamond,
    ComplexResourceType::Water,
    ComplexResourceType::Life,
    ComplexResourceType::Robot,
    ComplexResourceType::Dolphin,
    ComplexResourceType::AIPartner,
];

/// All resources, ordered.
pub const RESOURCES: [ResourceType; 10] = [
    ResourceType::Basic(BasicResourceType::Carbon),
    ResourceType::Basic(BasicResourceType::Hydrogen),
    ResourceType::Basic(BasicResourceType::Oxygen),
    ResourceType::Basic(BasicResourceType::Silicon),
    ResourceType::Complex(ComplexResourceType::Diamond),
    ResourceType::Complex(ComplexResourceType::Water),
    ResourceType::Complex(ComplexResourceType::Life),
    ResourceType::Complex(ComplexResourceType::Robot),
    ResourceType::Complex(ComplexResourceType::Dolphin),
    ResourceType::Complex(ComplexResourceType::AIPartner),
];

/// Index for each resource type
pub fn resource_index(resource: ResourceType) -> usize {
    RESOURCES
        .iter()
        .position(|&r| r == resource)
        .expect("Resource not found in RESOURCES")
}

/// Index for each basic resource type
pub fn basic_resource_index(resource: BasicResourceType) -> usize {
    BASIC_RESOURCES
        .iter()
        .position(|&r| r == resource)
        .expect("Basic resource not found in BASIC_RESOURCES")
}

/// Index for each complex resource type
pub fn complex_resource_index(resource: ComplexResourceType) -> usize {
    COMPLEX_RESOURCES
        .iter()
        .position(|&r| r == resource)
        .expect("Complex resource not found in COMPLEX_RESOURCES")
}

/// Get ingredients for a complex resource
pub fn get_ingredients(resource: ComplexResourceType) -> Vec<ResourceType> {
    match resource {
        ComplexResourceType::Diamond => vec![
            ResourceType::Basic(BasicResourceType::Carbon),
            ResourceType::Basic(BasicResourceType::Carbon),
        ],
        ComplexResourceType::Water => vec![
            ResourceType::Basic(BasicResourceType::Hydrogen),
            ResourceType::Basic(BasicResourceType::Oxygen),
        ],
        ComplexResourceType::Life => vec![
            ResourceType::Basic(BasicResourceType::Carbon),
            ResourceType::Complex(ComplexResourceType::Water),
        ],
        ComplexResourceType::Robot => vec![
            ResourceType::Basic(BasicResourceType::Silicon),
            ResourceType::Complex(ComplexResourceType::Life),
        ],
        ComplexResourceType::Dolphin => vec![
            ResourceType::Complex(ComplexResourceType::Water),
            ResourceType::Complex(ComplexResourceType::Life),
        ],
        ComplexResourceType::AIPartner => vec![
            ResourceType::Complex(ComplexResourceType::Diamond),
            ResourceType::Complex(ComplexResourceType::Robot),
        ],
    }
}

/// Value for each resource
pub fn resource_value(resource: ResourceType) -> u64 {
    match resource {
        ResourceType::Basic(BasicResourceType::Carbon) => 1,
        ResourceType::Basic(BasicResourceType::Hydrogen) => 1,
        ResourceType::Basic(BasicResourceType::Oxygen) => 1,
        ResourceType::Basic(BasicResourceType::Silicon) => 1,
        ResourceType::Complex(ComplexResourceType::Diamond) => 10,
        ResourceType::Complex(ComplexResourceType::Water) => 5,
        ResourceType::Complex(ComplexResourceType::Life) => 20,
        ResourceType::Complex(ComplexResourceType::Robot) => 15,
        ResourceType::Complex(ComplexResourceType::Dolphin) => 25,
        ResourceType::Complex(ComplexResourceType::AIPartner) => 30,
    }
}

/// Helper to build Planet's capabilieties from resources
pub fn build_capabilities(
    basics: &HashSet<BasicResourceType>,
    complex: &HashSet<ComplexResourceType>,
) -> Vec10 {
    let mut capabilities = Vec10::new([0; 10]);
    capabilities.set_basic(basics);
    capabilities.set_complex(complex);
    capabilities
}

/// Helper to build the bag vector from explorer bag
pub fn build_bag_vector(bag: &ExplorerBag) -> Vec10 {
    let mut bag_vector = [0; 10];
    for (resource, count) in bag.to_content().resources_amounts.iter() {
        bag_vector[resource_index(*resource)] += *count;
    }
    Vec10::new(bag_vector)
}

/// Helper to build the crafting vector from explorer bag
pub fn build_crafting_vector(bag_vector: &Vec10) -> Vec10 {
    let b = bag_vector.get();
    let mut cv = [1; 10];

    cv[4] = (b[0] >= 2) as u64; // Diamond
    cv[5] = (b[1] > 0 && b[2] > 0) as u64; // Water
    cv[6] = (b[0] > 0 && b[5] > 0) as u64; // Life
    cv[7] = (b[3] > 0 && b[6] > 0) as u64; // Robot
    cv[8] = (b[5] > 0 && b[6] > 0) as u64; // Dolphin
    cv[9] = (b[4] > 0 && b[7] > 0) as u64; // AIPartner
    Vec10::new(cv)
}
