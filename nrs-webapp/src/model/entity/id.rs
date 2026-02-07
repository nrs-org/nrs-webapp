use std::fmt::{Display, Formatter};

use uuid::Uuid;

#[derive(Debug, Clone)]
pub enum EntityId {
    Int(i64),
    String(String),
    Uuid(Uuid),
}

impl From<i64> for EntityId {
    fn from(id: i64) -> Self {
        EntityId::Int(id)
    }
}

impl From<String> for EntityId {
    fn from(id: String) -> Self {
        EntityId::String(id)
    }
}

impl From<&str> for EntityId {
    fn from(id: &str) -> Self {
        EntityId::String(id.to_string())
    }
}

impl From<Uuid> for EntityId {
    fn from(id: Uuid) -> Self {
        EntityId::Uuid(id)
    }
}

impl Display for EntityId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            EntityId::Int(id) => write!(f, "{}", id),
            EntityId::String(id) => write!(f, "{}", id),
            EntityId::Uuid(id) => write!(f, "{}", id),
        }
    }
}
