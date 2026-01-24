use crate::Explorer;
use common_game::logging::{ActorType, Channel, EventType, LogEvent, Participant, Payload};
use common_game::utils::ID;

#[macro_export]
macro_rules! payload {
    ($($key:ident : $val:expr),* $(,)?) => {{
        let mut p = common_game::logging::Payload::new();
        $(
            p.insert(stringify!($key).to_string(), format!{"{}", $val});
        )*
        p
    }};
}

impl Explorer {
    /// Creates and emits a log event without sender and receiver, and with `EventType::InternalOrchestratorAction`
    pub fn log_internal(channel: Channel, payload: Payload) {
        LogEvent::system(EventType::InternalExplorerAction, channel, payload).emit();
    }

    /// Creates a log event with itself as sender
    pub fn log_msg_to(
        &self,
        channel: Channel,
        event_type: EventType,
        to: (ActorType, ID),
        payload: Payload,
    ) {
        LogEvent::new(
            Some(Participant::new(ActorType::Explorer, self.id)),
            Some(Participant::new(to.0, to.1)),
            event_type,
            channel,
            payload,
        )
        .emit();
    }
}
