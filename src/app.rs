use leptos::prelude::*;
use leptos_meta::*;
use leptos_router::{
    components::{Route, Router, Routes},
    StaticSegment,
};
use loro::LoroDoc;

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    view! {
        <Stylesheet id="leptos" href="/style/output.css"/>
        <Link rel="shortcut icon" type_="image/ico" href="/favicon.ico"/>
        <Title text="Welcome to Leptos + Loro" />

        <Router>
            <Routes fallback=|| "Page not found.">
                <Route path=StaticSegment("") view=HomePage/>
            </Routes>
        </Router>
    }
}

/// Renders the home page of your application.
#[component]
fn HomePage() -> impl IntoView {
    let doc = StoredValue::new(LoroDoc::new());

    let version = RwSignal::new(-1);
    let max_version = RwSignal::new(0);
    let last_loro_id = RwSignal::new(None);

    let checkout_time = RwSignal::new(0.0);

    let text = RwSignal::new(String::new());

    // See the README for instructions on how to generate the snapshot
    let snapshot = include_bytes!("../public/seph-blog1.loro-snapshot");

    doc.update_value(|doc| {
        doc.import(snapshot.as_slice()).unwrap();
    });
    let frontiers = doc.with_value(|doc| doc.state_frontiers());
    let last = frontiers.last().cloned();
    leptos::logging::log!("Last version: {:#?}", last);
    last_loro_id.set(last);
    max_version.set(last.map(|id| id.counter).unwrap_or(0));

    view! {
        <h1>"Welcome to Leptos + Loro!"</h1>
        <div style="width: calc(100% - 32px); padding: 16px;">
            <RangeSelect
                version=version
                max_version=max_version.into()
                doc=doc
                text=text
                last_loro_id=last_loro_id.into()
                checkout_time=checkout_time
            />
            <RenderText text=text.into() checkout_time=checkout_time.into() />
        </div>
    }
}

#[component]
fn RangeSelect(
    version: RwSignal<i32>,
    max_version: Signal<i32>,
    doc: StoredValue<LoroDoc>,
    text: RwSignal<String>,
    last_loro_id: Signal<Option<loro::ID>>,
    checkout_time: RwSignal<f64>,
) -> impl IntoView {
    let range_on_input = leptos_use::use_throttle_fn_with_arg(
        move |ev: leptos::ev::Event| {
            let new_value = event_target_value(&ev).parse::<i32>().unwrap();
            version.set(new_value);

            let ts_start = leptos_use::use_timestamp().get();

            let ts_end: f64;

            if new_value == -1 {
                doc.update_value(|doc| doc.checkout(&[].into()));
                ts_end = leptos_use::use_timestamp().get();
                text.set("".to_string());
            } else {
                let new_loro_id = loro::ID {
                    peer: last_loro_id.get().unwrap().peer,
                    counter: new_value,
                };                
                doc.update_value(|doc| doc.checkout(&new_loro_id.into()));
                ts_end = leptos_use::use_timestamp().get();
                // As in the original example, we don't count the time it takes to read from the LoroDoc
                text.set(doc.with_value(|doc| doc.get_text("text").to_string()));
            }
            
            checkout_time.set(ts_end - ts_start);
        },
        100.0,
    );

    view! {
        <input
            type="range"
            style="width: 100%;"
            min="-1"
            max=move || max_version.get().to_string()
            value=move || version.get().to_string()
            on:input=move |ev| {
                range_on_input(ev);
            }
        />
        <div style="display: flex; justify-content: space-between; margin-top: 8px;">
            <span>"Current version: " {move || version.get()}</span>
            <span>"Max version: " {move || max_version.get()}</span>
        </div>
    }
}

#[component]
fn RenderText(text: Signal<String>, checkout_time: Signal<f64>) -> impl IntoView {
    view! {
        <div style="display: flex; justify-content: space-between; font-family: monospace;">
            <span style="margin-right: 2em;">
                "Checkout duration: " {move || format!("{:.2}", checkout_time.get())} " ms"
            </span>
            <span>"Text length: " {move || text.get().len()}</span>
        </div>
        <div style="position: relative; margin-top: 8px; transform: scale(1.075); transform-origin: 0px 0px 0px; text-align: left;">
            <div style="width: 100%; white-space: pre-wrap; transform: scale(0.8); transform-origin: 0px 0px 0px; position: absolute; top: 0px; left: 0px;">
                {move || text.get()}
            </div>
            <div style="width: 100%; white-space: pre-wrap; transform: scale(0.1) translateX(800%); transform-origin: 0px 0px 0px; position: absolute; top: 0px; left: 0px;">
                {move || text.get()}
            </div>
            <div style="width: 100%; white-space: pre-wrap; transform: scale(0.025) translateX(3600%); transform-origin: 0px 0px 0px; position: absolute; top: 0px; left: 0px;">
                {move || text.get()}
            </div>
        </div>
    }
}