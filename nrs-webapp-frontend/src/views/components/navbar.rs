use heroicons::{
    Icon,
    icon_name::{Bars3, Moon, Sun},
    icon_variant::Outline,
};
use hypertext::prelude::*;

#[component]
fn item<'a>(label: &'a str, href: &'a str) -> impl Renderable {
    rsx! {
        <a role="link" hx-get=(href) hx-target="#page" hx-swap="innerHTML" hx-push-url=true>(label)</a>
    }
}

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
                            <li><Item label=(label) href=(href) /></li>
                        }
                    </ul>
                </div>

                <a class="btn btn-ghost text-xl" hx-get="/" hx-target="#page" hx-swap="innerHTML" hx-push-url=true>
                    <span>"nrs-"<em>"webapp"</em></span>
                </a>
            </section>

            <section class="navbar-center hidden lg:flex">
                <ul class="menu menu-horizontal px-1">
                    @for (label, href) in routes.iter() {
                        <li><Item label=(label) href=(href) /></li>
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
                            <li><Item label="Profile" href="/profile" /></li>
                            <li><a role="link" hx-post="/logoff" hx-vals="{\"logoff\":true}" hx-target="#page" hx-swap="innerHTML" hx-push-url=true>Logoff</a></li>
                        </ul>
                    </div>
                } @else {
                    <a class="btn btn-primary" hx-get="/login" hx-target="#page" hx-swap="innerHTML" hx-push-url=true>Log in</a>
                }
            </section>
        </header>
    }
}
