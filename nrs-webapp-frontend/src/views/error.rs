use axum::response::IntoResponse;
use axum_htmx::{HxPushUrl, HxRequest, HxReswap, SwapOption};
use heroicons::{Icon, icon_name::ExclamationCircle, icon_variant::Solid};
use hypertext::prelude::*;
use serde::Serialize;

use crate::views::{
    components::toast::{Toast, ToastKind, toast_component},
    document::{Document, DocumentProps},
};

#[derive(Debug, Clone, Serialize)]
pub struct ClientError {
    pub title: String,
    pub description: String,
    pub req_uuid: String,
}

impl From<ClientError> for Toast {
    fn from(value: ClientError) -> Self {
        Self {
            kind: ToastKind::Error,
            title: value.title,
            description: rsx! { (value.description)" Error ID: "(value.req_uuid) }.render(),
        }
    }
}

pub fn error_page(error: &ClientError, props: &DocumentProps) -> impl Renderable {
    rsx! {
        <Document props=(DocumentProps {error: true, toasts: vec![], ..props.clone()})>
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
            (toast_component(&error.clone().into()))
        } @else {
            (error_page(error, props))
        }
    };
    let reswap = hx_req.then_some(HxReswap(SwapOption::None));
    let push_url = hx_req.then_some(HxPushUrl(false.to_string()));

    (reswap, push_url, html)
}
