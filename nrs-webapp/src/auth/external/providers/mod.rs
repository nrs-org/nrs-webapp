use crate::auth::external::AuthProvider;

mod github;
mod google;

pub fn google() -> Option<Box<dyn AuthProvider>> {
    google::GoogleAuthProvider::from_config().map(|p| Box::new(p) as Box<dyn AuthProvider>)
}

pub fn github() -> Option<Box<dyn AuthProvider>> {
    github::GithubAuthProvider::from_config().map(|p| Box::new(p) as Box<dyn AuthProvider>)
}
