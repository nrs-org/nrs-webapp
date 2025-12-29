use axum::response::IntoResponse;
use axum_htmx::HxRequest;
use hypertext::prelude::*;

use crate::views::{
    components::toast::ToastComponent,
    document::{Document, DocumentProps},
};

pub mod views;

pub fn maybe_document<R: Renderable>(
    HxRequest(is_hx_request): HxRequest,
    props: DocumentProps,
    children: R,
) -> impl IntoResponse + Renderable {
    rsx! {
        @if is_hx_request {
            @for toast in props.toasts.iter() {
                <ToastComponent toast=(toast) />
            }
            (children)
        } @else {
            <Document props=(props.clone())>
                (children)
            </Document>
        }
    }
}
