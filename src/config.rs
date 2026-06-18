use common_game::components::resource::{BasicResourceType, ComplexResourceType, ResourceType};

pub(crate) const SOFTMAX_TEMPERATURE: f64 = 10.0;
pub(crate) const BASIC_RESOURCE_WEIGHT: u64 = 10;
pub(crate) const COMPLEX_RESOURCE_WEIGHT: u64 = 15;

pub(crate) const INITIAL_NEEDS: [u64; 10] = [
    4,  // Carbon
    4,  // Hydrogen
    4,  // Oxygen
    4,  // Silicon
    5,  // Diamond
    5,  // Water
    5,  // Life
    5,  // Robot
    6,  // Dolphin
    10, // AI Partner
];

pub(crate) const NEEDS_MAGIC_NUMBER: u64 = 7;

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
