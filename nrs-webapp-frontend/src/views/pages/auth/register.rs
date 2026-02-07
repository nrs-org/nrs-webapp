use hypertext::prelude::*;

use super::Form;
use crate::views::components::link::{Link, LinkParams};

pub enum RegisterScreen {
    Regular,
    OAuth {
        username: Option<String>,
        email: Option<String>,
    },
}

/// Render the sign-up form UI fragment.
///
/// The fragment contains username, email, password, and password confirmation inputs with
/// client-side validation attributes and hints, a "Register" submit button, and a link
/// back to the login page.
///
/// # Examples
///
/// ```
/// use nrs_webapp_frontend::views::pages::auth::register::register;
/// use nrs_webapp_frontend::views::pages::auth::register::RegisterScreen;
/// let _fragment = register(RegisterScreen::Regular);
/// ```
pub fn register(screen: RegisterScreen) -> impl Renderable {
    let hx_post = match &screen {
        RegisterScreen::Regular => "/auth/register",
        RegisterScreen::OAuth { .. } => "/auth/oauth/register",
    };
    let (username, email, email_readonly) = match screen {
        RegisterScreen::Regular => Default::default(),
        RegisterScreen::OAuth { username, email } => {
            let username = username.unwrap_or_default();
            let (email, email_readonly) = match email {
                Some(email) => (email, true),
                None => (String::new(), false),
            };
            (username, email, email_readonly)
        }
    };
    rsx! {
        <Form form_id="signup-form" title="Sign up" hx_post=(hx_post)>
            <label class="label" for="signup-username">Username</label>
            <input id="signup-username" name="username" type="text" class="input validator w-full" required placeholder="Username"
                minlength="3" maxlength="20"
                pattern="[A-Za-z0-9_\\-]{3,20}" title="3-20 characters: letters, numbers, underscores and dashes"
                autocomplete="username" value=(username)
            />
            <p class="validator-hint hidden">
              "Must be 3 to 20 characters"
              <br/>"containing only letters, numbers, underscores or dashes"
            </p>

            <label class="label" for="signup-email">Email</label>
            <input id="signup-email" name="email" type="email" class="input validator w-full" required placeholder="Email"
                autocomplete = "email" value=(email) readonly=(email_readonly)
            />
            <div class="validator-hint hidden">Please enter a valid email</div>

            <label class="label" for="signup-password">Password</label>
            <input
                id="signup-password" name="password" type="password" class="input validator w-full" required placeholder="Password"
                minlength="8" maxlength="50" pattern="(?=.*\\d)(?=.*[a-z])(?=.*[A-Z]).{8,}"
                title="Must be 8-50 characters, including number, lowercase letter, uppercase letter"
                oninput="document.getElementById('signup-password-confirm').dispatchEvent(new Event('input'))"
                autocomplete="new-password"
            />
            <p class="validator-hint hidden">
              "Must be 8-50 characters, including"
              <br/>At least one number
              <br/>At least one lowercase letter
              <br/>At least one uppercase letter
            </p>

            <label class="label" for="signup-password-confirm">Confirm Password</label>
            <input
                id="signup-password-confirm" name="password_confirm" type="password" class="input validator w-full" required placeholder="Confirm Password"
                oninput="this.setCustomValidity(this.value != document.getElementById('signup-password').value ? 'Passwords do not match' : '')"
                autocomplete="new-password"
            />
            <p class="validator-hint hidden">Passwords do not match</p>

            <button type="submit" class="btn btn-neutral mt-4">Register</button>

            <Link params=(LinkParams{href:"/auth/login", class:"btn btn-secondary", ..Default::default()})>"Back to login page"</Link>
        </Form>
    }
}
