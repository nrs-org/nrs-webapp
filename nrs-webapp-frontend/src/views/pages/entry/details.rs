use hypertext::prelude::*;
use nrs_webapp_core::data::entry::types::idtype::EntryType;

pub struct EntryDetails {
    pub id: String,
    pub title: String,
    pub entry_type: EntryType,
    pub added_by_id: String,
    pub added_by_username: String,
    pub info_json: String,
}

pub fn entry_details_page(entry: &EntryDetails) -> impl Renderable {
    rsx! {
        <section class="flex flex-col items-center gap-10">
            <h1 class="font-bold text-3xl">("Entry Details Page (UNDER CONSTRUCTION)")</h1>
            <div>
                <h2 class="font-semibold text-2xl">(entry.title)</h2>
                <p>"Type: " (entry.entry_type.to_display_string())</p>
                <p>"ID: " (entry.id)</p>
                <p>"Added by: " (entry.added_by_username) " (ID: " (entry.added_by_id) ")"</p>
                <pre>
                    <code class="language-json">
                        (entry.info_json)
                    </code>
                </pre>
            </div>
        </section>
    }
}
