use common_game::components::resource::{BasicResourceType, ComplexResourceType};
use common_game::utils::ID;

#[derive(Debug)]
pub(crate) enum Intention {
    Generate(Option<BasicResourceType>),
    Combine(Option<ComplexResourceType>),
    Move(Option<ID>),
}
