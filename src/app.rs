use leptos::prelude::*;
use leptos_meta::{provide_meta_context, MetaTags, Stylesheet, Title};
use leptos_router::{
    components::{Route, Router, Routes},
    StaticSegment,
};

use loro::LoroDoc;

pub fn shell(options: LeptosOptions) -> impl IntoView {
    view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8" />
                <meta name="viewport" content="width=device-width, initial-scale=1" />
                <AutoReload options=options.clone() />
                <HydrationScripts options />
                <MetaTags />
            </head>
            <body>
                <App />
            </body>
        </html>
    }
}

#[component]
pub fn App() -> impl IntoView {
    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context();

    view! {
        <Stylesheet id="leptos" href="/pkg/leptos-loro-time-travel-demo.css" />

        // sets the document title
        <Title text="Welcome to Leptos + Loro" />

        // content for this welcome page
        <Router>
            <main>
                <Routes fallback=|| "Page not found.".into_view()>
                    <Route path=StaticSegment("") view=HomePage />
                </Routes>
            </main>
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

    let loro_doc_snapshot = Resource::new(|| (), |_| async move { get_loro_doc().await.unwrap() });

    view! {
        <h1>"Welcome to Leptos + Loro!"</h1>
        <div style="width: calc(100% - 32px); padding: 16px;">
            <Suspense fallback=|| {
                "Loading...".into_view()
            }>
                {move || Suspend::new(async move {
                    let snapshot = loro_doc_snapshot.await;
                    doc.update_value(|doc| {
                        doc.import(snapshot.as_slice()).unwrap();
                    });
                    let frontiers = doc.with_value(|doc| doc.state_frontiers());
                    let last = frontiers.last().cloned();
                    leptos::logging::log!("Last version: {:#?}", last);
                    last_loro_id.set(last);
                    max_version.set(last.map(|id| id.counter).unwrap_or(0));
                    view! {
                        <RangeSelect
                            version=version
                            max_version=max_version.into()
                            doc=doc
                            text=text
                            last_loro_id=last_loro_id.into()
                            checkout_time=checkout_time
                        />
                        <RenderText text=text.into() checkout_time=checkout_time.into() />
                    }
                })}
            </Suspense>
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
                doc.update_value(|doc| doc.checkout_to_latest());
                ts_end = leptos_use::use_timestamp().get();
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
        250.0,
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

#[server]
async fn get_loro_doc() -> Result<LoroDocSnapshot, ServerFnError> {
    // Sample data from https://github.com/josephg/editing-traces
    use serde::Deserialize;
    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct SequentialTrace {
        start_content: String,
        end_content: String,
        txns: Vec<Transaction>,
    }

    #[derive(Debug, Deserialize)]
    struct Transaction {
        time: String,
        patches: Vec<Patch>,
    }

    #[derive(Debug, Deserialize)]
    struct Patch {
        position: usize,
        num_deleted: usize,
        insert_content: String,
    }

    let seph_blog1_json_gz = include_bytes!("../public/seph-blog1.json.gz");

    use flate2::read::GzDecoder;
    let decoder = GzDecoder::new(seph_blog1_json_gz.as_slice());

    let seph_blog1_trace = serde_json::from_reader::<_, SequentialTrace>(decoder).unwrap();

    let doc = LoroDoc::new();
    let text = doc.get_text("text");
    text.insert(0, seph_blog1_trace.start_content.as_str())
        .unwrap();

    seph_blog1_trace.txns.iter().for_each(|txn| {
        _ = txn.time;
        txn.patches.iter().for_each(|patch| {
            text.splice(
                patch.position,
                patch.num_deleted,
                patch.insert_content.as_str(),
            )
            .unwrap();
        });
    });

    assert_eq!(text.to_string(), seph_blog1_trace.end_content);

    Ok(doc.export_snapshot())
}

type LoroDocSnapshot = Vec<u8>;
