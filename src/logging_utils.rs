use common_game::logging::{Channel, EventType, LogEvent, Payload};

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
    log_info,    Channel::Info;
    log_debug,   Channel::Debug;
    log_warning, Channel::Warning;
    log_error,   Channel::Error;
);

//log_trace,   Channel::Trace;