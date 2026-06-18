use common_explorer::ExplorerBag;
use common_game::components::resource::{BasicResourceType, ComplexResourceType, ResourceType};

use crate::vector::Vec10;

pub(crate) const BASIC_RESOURCE_WEIGHT: u64 = 10;
pub(crate) const COMPLEX_RESOURCE_WEIGHT: u64 = 15;

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

/// Value for each resource
pub fn resource_value(resource: ResourceType) -> u64 {
    match resource {
        ResourceType::Basic(BasicResourceType::Carbon | BasicResourceType::Hydrogen | BasicResourceType::Oxygen | BasicResourceType::Silicon) => BASIC_RESOURCE_WEIGHT,
        ResourceType::Complex(ComplexResourceType::Diamond | ComplexResourceType::Water | ComplexResourceType::Life | ComplexResourceType::Robot) => COMPLEX_RESOURCE_WEIGHT,
        ResourceType::Complex(ComplexResourceType::Dolphin | ComplexResourceType::AIPartner) => COMPLEX_RESOURCE_WEIGHT * 2,
    }
}

/// Helper to build the bag vector from explorer bag
pub fn build_bag_vector(bag: &ExplorerBag) -> Vec10 {
    let mut bag_vector = [0; 10];
    for (resource, count) in &bag.to_content().resources_amounts {
        bag_vector[resource_index(*resource)] += *count;
    }
    Vec10::new(bag_vector)
}

/// Helper to build the crafting vector from explorer bag
pub fn build_crafting_vector(bag_vector: &Vec10) -> Vec10 {
    let b = bag_vector.get();
    let mut cv = [1; 10];

    cv[4] = u64::from(b[0] >= 2); // Diamond
    cv[5] = u64::from(b[1] > 0 && b[2] > 0); // Water
    cv[6] = u64::from(b[0] > 0 && b[5] > 0); // Life
    cv[7] = u64::from(b[3] > 0 && b[6] > 0); // Robot
    cv[8] = u64::from(b[5] > 0 && b[6] > 0); // Dolphin
    cv[9] = u64::from(b[4] > 0 && b[7] > 0); // AIPartner
    Vec10::new(cv)
}
