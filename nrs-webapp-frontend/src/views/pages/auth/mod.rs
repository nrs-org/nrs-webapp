use hypertext::prelude::*;

pub mod confirm_email;
pub mod forgot_pass;
pub mod login;
pub mod register;

/// Renders a styled form section containing a fieldset, an optional legend, and provided children.
///
/// The `hx_post` parameter is treated as optional: when given as an empty string the `hx-post`
/// attribute is omitted; otherwise it is set to the provided value.
///
/// # Parameters
///
/// - `form_id` — identifier set on the `form` element and used to construct the `hx-disable` selector.
/// - `title` — legend text; when empty no `legend` element is rendered.
/// - `hx_post` — value for the `hx-post` attribute; use an empty string to omit the attribute.
/// - `children` — renderable content placed inside the fieldset.
///
/// # Examples
///
/// ```
/// # use dioxus_core::prelude::*;
/// rsx! {
///     form("login_form", "Sign in", "/auth/login", &rsx!{
///         input { r#type: "text", name: "username" }
///         input { r#type: "password", name: "password" }
///     })
/// }
/// ```
#[component]
fn form<'a, R: Renderable>(
    form_id: &'a str,
    title: &'a str,
    hx_post: &'a str,
    children: &R,
) -> impl Renderable {
    let hx_post = if hx_post.is_empty() {
        None
    } else {
        Some(hx_post)
    };
    rsx! {
        <section class="flex flex-col items-center">
            <form id=(form_id) hx-post=(hx_post) hx-target="#page" hx-swap="innerHTML" class="w-full max-w-lg" hx-disable={"#"(form_id)" fieldset"}>
                <fieldset class="fieldset bg-base-200 border-base-300 rounded-box border p-4 w-full mt-4">
                    @if !title.is_empty() {
                        <legend class="fieldset-legend">(title)</legend>
                    }
                    (children)
                </fieldset>
            </form>
        </section>
    }
}
