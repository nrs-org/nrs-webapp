use axum::{http::StatusCode, response::IntoResponse};
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
    /// Convert a `ClientError` into a `Toast` that represents an error notification.
    ///
    /// The resulting `Toast` uses `ToastKind::Error`, takes the `title` from the error,
    /// and composes the `description` by appending the request UUID as an "Error ID".
    ///
    /// # Examples
    ///
    /// ```
    /// use nrs_webapp_frontend::views::error::ClientError;
    /// use nrs_webapp_frontend::views::components::toast::Toast;
    ///
    /// let err = ClientError {
    ///     title: "Failure".into(),
    ///     description: "Something went wrong".into(),
    ///     req_uuid: "abc-123".into(),
    /// };
    /// let toast: Toast = err.into();
    /// assert_eq!(toast.title, "Failure");
    /// assert!(toast.description.into_inner().contains("Error ID: abc-123"));
    /// ```
    fn from(value: ClientError) -> Self {
        Self {
            kind: ToastKind::Error,
            title: value.title,
            description: rsx! { (value.description)" Error ID: "(value.req_uuid) }.render(),
        }
    }
}

/// Render a full error page showing the provided ClientError message.
///
/// The returned renderable is a Document configured for an error state and
/// displays the error title, description, and a "Go Back" button.
///
/// # Examples
///
/// ```no_run
/// use nrs_webapp_frontend::views::error::{ClientError, error_page};
/// use nrs_webapp_frontend::views::document::DocumentProps;
///
/// let error = ClientError {
///     title: "Not Found".into(),
///     description: "The requested resource could not be found.".into(),
///     req_uuid: "req-123".into(),
/// };
/// let props = DocumentProps::default();
/// let _renderable = error_page(&error, &props);
/// ```
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

/// Render an error response, using a toast when the request is an HTMX request and a full error page otherwise.
///
/// The function returns HTMX response hints (optional `HxReswap` and `HxPushUrl`) together with the HTML fragment to send as the response body. When `hx_req` is true the returned HTML is a toast built from the provided `ClientError`; otherwise it is the full error page rendered with the provided `DocumentProps`.
///
/// # Returns
///
/// `(Option<HxReswap>, Option<HxPushUrl>, impl Renderable)` where the first two elements are HTMX hints to control swapping and URL push behavior, and the third element is the HTML fragment to include in the response.
///
/// # Examples
///
/// ```no_run
/// use nrs_webapp_frontend::views::error::{ClientError, error};
/// use nrs_webapp_frontend::views::document::DocumentProps;
/// use axum_htmx::HxRequest;
/// use hypertext::prelude::*;
/// use axum::http::StatusCode;
///
/// let client_error = ClientError {
///    title: "Server Error".into(),
///    description: "An unexpected error occurred.".into(),
///    req_uuid: "req-456".into(),
/// };
/// let props = DocumentProps::default();
///
/// // `hx_req`, `props`, and `client_error` are provided by the application context.
/// let _html = error(StatusCode::OK, HxRequest(false), &props, &client_error);
/// ```
pub fn error(
    code: StatusCode,
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

    (code, reswap, push_url, html)
}
