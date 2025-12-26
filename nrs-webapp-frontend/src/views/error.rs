use axum::response::IntoResponse;
use axum_htmx::{HxPushUrl, HxRequest, HxReswap, SwapOption};
use heroicons::{
    Icon,
    icon_name::{ExclamationCircle, XMark},
    icon_variant::Solid,
};
use hypertext::prelude::*;

use crate::views::document::{Document, DocumentProps};

pub struct ClientError {
    pub title: String,
    pub description: String,
}

pub fn error_toast(error: &ClientError) -> impl Renderable {
    let fade_out_duration_ms = 300;
    rsx! {
        <div hx-swap-oob="afterbegin:#toast-root">
            <div role={"alert alert-error alert-vertical sm:alert-horizontal pointer-events-auto transition-opacity duration-"(fade_out_duration_ms)}>
                <Icon name=(ExclamationCircle) variant=(Solid) .. />

                <div>
                    <h3>(error.title)</h3>
                    <p>(error.description)</p>
                </div>


                <button class="btn btn-ghost" onclick={"
                    const a = this.closest('.alert');
                    a.classList.add('opacity-0');
                    setTimeout(() => a.remove(), "(fade_out_duration_ms)");
                "}>
                    <Icon name=(XMark) variant=(Solid) .. />
                </button>

                <div class="absolute bottom-0 left-0 h-1 bg-white/40">
                    <div class="h-full bg-white"
                         style="animation: toast-progress 4s linear forwards"></div>
                </div>
            </div>
        </div>
    }
}

pub fn error_page(error: &ClientError, props: &DocumentProps) -> impl Renderable {
    rsx! {
        <Document props=(DocumentProps {error: true, ..*props})>
            <main class="bg-base-200 flex flex-col items-center justify-center">
                <Icon class="size-30 text-error mb-2" name=(ExclamationCircle) variant=(Solid) .. />
                <h1 class="font-bold text-5xl mb-8">(error.title)</h1>
                <p class="text-base-content/80">(error.description)</p>
                <button onclick={"history.back()"} class="btn btn-primary mt-8">("Go Back")</button>
            </main>
        </Document>
    }
}

pub fn error(
    HxRequest(hx_req): HxRequest,
    props: &DocumentProps,
    error: &ClientError,
) -> impl IntoResponse {
    let html = rsx! {
        @if hx_req {
            (error_toast(error))
        } @else {
            (error_page(error, props))
        }
    };
    let reswap = hx_req.then_some(HxReswap(SwapOption::None));
    let push_url = hx_req.then_some(HxPushUrl(false.to_string()));

    (reswap, push_url, html)
}
