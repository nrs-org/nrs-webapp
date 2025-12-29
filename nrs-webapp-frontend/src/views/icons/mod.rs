use hypertext::{Raw, component};

macro_rules! include_raw {
    ($path:expr) => {
        hypertext::Raw::dangerously_create(std::include_str!($path))
    };
}

#[component]
pub fn github() -> Raw<&'static str> {
    include_raw!("github.svg")
}

#[component]
pub fn google() -> Raw<&'static str> {
    include_raw!("google.svg")
}
