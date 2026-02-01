use uuid::Uuid;

use super::{Error, Result};
use crate::crypt::jwt::JwtClaims;

#[derive(Debug, Clone)]
pub struct Session {
    pub user_id: Uuid,
}

impl TryFrom<JwtClaims> for Session {
    type Error = Error;

    /// Creates a `Session` from JWT claims.
    ///
    /// The session's `user_id` is populated from the claims' `sub` field.
    ///
    /// # Examples
    ///
    /// ```
    /// // Construct JwtClaims with a subject and convert into a Session.
    /// let claims = JwtClaims { sub: String::from("user123") };
    /// let session = Session::from(claims);
    /// assert_eq!(session.user_id, "user123");
    /// ```
    fn try_from(value: JwtClaims) -> Result<Self> {
        value
            .sub
            .parse::<Uuid>()
            .map_err(Error::UuidParseError)
            .map(|user_id| Session { user_id })
    }
}
