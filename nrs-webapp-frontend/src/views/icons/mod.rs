use hypertext::{Raw, component};

macro_rules! include_raw {
    ($path:expr) => {
        hypertext::Raw::dangerously_create(std::include_str!($path))
    };
}

/// GitHub SVG icon provided as a `Raw<&'static str>` for inline rendering.
///
/// # Examples
///
/// ```
/// let _svg = github();
/// ```
#[component]
pub fn github() -> Raw<&'static str> {
    include_raw!("github.svg")
}

/// Returns a component that renders the bundled `google.svg` inline.
///
/// The component is a `Raw<&'static str>` containing the SVG markup embedded at compile time from `google.svg`.
///
/// # Examples
///
/// ```
/// let svg_component = google();
/// // `svg_component` contains the SVG markup from the bundled `google.svg`
/// ```
#[component]
pub fn google() -> Raw<&'static str> {
    include_raw!("google.svg")
}