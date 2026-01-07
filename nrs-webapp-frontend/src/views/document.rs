#[cfg(debug_assertions)]
use hypertext::Raw;
use hypertext::prelude::*;

use crate::views::components::{
    footer::Footer,
    navbar::Navbar,
    toast::{Toast, ToastComponent},
};

#[derive(Debug, Clone, Default)]
pub struct DocumentProps {
    pub error: bool,
    pub logged_in: bool,
    pub toasts: Vec<Toast>,
}

/// Renders the full HTML document shell for the application, including head assets, a toast container, navbar, page content, and footer.
///
/// The output sets `lang="en"`, applies the "winter" theme, and exposes an `data-is-error` attribute matching `props.error`. Toasts from `props.toasts` are rendered into a fixed toast root. In debug builds an embedded live-reload script is included; it is omitted in non-debug builds.
///
/// # Examples
///
/// ```
/// use nrs_webapp_frontend::views::document::{Document, DocumentProps};
/// use hypertext::prelude::*;
///
/// let _html = rsx! {
///    <Document props=(DocumentProps { error: false, logged_in: true, toasts: vec![] })>
///    // page content here
///    </Document>
/// };
/// ```
#[component]
pub fn document<R: Renderable>(props: &DocumentProps, children: &R) -> impl Renderable {
    // XSS SAFETY: this is only included in debug builds for live reloading
    // we dont put this script into the static dir to avoid shipping it to prod
    #[cfg(debug_assertions)]
    let live_reload_script =
        Raw::dangerously_create(include_str!("../inline_scripts/live-reload.js"));
    #[cfg(not(debug_assertions))]
    let live_reload_script = "";
    rsx! {
        <!DOCTYPE html>
        <html lang="en" data-theme="winter" data-is-error=(props.error)>
            <head>
                <title>"NRS Gaming"</title>
                <meta charset="UTF-8">
                <meta
                    name="viewport"
                    content="width=device-width, initial-scale=1.0"
                >
                <script src="/static/htmx.min.js"></script>
                <script src="/static/create-entry-form.js" type="module"></script>
                <script src="/static/toast-on-load.js" type="module" defer></script>
                <script src="/static/alpine.min.js" defer></script>
                <link rel="stylesheet" href="/static/generated/output.css">
                <script>
                    (live_reload_script)
                </script>
                <script src="/static/theme-controller.js" type="module"></script>
            </head>
            <body>
                <div
                    id="toast-root"
                    class={
                        "fixed top-4 left-1/2 -translate-x-1/2 z-50 flex flex-col space-y-3 pointer-events-none"
                        " w-full max-w-md sm:max-w-lg md:max-w-xl lg:max-w-3xl px-4"
                    }
                >
                    @for toast in &props.toasts {
                        <ToastComponent toast=(toast) />
                    }
                </div>
                <div class="min-h-[100dvh] grid grid-rows-[auto_1fr_auto]">
                    <Navbar logged_in=(props.logged_in) />
                    <main id="page" class="contents">
                        (children)
                    </main>
                    <Footer />
                </div>
            </body>
        </html>
    }
}
