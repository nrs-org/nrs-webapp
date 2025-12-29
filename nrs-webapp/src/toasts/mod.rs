use hypertext::prelude::*;
use nrs_webapp_frontend::views::components::toast::{Toast, ToastKind};
use strum::{EnumString, IntoStaticStr};

#[derive(EnumString, IntoStaticStr)]
pub enum ConstToast {
    LoginAgainAfterEmailVerification,
    LoginAgainAfterPasswordReset,
}

impl From<ConstToast> for Toast {
    fn from(value: ConstToast) -> Self {
        match value {
            ConstToast::LoginAgainAfterEmailVerification => Toast {
                kind: ToastKind::Success,
                title: "Email Verified".to_string(),
                description: rsx! {"Please log in again to continue."}.render(),
            },
            ConstToast::LoginAgainAfterPasswordReset => Toast {
                kind: ToastKind::Success,
                title: "Password Reset Successful".to_string(),
                description:
                    rsx! {"Your password has been reset. Please log in again to continue."}.render(),
            },
        }
    }
}

impl ConstToast {}
