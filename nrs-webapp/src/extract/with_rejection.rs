use axum::{
    Form, Json,
    extract::{
        FromRef, FromRequest, FromRequestParts, Query, Request,
        rejection::{FormRejection, JsonRejection, QueryRejection},
    },
    http::request::Parts,
    response::IntoResponse,
};
use thiserror::Error;
use validator::{Validate, ValidateArgs, ValidationError, ValidationErrors};

pub struct WithRejection<T>(pub T);

pub struct WRForm<T>(pub T);
pub struct WRQuery<T>(pub T);

impl<T, S> FromRequest<S> for WRForm<T>
where
    S: Send + Sync,
    WithRejection<Form<T>>: FromRequest<S>,
{
    type Rejection = <WithRejection<Form<T>> as FromRequest<S>>::Rejection;

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

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let extractor = T::from_request_parts(parts, state).await?;
        Ok(Self(extractor))
    }
}

impl IntoResponse for RejectionError {
    fn into_response(self) -> axum::response::Response {
        crate::Error::Rejection(self).into_response()
    }
}
