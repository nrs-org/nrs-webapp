use axum::http::Method;
use heroicons::{
    Icon,
    icon_name::{Bars3, Moon, Sun},
    icon_variant::Outline,
};
use hypertext::prelude::*;

use crate::views::components::link::{Link, LinkParams};

/// Renders a theme toggle control that swaps between sun and moon icons.
///
/// The component emits a hidden checkbox (value `"night"`) and two Icon elements
/// whose visibility toggles via the surrounding "swap" classes; intended to be
/// used as a UI hook for switching light/dark theme states.
///
/// # Examples
///
/// ```
/// // Instantiate the component in a parent view
/// let _component = theme_controller();
/// ```
#[component]
fn theme_controller() -> impl Renderable {
    rsx! {
        <label class="hidden swap swap-rotate">
            <input type="checkbox" class="theme-controller" value="night" />
            <Icon class="size-6 swap-off" name=(Sun) variant=(Outline) .. />
            <Icon class="size-6 swap-on" name=(Moon) variant=(Outline) .. />
        </label>
    }
}

/// Renders the application's responsive navigation bar.
///
/// The header includes primary route links, a brand link, a theme toggle, and authentication controls:
/// - Desktop and mobile navigation for the routes "Home" and "Entries".
/// - A theme toggle control.
/// - When `logged_in` is `true`, a user avatar with a dropdown containing "Profile" and a "Logoff" action (POST to `/auth/logoff`).
/// - When `logged_in` is `false`, a "Log in" button that loads the login fragment via HTMX.
///
/// # Parameters
///
/// - `logged_in`: when `true`, show the authenticated user menu; when `false`, show the login trigger.
///
/// # Examples
///
/// ```
/// // Render the navbar for an anonymous user
/// let _ = navbar(false);
/// ```
#[component]
pub fn navbar(logged_in: bool) -> impl Renderable {
    let routes = [("Home", "/"), ("Entries", "/entries")];

    rsx! {
        <header class="navbar bg-base-100 shadow-sm">
            <section class="navbar-start">
                <div class="dropdown">
                    <div tabindex="0" role="button" class="btn btn-ghost lg:hidden">
                        <Icon class="size-6" name=(Bars3) variant=(Outline) .. />
                    </div>

                    <ul tabindex="-1" class="menu menu-sm dropdown-content bg-base-100 rounded-box z-50 mt-3 w-52 p-2 shadow">
                        @for (label, href) in routes.iter() {
                            <li><Link params=(LinkParams{href,..Default::default()})>(label)</Link></li>
                        }
                    </ul>
                </div>

                <Link params=(LinkParams{class:"btn btn-ghost text-xl", href: "/", ..Default::default()})>
                    <span>"nrs-"<em>"webapp"</em></span>
                </Link>
            </section>

            <section class="navbar-center hidden lg:flex">
                <ul class="menu menu-horizontal px-1">
                    @for (label, href) in routes.iter() {
                        <li><Link params=(LinkParams{href,..Default::default()})>(label)</Link></li>
                    }
                </ul>
            </section>

            <section class="navbar-end gap-2">
                <ThemeController />
                @if logged_in {
                    <div class="dropdown dropdown-end">
                        <div tabindex="0" role="button" class="btn btn-ghost btn-circle avatar avatar-placeholder">
                            <div class="w-10 rounded-full bg-neutral-focus text-neutral-content flex items-center justify-center">
                                <span>"U"</span>
                            </div>
                        </div>

                        <ul tabindex="-1" class="mt-3 z-50 p-2 shadow menu menu-sm dropdown-content bg-base-100 rounded-box w-32">
                            <li><Link params=(LinkParams { href: "/profile", ..Default::default() })>Profile</Link></li>
                            <li><Link params=(LinkParams { href: "/auth/logoff", hx_vals: "{\"logoff\":true}", method: Method::POST, ..Default::default() })>Logoff</Link></li>
                        </ul>
                    </div>
                } @else {
                    <a class="btn btn-primary" hx-get="/auth/login" hx-target="#page" hx-swap="innerHTML" hx-push-url=true>Log in</a>
                }
            </section>
        </header>
    }
}
