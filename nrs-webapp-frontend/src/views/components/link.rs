use axum::http::Method;
use hypertext::prelude::*;

#[derive(Default, Clone)]
pub struct LinkParams<'a> {
    pub href: &'a str,
    pub class: &'a str,
    pub hx_vals: &'a str,
    pub method: Method,
}

/// Renders an anchor (<a>) configured for htmx-driven navigation.
///
/// The anchor uses `params.class` for CSS classes, `params.hx_vals` for htmx values,
/// and targets `#page` with `hx-swap="innerHTML"` and `hx-push-url=true`. It sets
/// `hx-get` to `params.href` when `params.method` is `Method::GET`, or `hx-post` to
/// `params.href` when `params.method` is `Method::POST`.
///
/// # Examples
///
/// ```
/// use axum::http::Method;
/// use nrs_webapp_frontend::views::components::link::{Link, LinkParams};
/// use hypertext::prelude::*;
///
/// let _node = rsx! {<Link params=(LinkParams { href: "/items", class: "nav-link", hx_vals: "{}", method: Method::GET })>"Open items"</Link>};
/// ```
#[component]
pub fn link<'a, R: Renderable>(children: &R, params: &LinkParams<'a>) -> impl Renderable {
    // TODO: add more if needed
    let hx_get = (params.method == Method::GET).then_some(params.href);
    let hx_post = (params.method == Method::POST).then_some(params.href);
    rsx! {
        <a role="link" class=(params.class) hx-get=(hx_get) hx-post=(hx_post) hx-target="#page" hx-swap="innerHTML" hx-push-url=true hx-vals=(params.hx_vals)>
            (children)
        </a>
    }
}
