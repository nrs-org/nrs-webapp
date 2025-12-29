use crate::crypt::jwt::JwtClaims;

#[derive(Debug, Clone)]
pub struct Session {
    pub user_id: String,
}

impl From<JwtClaims> for Session {
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
    fn from(value: JwtClaims) -> Self {
        Self { user_id: value.sub }
    }
}
