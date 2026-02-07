use axum::{
    Form,
    extract::{
        FromRef, FromRequest, FromRequestParts, Query, Request,
        rejection::{FormRejection, JsonRejection, QueryRejection},
    },
    http::request::Parts,
    response::IntoResponse,
};
use thiserror::Error;
use validator::{Validate, ValidateArgs, ValidationErrors};

pub struct WithRejection<T>(pub T);

pub struct WRForm<T>(pub T);
pub struct WRQuery<T>(pub T);

impl<T, S> FromRequest<S> for WRForm<T>
where
    S: Send + Sync,
    WithRejection<Form<T>>: FromRequest<S>,
{
    type Rejection = <WithRejection<Form<T>> as FromRequest<S>>::Rejection;

    /// Extract a form-encoded payload from the request and return it wrapped in `WRForm`.
    ///
    /// On success returns `WRForm` containing the parsed form value. On failure returns the extractor's rejection.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use axum::http::Request;
    /// // Assume `MyForm` implements `serde::Deserialize` and the necessary extractor traits are in scope.
    /// // let req: Request = Request::builder()...; // a request with form body
    /// // let state = ...; // your application state reference
    /// // let wrapped = WRForm::<MyForm>::from_request(req, &state).await.unwrap();
    /// // let form: MyForm = wrapped.0;
    /// ```
    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        let WithRejection(Form(data)) = WithRejection::from_request(req, state).await?;
        Ok(Self(data))
    }
}

impl<T, S> FromRequestParts<S> for WRQuery<T>
where
    S: Send + Sync,
    WithRejection<Query<T>>: FromRequestParts<S>,
{
    type Rejection = <WithRejection<Query<T>> as FromRequestParts<S>>::Rejection;

    /// Extracts a query payload from request parts and returns it wrapped in `WRQuery`.
    ///
    /// # Examples
    ///
    /// ```
    /// use axum::extract::FromRequestParts;
    /// use nrs_webapp::extract::with_rejection::WRQuery;
    ///
    /// struct MyQuery { /* fields omitted */ }
    ///
    /// async fn handler(parts: &mut axum::http::request::Parts, state: &()) -> Result<(), ()> {
    ///     let WRQuery(query): WRQuery<MyQuery> = WRQuery::from_request_parts(parts, state).await?;
    ///     // use `query`
    ///     Ok(())
    /// }
    /// ```
    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let WithRejection(Query(data)) = WithRejection::from_request_parts(parts, state).await?;
        Ok(Self(data))
    }
}

pub struct WRVForm<T>(pub T);
pub struct WRVQuery<T>(pub T);
pub struct WRVexForm<T>(pub T);
pub struct WRVexQuery<T>(pub T);

impl<T, S> FromRequest<S> for WRVForm<T>
where
    S: Send + Sync,
    WRForm<T>: FromRequest<S>,
    T: Validate,
    <WRForm<T> as FromRequest<S>>::Rejection: From<ValidationErrors>,
{
    type Rejection = <WRForm<T> as FromRequest<S>>::Rejection;

    /// Extracts a form from the request, validates it, and returns the validated wrapper.
    ///
    /// On success returns `Self` containing the validated form data. On failure returns the
    /// extractor's rejection or `ValidationErrors` produced by validation.
    ///
    /// # Examples
    ///
    /// ```
    /// # async fn doc_example() {
    /// use axum::http::Request;
    /// // assume `MyForm` implements `validator::Validate`
    /// // and `WRVForm<MyForm>` is the validated wrapper produced by this extractor.
    /// async fn handler(req: Request<axum::body::Body>, state: &crate::AppState) {
    ///     let wrv = WRVForm::<MyForm>::from_request(req, state).await.unwrap();
    ///     let validated: MyForm = wrv.0;
    /// }
    /// # }
    /// ```
    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        let WRForm(data) = WRForm::from_request(req, state).await?;
        data.validate()?;
        Ok(Self(data))
    }
}

impl<T, S> FromRequest<S> for WRVexForm<T>
where
    S: Send + Sync,
    WRForm<T>: FromRequest<S>,
    T: for<'args> ValidateArgs<'args>,
    for<'args> <T as ValidateArgs<'args>>::Args: FromRef<S> + Send + Sync,
    <WRForm<T> as FromRequest<S>>::Rejection: From<ValidationErrors>,
{
    type Rejection = <WRForm<T> as FromRequest<S>>::Rejection;

    /// Extracts a form from the request, validates it with state-derived arguments, and returns the validated wrapper.
    ///
    /// The implementation delegates extraction to `WRForm<T>`, then calls `validate_with_args` on the inner value
    /// using `FromRef::from_ref(state)` to obtain validation arguments. On success returns `WRVexForm<T>` containing
    /// the validated value. If extraction or validation fails, the extractor's rejection is returned.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use axum::Request;
    /// # use core::future::Future;
    /// // Assuming `WRVexForm<MyForm>` is in scope and `state` implements `FromRef` for the form's Args:
    /// async fn handler<S>(req: Request, state: &S) {
    ///     let validated = WRVexForm::<MyForm>::from_request(req, state).await;
    ///     // `validated` is `Ok(WRVexForm(MyForm))` on success or an extractor rejection on failure.
    /// }
    /// ```
    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        let WRForm(data) = WRForm::from_request(req, state).await?;
        data.validate_with_args(FromRef::from_ref(state))?;
        Ok(Self(data))
    }
}

impl<T, S> FromRequestParts<S> for WRVQuery<T>
where
    S: Send + Sync,
    WRQuery<T>: FromRequestParts<S>,
    T: Validate,
    <WRQuery<T> as FromRequestParts<S>>::Rejection: From<ValidationErrors>,
{
    type Rejection = <WRQuery<T> as FromRequestParts<S>>::Rejection;

    /// Extracts query parameters from request parts and validates them, returning a `WRVQuery` with the validated value.
    ///
    /// The extractor obtains a `WRQuery<T>` from `parts` and invokes `T::validate()`. Validation failures are returned as the extractor's rejection.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// // In an async extractor context:
    /// // let validated: WRVQuery<MyQuery> = WRVQuery::from_request_parts(&mut parts, &state).await?;
    /// ```
    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let WRQuery(data) = WRQuery::from_request_parts(parts, state).await?;
        data.validate()?;
        Ok(Self(data))
    }
}

impl<T, S> FromRequestParts<S> for WRVexQuery<T>
where
    S: Send + Sync,
    WRQuery<T>: FromRequestParts<S>,
    T: for<'args> ValidateArgs<'args>,
    for<'args> <T as ValidateArgs<'args>>::Args: FromRef<S> + Send + Sync,
    <WRQuery<T> as FromRequestParts<S>>::Rejection: From<ValidationErrors>,
{
    type Rejection = <WRQuery<T> as FromRequestParts<S>>::Rejection;

    /// Extracts a query value from request parts and validates it using arguments derived from `state`.
    ///
    /// On success returns a `WRVexQuery<T>` containing the validated value. On failure returns the extractor's rejection (extraction or validation error).
    ///
    /// # Examples
    ///
    /// ```
    /// use axum::http::request::Parts;
    /// # async fn example<S>(_parts: &mut Parts, _state: &S) {}
    /// // Called by the framework in practice; shown here for illustration:
    /// // let validated = WRVexQuery::<MyQuery>::from_request_parts(&mut parts, &state).await?;
    /// ```
    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let WRQuery(data) = WRQuery::from_request_parts(parts, state).await?;
        data.validate_with_args(FromRef::from_ref(state))?;
        Ok(Self(data))
    }
}

#[derive(Debug, Error)]
pub enum RejectionError {
    #[error(transparent)]
    Form(#[from] FormRejection),

    #[error(transparent)]
    Json(#[from] JsonRejection),

    #[error(transparent)]
    Query(#[from] QueryRejection),

    #[error("Validation error: {0}")]
    Validation(#[from] ValidationErrors),
}

impl<T, S> FromRequest<S> for WithRejection<T>
where
    S: Send + Sync,
    T: FromRequest<S>,
    RejectionError: From<T::Rejection>,
{
    type Rejection = RejectionError;

    /// Extracts a value using the inner extractor `T` from the request and returns it wrapped in `WithRejection`.
    ///
    /// # Returns
    ///
    /// An instance of `WithRejection<T>` containing the extracted value.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// // Intended usage (requires an async runtime and appropriate extractor implementations):
    /// // let wrapped = WithRejection::<Form<MyType>>::from_request(req, &state).await?;
    /// ```
    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        let extractor = T::from_request(req, state).await?;
        Ok(Self(extractor))
    }
}

impl<T, S> FromRequestParts<S> for WithRejection<T>
where
    S: Send + Sync,
    T: FromRequestParts<S>,
    RejectionError: From<T::Rejection>,
{
    type Rejection = RejectionError;

    /// Extracts `T` from the request parts and returns it wrapped in `WithRejection`.
    ///
    /// Attempts to perform `T::from_request_parts(parts, state)` and, on success, returns `Ok(WithRejection(extractor))`.
    ///
    /// # Returns
    ///
    /// `Ok(WithRejection(T))` when the inner extractor succeeds, `Err` when the inner extractor rejects.
    ///
    /// # Examples
    ///
    /// ```
    /// // Illustrative usage inside an async context implementing FromRequestParts:
    /// # async fn _example<S, T>(mut parts: axum::http::request::Parts, state: &S)
    /// # -> Result<(), T::Rejection>
    /// # where
    /// #     S: Send + Sync,
    /// #     T: axum::extract::FromRequestParts<S>,
    /// # {
    /// type Wrapped<T> = crate::extract::WithRejection<T>;
    /// // let wrapped = Wrapped::<T>::from_request_parts(&mut parts, state).await?;
    /// # Ok(())
    /// # }
    /// ```
    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let extractor = T::from_request_parts(parts, state).await?;
        Ok(Self(extractor))
    }
}

impl IntoResponse for RejectionError {
    /// Convert this `RejectionError` into an Axum HTTP response using the crate's error mapping.
    ///
    /// # Examples
    ///
    /// ```
    /// use crate::extract::with_rejection::RejectionError;
    /// let err = RejectionError::Validation(validator::ValidationErrors::new());
    /// let _resp = err.into_response();
    /// ```
    fn into_response(self) -> axum::response::Response {
        crate::Error::Rejection(self).into_response()
    }
}
