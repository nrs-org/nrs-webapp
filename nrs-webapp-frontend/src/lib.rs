use axum::response::IntoResponse;
use axum_htmx::HxRequest;
use hypertext::prelude::*;

use crate::views::{
    components::toast::ToastComponent,
    document::{Document, DocumentProps},
};

pub mod views;

/// Conditionally wraps content in a full HTML document or returns partial (HX) content.
///
/// When the incoming request is an HTMX request, renders any toasts from `props.toasts` followed by
/// the provided `children`. For non-HX requests, renders a `Document` component with `props` and
/// places `children` inside it.
///
/// # Examples
///
/// ```
/// // HX request: only toasts and children are rendered
/// let hx = HxRequest(true);
/// let props = DocumentProps::default();
/// let fragment = maybe_document(hx, props, rsx! { "partial content" });
///
/// // Full request: document wrapper is rendered around children
/// let full = maybe_document(HxRequest(false), DocumentProps::default(), rsx! { "full page" });
/// ```
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
