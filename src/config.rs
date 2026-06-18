use common_game::components::resource::{BasicResourceType, ComplexResourceType, ResourceType};

pub(crate) const SOFTMAX_TEMPERATURE : f64 = 10.0;
pub(crate) const BASIC_RESOURCE_WEIGHT: u64 = 10;
pub(crate) const COMPLEX_RESOURCE_WEIGHT: u64 = 15;

pub(crate) const INITIAL_NEEDS: [u64; 10] = [
    3,  // Carbon
    3,  // Hydrogen
    3,  // Oxygen
    3,  // Silicon
    3,  // Diamond
    3,  // Water
    3,  // Life
    3,  // Robot
    4,  // Dolphin
    10, // AI Partner
];

pub(crate) const LAST_SUCCESS_TIMEOUT_MULTIPLIER: u32 = 6;

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