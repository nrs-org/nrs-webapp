use hypertext::prelude::*;

use super::Form;
use crate::views::components::link::{Link, LinkParams};

pub fn forgot_pass() -> impl Renderable {
    rsx! {
        <Form form_id="forgotpass-form" title="Recover password" hx_post="/auth/forgotpass">
            <label class="label" for="forgotpass-email">Email</label>
            <input id="forgotpass-email" name="email" type="email" class="input validator w-full" required placeholder="Email" />
            <div class="validator-hint hidden">Please enter a valid email</div>

            <button type="submit" class="btn btn-neutral mt-4">Reset password</button>

            <Link params=(LinkParams{href:"/auth/login", class:"btn btn-secondary", ..Default::default()})>"Back to login page"</Link>
        </Form>
    }
}

pub fn forgot_pass_sent() -> impl Renderable {
    rsx! {
        <Form form_id="forgotpass-sent-form" title="Password reset email sent" hx_post="/auth/forgotpass">
            <p>Please check your email to reset your password</p>

            <p>"An email containing the password reset link has been sent to your email address. Click the link in the email to reset your password."</p>

            <p class="text-xs opacity-80">"Email delivery may take a few minutes. If you do not see the email, please check your spam or junk folder."</p>
        </Form>
    }
}

pub fn reset_pass(token: String) -> impl Renderable {
    rsx! {
        <Form form_id="reset-form" title="Recover password" hx_post="/auth/forgotpass/reset">
            <input type="hidden" name="token" value=(token) />

            <label class="label" for="reset-password">New password</label>
            <input
                id="reset-password" name="password" type="password" class="input validator w-full" required placeholder="New password"
                minlength="8" pattern="(?=.*\\d)(?=.*[a-z])(?=.*[A-Z]).{8,}"
                title="Must be more than 8 characters, including number, lowercase letter, uppercase letter"
            />
            <p class="validator-hint hidden">
              "Must be more than 8 characters, including"
              <br/>At least one number
              <br/>At least one lowercase letter
              <br/>At least one uppercase letter
            </p>

            <label class="label" for="reset-password-confirm">Confirm new password</label>
            <input
                id="reset-password-confirm" name="password_confirm" type="password" class="input validator w-full" required placeholder="Confirm new password"
                oninput="this.setCustomValidity(this.value != document.getElementById('reset-password').value ? 'Passwords do not match' : '')"
            />
            <p class="validator-hint hidden">Passwords do not match</p>

            <button type="submit" class="btn btn-neutral mt-4">Reset password</button>

            <Link params=(LinkParams{href:"/auth/login", class:"btn btn-secondary", ..Default::default()})>"Back to login page"</Link>
        </Form>
    }
}
