use crate::crypt::jwt::JwtClaims;

#[derive(Debug, Clone)]
pub struct Session {
    pub user_id: String,
}

impl From<JwtClaims> for Session {
    fn from(value: JwtClaims) -> Self {
        Self { user_id: value.sub }
    }
}
