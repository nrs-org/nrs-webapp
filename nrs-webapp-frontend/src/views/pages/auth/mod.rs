use hypertext::prelude::*;

pub mod confirm_email;
pub mod forgot_pass;
pub mod login;
pub mod register;

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
