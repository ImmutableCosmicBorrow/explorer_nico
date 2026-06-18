use common_game::logging::{ActorType, Channel, EventType, LogEvent, Participant, Payload};
use common_game::utils::ID;

#[macro_export]
/// A macro to create a payload, given a list of `(key, msg)` pairs.
macro_rules! payload {
    ($($key:ident : $val:expr),* $(,)?) => {{
        let mut p = common_game::logging::Payload::new();
        $(
            p.insert(stringify!($key).to_string(), format!{"{}", $val});
        )*
        p
    }};
}
/// A macro to generate different logging functions, that log into different Channels.
macro_rules! generate_logs {
    ($($name:ident, $channel:expr);+ $(;)?) => {
        $(
            pub(crate) fn $name(p: Payload) {
                LogEvent::system(
                    EventType::InternalExplorerAction,
                    $channel,
                    p
                ).emit();
            }
        )+
    };
}
generate_logs!(
    log_debug,   Channel::Debug;
    log_warning, Channel::Warning;
    log_error,   Channel::Error;
    log_trace,   Channel::Trace;
);

pub(crate) fn log_to_orchestrator(explorer_id: ID, p: Payload) {
    LogEvent::new(
        Some(Participant::new(ActorType::Explorer, explorer_id)),
        Some(Participant::new(ActorType::Orchestrator, 0u32)),
        EventType::MessageExplorerToOrchestrator,
        Channel::Trace,
        p,
    )
        .emit();
}

pub(crate) fn log_to_planet(explorer_id: ID, planet_id: ID, p: Payload) {
    LogEvent::new(
        Some(Participant::new(ActorType::Explorer, explorer_id)),
        Some(Participant::new(ActorType::Planet, planet_id)),
        EventType::MessageExplorerToPlanet,
        Channel::Trace,
        p,
    )
        .emit();
}
