use hypertext::prelude::*;

#[component]
fn basic_hero<R: Renderable>(children: &R) -> impl Renderable {
    rsx! {
        <section class="hero bg-base-200 min-h-96 md:min-h-[500px] p-10">
            <div class="hero-content w-full text-center flex flex-col">
                (children)
            </div>
        </section>
    }
}

pub fn home() -> impl Renderable {
    rsx! {
        <div class="flex flex-col items-center gap-10">
            <BasicHero>
                <div class="max-w-lg">
                    <h1 class="text-4xl font-bold">
                        <strong class="font-black">NRS</strong>
                        " - New Rating System"
                    </h1>
                    <div class="py-6">
                        <p>
                            <strong>NRS</strong>" is a system for ranking "
                            <strong>entries</strong>", which can be anime, manga, novels,
                            music albums, or anything really."
                        </p>
                        <p>
                            "It utilizes an unified pipeline to calculate ratings across all
                            types of entries, ensuring fairness and consistency."
                        </p>
                    </div>

                    <a
                        class="btn btn-primary"
                        hx-get="/entries"
                        hx-target="#page"
                        hx-swap="innerHTML"
                        hx-push-url="true"
                    >
                        "See rankings"
                    </a>
                </div>
            </BasicHero>

            <BasicHero>
                <section>
                    <h1 class="text-4xl font-bold">"How it works?"</h1>
                    <h2 class="text-2xl mt-4">
                        "We utilize " <strong>"impacts"</strong>
                        " and " <strong>"relations"</strong> " to score entries."
                    </h2>
                </section>

                <section class="flex w-full flex-col lg:flex-row m-4 justify-center items-center">
                    <div class="card lg:w-[40%] p-4 bg-base-300 rounded-box flex-col">
                        <h3 class="text-xl font-semibold uppercase mb-2">"Impacts"</h3>
                        <p>
                            "Impacts are what the entry specifically achieved, such as making one tears up, feels excited, or be inspired."
                        </p>
                        <p>
                            "Each entry grants its respective impact "
                            <strong>"constant"</strong>
                            " score, which would then be combined with other impacts to form the final score."
                        </p>
                        <p class="font-extralight text-sm mt-4">
                            "If the music track Alice made the listener feels depressed for a month, it got a one-month \"PADS\" impact."
                        </p>
                    </div>
                    <div class="divider lg:divider-horizontal">AND</div>
                    <div class="card lg:w-[40%] p-4 bg-base-300 rounded-box flex-col">
                        <h3 class="text-xl font-semibold uppercase mb-2">"Relations"</h3>
                        <p>
                            "Relations are how the entry connects with other entries, like being a prequel to a masterpiece sequel, inspiring new works, or just authorship in general."
                        </p>
                        <p>
                            "Each relation grants its respective entry "
                            <strong>"a portion of"</strong>
                            " the score from the referenced entries."
                        </p>
                        <p class="font-extralight text-sm mt-4">
                            "If the anime Atri fully inspires an influential music track Alice, it got 20% of Alice's total score."
                        </p>
                    </div>
                </section>


                <h2 class="text-2xl mt-4">
                    "The total score for each entry is calculated by "
                    <strong>"combining"</strong>
                    " all the scores from contributing impacts and relations."
                </h2>

                <h3 class="text-xl font-medium mt-6">
                    "For the mathematically inclined, go "
                    <a class="link" target="_blank" href="https://btmxh.github.io/blog/posts/combine">"here"</a>
                    " to see how the combine functions are derived."
                </h3>
            </BasicHero>
        </div>
    }
}
