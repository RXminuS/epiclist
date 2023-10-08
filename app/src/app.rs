use crate::error_template::{AppError, ErrorTemplate};
use leptos::*;
use leptos_meta::*;
use leptos_router::*;

#[component]
pub fn App() -> impl IntoView {
    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context();

    view! {


        // injects a stylesheet into the document <head>
        // id=leptos means cargo-leptos will hot-reload this stylesheet
        <Stylesheet id="leptos" href="/pkg/app.css"/>

        // sets the document title
        <Title text="Welcome to Leptos"/>

        // content for this welcome page
        <Router fallback=|| {
            let mut outside_errors = Errors::default();
            outside_errors.insert_with_default_key(AppError::NotFound);
            view! {
                <ErrorTemplate outside_errors/>
            }
            .into_view()
        }>
            <main>
                <Routes>
                    <Route path="" view=HomePage/>
                </Routes>
            </main>
        </Router>
    }
}

/// Renders the home page of your application.
#[component]
fn HomePage() -> impl IntoView {
    // Creates a reactive value to update the button
    let (count, set_count) = create_signal(0);
    let on_plus = move |_| set_count.update(|count| *count += 1);
    let on_minus = move |_| set_count.update(|count| *count -= 1);

    view! {
        <Title text="Leptos + Tailwindcss"/>
        <main data-theme="light">
            <div class="flex flex-col min-h-screen font-mono text-white bg-gradient-to-tl from-gray-900 to-gray-700">
                <div class="flex flex-row-reverse flex-wrap items-center m-auto">
                    <button on:click=on_plus class="btn">
                        "+"
                    </button>
                    <div class="px-2">
                        {count}
                    </div>
                    <button on:click=on_minus class="btn">
                        "-"
                    </button>
                </div>
            </div>
        </main>
    }
}
