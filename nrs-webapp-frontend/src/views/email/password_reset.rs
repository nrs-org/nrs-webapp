use hypertext::prelude::*;

/// Creates a renderable password-reset email fragment personalized for the given user.
///
/// The returned `Renderable` contains a <main> element with a greeting that includes `username`,
/// a notice that a password reset was requested for the site, an anchor linking to `href`,
/// and a final note instructing the recipient to ignore the message if they did not request it.
///
/// # Examples
///
/// ```
/// use hypertext::prelude::*;
/// use nrs_webapp_frontend::views::email::password_reset::password_reset;
/// let _view = password_reset("alice", "https://example.com/reset");
/// ```
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
