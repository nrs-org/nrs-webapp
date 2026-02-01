use heroicons::{
    Icon,
    icon_name::{CheckCircle, ExclamationCircle, ExclamationTriangle, InformationCircle, XMark},
    icon_variant::Solid,
};
use hypertext::{Raw, prelude::*};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ToastKind {
    Info,
    Success,
    Warning,
    Error,
}

impl ToastKind {
    /// Get the CSS alert class name for the toast kind.
    ///
    /// # Returns
    ///
    /// `&'static str` CSS class name corresponding to the variant:
    /// - `ToastKind::Info` => `"alert-info"`
    /// - `ToastKind::Success` => `"alert-success"`
    /// - `ToastKind::Warning` => `"alert-warning"`
    /// - `ToastKind::Error` => `"alert-error"`
    ///
    /// # Examples
    ///
    /// ```
    /// use nrs_webapp_frontend::views::components::toast::ToastKind;
    /// let cls = ToastKind::Success.alert_class();
    /// assert_eq!(cls, "alert-success");
    /// ```
    pub fn alert_class(&self) -> &'static str {
        match self {
            ToastKind::Info => "alert-info",
            ToastKind::Success => "alert-success",
            ToastKind::Warning => "alert-warning",
            ToastKind::Error => "alert-error",
        }
    }
}

#[derive(Debug, Clone)]
pub struct Toast {
    pub title: String,
    pub description: Rendered<String>,
    pub kind: ToastKind,
}

/// Renders the appropriate solid heroicon for the given toast kind.
///
/// Icons selected:
/// - `ToastKind::Info` -> `InformationCircle`
/// - `ToastKind::Success` -> `CheckCircle`
/// - `ToastKind::Warning` -> `ExclamationTriangle`
/// - `ToastKind::Error` -> `ExclamationCircle`
#[component]
fn toast_icon(kind: ToastKind) -> impl Renderable {
    rsx! {
        @match kind {
            ToastKind::Info => <Icon class="size-6" name=(InformationCircle) variant=(Solid) .. />,
            ToastKind::Success => <Icon class="size-6" name=(CheckCircle) variant=(Solid) .. />,
            ToastKind::Warning => <Icon class="size-6" name=(ExclamationTriangle) variant=(Solid) .. />,
            ToastKind::Error => <Icon class="size-6" name=(ExclamationCircle) variant=(Solid) .. />,
        }
    }
}

/// Renders a dismissible toast notification with icon, title, description, auto-close behavior, and progress indicator.
///
/// The returned component produces a styled alert inserted into `#toast-root`, shows an icon based on the toast's `kind`,
/// and automatically closes after a short duration while also allowing manual dismissal via a close button. The toast's
/// description is rendered as trusted HTML.
///
/// # Returns
///
/// A renderable toast component that produces the described notification UI.
///
/// # Examples
///
/// ```
/// use nrs_webapp_frontend::views::components::toast::{Toast, ToastKind, ToastComponent};
/// use hypertext::prelude::*;
///
/// let toast = Toast {
///     title: "Saved".into(),
///     description: rsx! { <strong>"Your changes were saved."</strong> }.render(),
///     kind: ToastKind::Success,
/// };
///
/// // Render or embed the component into your view
/// let _component = rsx!{<ToastComponent toast=(&toast) />};
/// ```
#[component]
pub fn toast_component<'a>(toast: &'a Toast) -> impl Renderable {
    let fade_out_duration_ms = 300;
    // NOTE: keep in sync with CSS animation duration defined in input.css and toast-on-load.js
    let toast_autoclose_duration_ms = 10000;
    // SAFETY: description is rendered from trusted source
    let description = Raw::dangerously_create(toast.description.as_inner());
    rsx! {
        <div hx-swap-oob="afterbegin:#toast-root">
            <div class={
                "nrs-toast alert "(toast.kind.alert_class())" relative overflow-hidden alert-vertical sm:alert-horizontal
                 pointer-events-auto transition-opacity border-none duration-"(fade_out_duration_ms)}
                hx-on:htmx:after:process={"setTimeout(() => this.querySelector('.close-button')?.click(), "(toast_autoclose_duration_ms)")"}
            >
                <ToastIcon kind=(toast.kind) />

                <div>
                    <h3 class="font-bold">(toast.title)</h3>
                    <p class="text-xs">(description)</p>
                </div>


                <button class="btn btn-ghost close-button" onclick={"
                    const a = this.closest('.alert');
                    a.classList.add('opacity-0');
                    setTimeout(() => a.remove(), "(fade_out_duration_ms)");
                "}>
                    <Icon class="size-6" name=(XMark) variant=(Solid) .. />
                </button>

                <div class="absolute bottom-0 left-0 right-0 h-1 bg-white/40">
                    <div class="h-full bg-white/80 animate-toast-progress"></div>
                </div>
            </div>
        </div>
    }
}

impl Renderable for Toast {
    /// Renders this toast into the provided HTML buffer.
    ///
    /// The buffer will receive the toast's HTML representation so it can be inserted into the page.
    ///
    /// # Examples
    ///
    /// ```
    /// use nrs_webapp_frontend::views::components::toast::{Toast, ToastKind};
    /// use hypertext::{Rendered, Raw, Renderable, prelude::*};
    ///
    /// let toast = Toast {
    ///     title: "Saved".into(),
    ///     description: rsx! { <strong>"Your changes were saved."</strong> }.render(),
    ///     kind: ToastKind::Success,
    /// };
    /// let _toast_html = toast.render();
    /// ```
    fn render_to(&self, buffer: &mut hypertext::Buffer<hypertext::context::Node>) {
        toast_component(self).render_to(buffer);
    }
}
