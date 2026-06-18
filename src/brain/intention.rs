use common_game::components::resource::{BasicResourceType, ComplexResourceRequest};
use common_game::utils::ID;

#[derive(Debug)]
pub(crate) enum Intention {
    Generate(Option<BasicResourceType>),
    Combine(Option<ComplexResourceRequest>),
    Move(Option<ID>),
}