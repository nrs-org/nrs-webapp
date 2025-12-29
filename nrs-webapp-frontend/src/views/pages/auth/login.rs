use hypertext::prelude::*;
use strum::{EnumString, IntoStaticStr};

use crate::views::{
    components::link::{Link, LinkParams},
    icons::{Github, Google},
};

use super::Form;

#[derive(EnumString, IntoStaticStr)]
pub enum LoginToast {
    LoginAgainAfterEmailVerification,
}

pub fn login() -> impl Renderable {
    rsx! {
        <Form form_id="signin-form" title="Sign in" hx_post="/auth/login">
            <label class="label" for="signin-username">Username</label>
            <input id="signin-username" name="username" type="text" class="input w-full validator" required placeholder="Username" />
            <div class="validator-hint hidden">Please enter a valid username</div>

            <label class="label" for="signin-password">Password</label>
            <input id="signin-password" name="password" type="password" class="input w-full validator" required placeholder="Password" />
            <div class="validator-hint hidden">Please enter your password</div>

            <button type="submit" class="btn btn-neutral mt-4">Login</button>

            <div class="flex justify-between w-full">
                <Link params=(LinkParams{href:"/auth/forgotpass", class:"link", ..Default::default()})>"Forgot password?"</Link>
                <Link params=(LinkParams{href:"/auth/register", class:"link", ..Default::default()})>"Create new account"</Link>
            </div>

            <div class="divider"></div>

            <button class="btn bg-black text-white border-black">
                <Github />
                Login with GitHub
            </button>

            <button class="btn bg-white text-black border-[#e5e5e5]">
                <Google />
                Login with Google
            </button>
        </Form>
    }
}
