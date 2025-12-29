use hypertext::prelude::*;

use super::Form;

pub fn confirm_mail(username: String) -> impl Renderable {
    rsx! {
        <Form form_id="confirmmail-form" title="Confirm your email" hx_post="/auth/confirmmail/resend">
            <p>Please verify your email address to activate your account</p>

            <p>"A confirmation email has been sent to your email address. Click the link in the email to confirm your email."</p>

            <input name="username" type="hidden" class="input" value=(username) required />

            <button type="submit" class="btn btn-neutral mt-4">Resend confirmation email</button>

            <p class="text-xs opacity-80">"Email delivery may take a few minutes. If you do not see the email, please check your spam or junk folder."</p>
        </Form>
    }
}
