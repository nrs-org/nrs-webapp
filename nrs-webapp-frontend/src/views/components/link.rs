use axum::http::Method;
use hypertext::prelude::*;

#[derive(Default)]
pub struct LinkParams<'a> {
    pub href: &'a str,
    pub class: &'a str,
    pub hx_vals: &'a str,
    pub method: Method,
}

/// Renders an anchor (<a>) configured for htmx-driven navigation.
///
/// The anchor uses `params.class` for CSS classes, `params.hx_vals` for htmx values,
/// and targets `#page` with `hx-swap="innerHTML"` and `hx-push-url=true`. It sets
/// `hx-get` to `params.href` when `params.method` is `Method::GET`, or `hx-post` to
/// `params.href` when `params.method` is `Method::POST`.
///
/// # Examples
///
/// ```
/// use axum::http::Method;
///
/// let params = LinkParams {
///     href: "/items",
///     class: "nav-link",
///     hx_vals: "{}",
///     method: Method::GET,
/// };
///
/// let node = link(&"Open items", &params);
/// ```
#[component]
pub fn link<'a, R: Renderable>(children: &R, params: &LinkParams<'a>) -> impl Renderable {
    // TODO: add more if needed
    let hx_get = (params.method == Method::GET).then_some(params.href);
    let hx_post = (params.method == Method::POST).then_some(params.href);
    rsx! {
        <a role="link" class=(params.class) hx-get=(hx_get) hx-post=(hx_post) hx-target="#page" hx-swap="innerHTML" hx-push-url=true hx-vals=(params.hx_vals)>
            (children)
        </a>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_link_params_default() {
        let params = LinkParams::default();
        
        assert_eq!(params.href, "");
        assert_eq!(params.class, "");
        assert_eq!(params.hx_vals, "");
        assert_eq!(params.method, Method::GET);
    }

    #[test]
    fn test_link_params_custom_get() {
        let params = LinkParams {
            href: "/items",
            class: "btn-primary",
            hx_vals: "{}",
            method: Method::GET,
        };
        
        assert_eq!(params.href, "/items");
        assert_eq!(params.class, "btn-primary");
        assert_eq!(params.method, Method::GET);
    }

    #[test]
    fn test_link_params_custom_post() {
        let params = LinkParams {
            href: "/submit",
            class: "btn-success",
            hx_vals: r#"{"action":"create"}"#,
            method: Method::POST,
        };
        
        assert_eq!(params.href, "/submit");
        assert_eq!(params.method, Method::POST);
        assert_eq!(params.hx_vals, r#"{"action":"create"}"#);
    }

    #[test]
    fn test_link_params_empty_href() {
        let params = LinkParams {
            href: "",
            ..Default::default()
        };
        
        assert_eq!(params.href, "");
    }

    #[test]
    fn test_link_params_various_methods() {
        let get_params = LinkParams {
            method: Method::GET,
            ..Default::default()
        };
        assert_eq!(get_params.method, Method::GET);
        
        let post_params = LinkParams {
            method: Method::POST,
            ..Default::default()
        };
        assert_eq!(post_params.method, Method::POST);
    }

    #[test]
    fn test_link_params_long_href() {
        let long_href = "/very/long/path/to/resource/with/many/segments";
        let params = LinkParams {
            href: long_href,
            ..Default::default()
        };
        
        assert_eq!(params.href, long_href);
    }

    #[test]
    fn test_link_params_complex_hx_vals() {
        let complex_json = r#"{"key":"value","nested":{"a":1,"b":2}}"#;
        let params = LinkParams {
            hx_vals: complex_json,
            ..Default::default()
        };
        
        assert_eq!(params.hx_vals, complex_json);
    }

    #[test]
    fn test_link_params_with_query_string() {
        let params = LinkParams {
            href: "/search?q=test&page=1",
            ..Default::default()
        };
        
        assert_eq!(params.href, "/search?q=test&page=1");
    }

    #[test]
    fn test_link_params_with_fragment() {
        let params = LinkParams {
            href: "/page#section",
            ..Default::default()
        };
        
        assert_eq!(params.href, "/page#section");
    }

    #[test]
    fn test_link_params_multiple_css_classes() {
        let params = LinkParams {
            class: "btn btn-primary btn-lg",
            ..Default::default()
        };
        
        assert_eq!(params.class, "btn btn-primary btn-lg");
    }
}