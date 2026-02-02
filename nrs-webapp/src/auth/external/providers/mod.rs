use crate::auth::external::AuthProvider;

mod google;

pub fn google() -> Option<Box<dyn AuthProvider>> {
    google::GoogleAuthProvider::from_config().map(|p| Box::new(p) as Box<dyn AuthProvider>)
}
