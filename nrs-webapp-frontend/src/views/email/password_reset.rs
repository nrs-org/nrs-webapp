use hypertext::prelude::*;

pub fn password_reset<'a>(username: &'a str, href: &'a str) -> impl Renderable {
    rsx! {
        <main>
            <p>"Hi, "(username)</p>
            <p>"A password reset request for your account on nrs-"<em>webapp</em>" has been received."</p>
            <p>"Please click the following link to reset your password:"</p>
            <a href=(href) target="_blank" rel="noopener noreferrer">(href)</a>
            <p>"If you did not request for a password reset, please ignore this email."</p>
        </main>
    }
}
