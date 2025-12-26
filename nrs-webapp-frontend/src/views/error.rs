use axum::response::IntoResponse;
use axum_htmx::{HxPushUrl, HxRequest, HxReswap, SwapOption};
use heroicons::{
    Icon,
    icon_name::{ExclamationCircle, XMark},
    icon_variant::Solid,
};
use hypertext::prelude::*;
use serde::Serialize;

use crate::views::document::{Document, DocumentProps};

#[derive(Debug, Serialize)]
pub struct ClientError {
    pub title: String,
    pub description: String,
    pub req_uuid: String,
}

pub fn error_toast(error: &ClientError) -> impl Renderable {
    let fade_out_duration_ms = 300;
    // NOTE: keep in sync with CSS animation duration defined in input.css
    let toast_autoclose_duration_ms = 4000;
    rsx! {
        <div hx-swap-oob="afterbegin:#toast-root">
            <div class={
                "alert alert-error relative overflow-hidden alert-vertical sm:alert-horizontal
                 pointer-events-auto transition-opacity border-none duration-"(fade_out_duration_ms)}
                hx-on:htmx:after:process={"setTimeout(() => this.querySelector('.close-button')?.click(), "(toast_autoclose_duration_ms)")"}
            >
                <Icon class="size-6" name=(ExclamationCircle) variant=(Solid) .. />

                <div>
                    <h3 class="font-bold">(error.title)</h3>
                    <p class="text-xs">(error.description)" Error ID: "(error.req_uuid)</p>
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

pub fn error_page(error: &ClientError, props: &DocumentProps) -> impl Renderable {
    rsx! {
        <Document props=(DocumentProps {error: true, ..*props})>
            <section class="bg-base-200 flex flex-col items-center justify-center">
                <Icon class="size-30 text-error mb-2" name=(ExclamationCircle) variant=(Solid) .. />
                <h1 class="font-bold text-5xl mb-8">(error.title)</h1>
                <p class="text-base-content/80">(error.description)</p>
                <button onclick={"history.back()"} class="btn btn-primary mt-8">("Go Back")</button>
            </section>
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
