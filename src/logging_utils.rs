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
