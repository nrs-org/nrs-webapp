use hypertext::prelude::*;

#[component]
pub fn footer() -> impl Renderable {
    rsx! {
        <footer class="footer sm:footer-horizontal footer-center bg-base-300 text-base-content p-4">
            <aside>
                "Copyright © 2025 - All right reserved by btmxh"
            </aside>
        </footer>
    }
}
