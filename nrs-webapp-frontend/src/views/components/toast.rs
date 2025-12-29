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

#[component]
pub fn toast_component<'a>(toast: &'a Toast) -> impl Renderable {
    let fade_out_duration_ms = 300;
    // NOTE: keep in sync with CSS animation duration defined in input.css and toast-on-load.js
    let toast_autoclose_duration_ms = 4000;
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
    fn render_to(&self, buffer: &mut hypertext::Buffer<hypertext::context::Node>) {
        toast_component(self).render_to(buffer);
    }
}
