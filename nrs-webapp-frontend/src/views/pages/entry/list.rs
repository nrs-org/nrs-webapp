use hypertext::prelude::*;
use nrs_webapp_core::data::entry::types::idtype::EntryType;

use crate::views::components::link::{Link, LinkParams};

pub struct EntryListEntry {
    pub id: String,
    pub title: String,
    pub entry_type: EntryType,
    pub added_by: String,
}

pub fn entry_list_page(entries: &[EntryListEntry]) -> impl Renderable {
    rsx! {
        <div class="flex flex-col items-center gap-10 w-full max-w-4xl">
            <h1 class="font-bold text-3xl">("Entry List Page (UNDER CONSTRUCTION)")</h1>
            <ul>
                @for EntryListEntry { id, title, entry_type, added_by } in entries {
                    @let href = format!("/entry/{}", id);
                    <li>
                        <Link params=(LinkParams {href: href.as_str(), class: "link link-hover", ..Default::default()})>
                            (title)" (" (entry_type.to_display_string()) ", added by " (added_by) ")"
                        </Link>
                    </li>
                }
            </ul>
        </div>
    }
}
