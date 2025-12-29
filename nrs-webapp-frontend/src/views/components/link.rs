use axum::http::Method;
use hypertext::prelude::*;

#[derive(Default)]
pub struct LinkParams<'a> {
    pub href: &'a str,
    pub class: &'a str,
    pub hx_vals: &'a str,
    pub method: Method,
}

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
