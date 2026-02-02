use std::{
    fmt::{self, Display, Formatter},
    str::FromStr,
};

use super::{Error, Result};
use base64::{Engine, prelude::BASE64_URL_SAFE_NO_PAD};
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use time::OffsetDateTime;
use uuid::Uuid;

#[serde_as]
#[derive(Debug, Serialize, Deserialize)]
pub struct SessionToken {
    sub: Uuid,
    #[serde_as(as = "serde_with::TimestampSeconds")]
    expires_at: OffsetDateTime,
}

impl SessionToken {
    pub fn new(user_id: Uuid, expires_at: OffsetDateTime) -> Self {
        Self {
            sub: user_id,
            expires_at,
        }
    }

    pub fn validate(&self) -> Result<Uuid> {
        let now = OffsetDateTime::now_utc();
        if now > self.expires_at {
            return Err(Error::TokenExpired);
        }
        Ok(self.sub)
    }
}

impl Display for SessionToken {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let json = serde_json::to_string(self).map_err(|_| fmt::Error)?;
        let base64 = BASE64_URL_SAFE_NO_PAD.encode(json);
        write!(f, "{}", base64)
    }
}

impl FromStr for SessionToken {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        let json_bytes = BASE64_URL_SAFE_NO_PAD
            .decode(s)
            .map_err(|_| Error::InvalidTokenFormat)?;
        let json_str = String::from_utf8(json_bytes).map_err(|_| Error::InvalidTokenFormat)?;
        let token: SessionToken =
            serde_json::from_str(&json_str).map_err(|_| Error::InvalidTokenFormat)?;
        Ok(token)
    }
}
