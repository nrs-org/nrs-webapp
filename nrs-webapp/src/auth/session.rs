use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct Session {
    pub user_id: Uuid,
}

impl Session {
    pub fn new(user_id: Uuid) -> Self {
        Self { user_id }
    }
}
