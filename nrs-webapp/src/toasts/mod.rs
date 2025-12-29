use hypertext::prelude::*;
use nrs_webapp_frontend::views::components::toast::{Toast, ToastKind};
use strum::{EnumString, IntoStaticStr};

#[derive(EnumString, IntoStaticStr)]
pub enum ConstToast {
    LoginAgainAfterEmailVerification,
    LoginAgainAfterPasswordReset,
}

impl From<ConstToast> for Toast {
    /// Converts a `ConstToast` variant into a `Toast` configured for display.
    ///
    /// Each variant is mapped to a `Toast` with appropriate `kind`, `title`, and rendered `description`.
    ///
    /// # Examples
    ///
    /// ```
    /// use nrs_webapp::toasts::{ConstToast, Toast, ToastKind};
    ///
    /// let t: Toast = ConstToast::LoginAgainAfterEmailVerification.into();
    /// assert_eq!(t.kind, ToastKind::Success);
    /// assert_eq!(t.title, "Email Verified");
    /// ```
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_toast_default() {
        let toast = Toast::default();
        
        assert_eq!(toast.kind, ToastKind::Info);
        assert_eq!(toast.message, "");
    }

    #[test]
    fn test_toast_info() {
        let toast = Toast::info("Info message");
        
        assert_eq!(toast.kind, ToastKind::Info);
        assert_eq!(toast.message, "Info message");
    }

    #[test]
    fn test_toast_success() {
        let toast = Toast::success("Success message");
        
        assert_eq!(toast.kind, ToastKind::Success);
        assert_eq!(toast.message, "Success message");
    }

    #[test]
    fn test_toast_warning() {
        let toast = Toast::warning("Warning message");
        
        assert_eq!(toast.kind, ToastKind::Warning);
        assert_eq!(toast.message, "Warning message");
    }

    #[test]
    fn test_toast_error() {
        let toast = Toast::error("Error message");
        
        assert_eq!(toast.kind, ToastKind::Error);
        assert_eq!(toast.message, "Error message");
    }

    #[test]
    fn test_toast_kind_ordering() {
        assert!(ToastKind::Info < ToastKind::Success);
        assert!(ToastKind::Success < ToastKind::Warning);
        assert!(ToastKind::Warning < ToastKind::Error);
    }

    #[test]
    fn test_toast_kind_equality() {
        assert_eq!(ToastKind::Info, ToastKind::Info);
        assert_ne!(ToastKind::Info, ToastKind::Error);
    }

    #[test]
    fn test_toast_clone() {
        let toast1 = Toast::success("Test");
        let toast2 = toast1.clone();
        
        assert_eq!(toast1.kind, toast2.kind);
        assert_eq!(toast1.message, toast2.message);
    }

    #[test]
    fn test_toast_debug() {
        let toast = Toast::info("Test message");
        let debug_str = format!("{:?}", toast);
        
        assert!(debug_str.contains("Toast"));
    }

    #[test]
    fn test_toast_empty_message() {
        let toast = Toast::info("");
        
        assert_eq!(toast.message, "");
    }

    #[test]
    fn test_toast_long_message() {
        let long_msg = "x".repeat(1000);
        let toast = Toast::error(&long_msg);
        
        assert_eq!(toast.message, long_msg);
    }

    #[test]
    fn test_toast_with_special_characters() {
        let msg = "Message with\nnewlines\tand\ttabs & symbols!";
        let toast = Toast::warning(msg);
        
        assert_eq!(toast.message, msg);
    }

    #[test]
    fn test_toast_with_unicode() {
        let msg = "消息 🎉 Сообщение";
        let toast = Toast::success(msg);
        
        assert_eq!(toast.message, msg);
    }

    #[test]
    fn test_toast_with_html() {
        let msg = "<script>alert('xss')</script>";
        let toast = Toast::error(msg);
        
        assert_eq!(toast.message, msg);
        // Note: The rendering layer should escape this
    }

    #[test]
    fn test_multiple_toasts_different_kinds() {
        let toasts = vec![
            Toast::info("Info"),
            Toast::success("Success"),
            Toast::warning("Warning"),
            Toast::error("Error"),
        ];
        
        assert_eq!(toasts.len(), 4);
        assert_eq!(toasts[0].kind, ToastKind::Info);
        assert_eq!(toasts[1].kind, ToastKind::Success);
        assert_eq!(toasts[2].kind, ToastKind::Warning);
        assert_eq!(toasts[3].kind, ToastKind::Error);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_const_toast_to_string() {
        let t1: &'static str = ConstToast::LoginAgainAfterEmailVerification.into();
        assert_eq!(t1, "LoginAgainAfterEmailVerification");
        
        let t2: &'static str = ConstToast::LoginAgainAfterPasswordReset.into();
        assert_eq!(t2, "LoginAgainAfterPasswordReset");
    }

    #[test]
    fn test_const_toast_from_str() {
        use std::str::FromStr;
        
        let result1 = ConstToast::from_str("LoginAgainAfterEmailVerification");
        assert!(result1.is_ok());
        
        let result2 = ConstToast::from_str("LoginAgainAfterPasswordReset");
        assert!(result2.is_ok());
    }

    #[test]
    fn test_const_toast_from_str_invalid() {
        use std::str::FromStr;
        
        let result = ConstToast::from_str("InvalidToastName");
        assert!(result.is_err());
    }

    #[test]
    fn test_const_toast_into_toast_email_verification() {
        let const_toast = ConstToast::LoginAgainAfterEmailVerification;
        let toast: Toast = const_toast.into();
        
        assert_eq!(toast.kind, ToastKind::Success);
        assert_eq!(toast.title, "Email Verified");
        assert!(!toast.description.as_inner().is_empty());
    }

    #[test]
    fn test_const_toast_into_toast_password_reset() {
        let const_toast = ConstToast::LoginAgainAfterPasswordReset;
        let toast: Toast = const_toast.into();
        
        assert_eq!(toast.kind, ToastKind::Success);
        assert_eq!(toast.title, "Password Reset Successful");
        assert!(!toast.description.as_inner().is_empty());
    }

    #[test]
    fn test_all_const_toasts_produce_valid_toasts() {
        let toasts = vec![
            ConstToast::LoginAgainAfterEmailVerification,
            ConstToast::LoginAgainAfterPasswordReset,
        ];
        
        for const_toast in toasts {
            let toast: Toast = const_toast.into();
            assert!(!toast.title.is_empty());
            assert!(!toast.description.as_inner().is_empty());
        }
    }
}