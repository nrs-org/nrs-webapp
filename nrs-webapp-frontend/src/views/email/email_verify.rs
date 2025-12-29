use hypertext::prelude::*;

/// Render a verification email body containing a greeting, account notice, verification link, and dismissal note.
///
/// The returned renderable includes the provided `username` and `href` in the visible content and sets the link to open in a new tab with `rel="noopener noreferrer"`.
///
/// # Examples
///
/// ```
/// let _view = email_verify("alice", "https://example.com/verify");
/// ```
pub fn email_verify<'a>(username: &'a str, href: &'a str) -> impl Renderable {
    rsx! {
        <main>
            <p>"Hi, "(username)</p>
            <p>"An account on nrs-"<em>webapp</em>" has been registered using this email address."</p>
            <p>"Please click the following link to verify your email address:"</p>
            <a href=(href) target="_blank" rel="noopener noreferrer">(href)</a>
            <p>"If you did not register an account, please ignore this email."</p>
        </main>
    }
}
