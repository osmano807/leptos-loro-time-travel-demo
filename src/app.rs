use core::f64;

use leptos::prelude::*;
use leptos_meta::*;
use leptos_router::{
    components::{Route, Router, Routes},
    StaticSegment,
};
use loro::LoroDoc;
use std::ops::ControlFlow;

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    view! {
        <Title text="Welcome to Leptos + Loro" />

        <Router>
            <Routes fallback=|| "Page not found.">
                <Route path=StaticSegment("") view=HomePage />
                <Route path=StaticSegment("leptos-loro-time-travel-demo") view=HomePage />
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

    let checkout_stats = RwSignal::new(CheckoutStats::new());

    let text = RwSignal::new(String::new());

    // See the README for instructions on how to generate the snapshot
    let snapshot = include_bytes!("../public/seph-blog1.loro-snapshot");

    doc.update_value(|doc| {
        doc.import(snapshot.as_slice()).unwrap();
    });

    let last_frontiers = doc.with_value(|doc| doc.state_frontiers());

    let mut frontiers = vec![];
    doc.with_value(|doc| {
        doc.travel_change_ancestors(last_frontiers[0], &mut |meta| {
            // For all inner changes
            for counter in (meta.id.counter..meta.id.counter + meta.len as i32).rev() {
                frontiers.push(loro::ID::new(meta.id.peer, counter));
            }
            // For changes only
            /* frontiers
                .push(loro::ID::new(meta.id.peer, meta.id.counter + meta.len as i32 - 1)); */
            ControlFlow::Continue(())
        })
    });
    frontiers.reverse();

    max_version.set(frontiers.len() as i32 - 1); // 0 based index

    let frontiers = StoredValue::new(frontiers);

    view! {
        <h1>"Welcome to Leptos + Loro!"</h1>
        <div style="width: calc(100% - 32px); padding: 16px;">
            <RangeSelect
                version=version
                max_version=max_version.into()
                doc=doc
                text=text
                frontiers=frontiers
                checkout_stats=checkout_stats
            />
            <RenderText text=text.into() checkout_stats=checkout_stats.into() />
        </div>
    }
}

#[component]
fn RangeSelect(
    version: RwSignal<i32>,
    max_version: Signal<i32>,
    doc: StoredValue<LoroDoc>,
    text: RwSignal<String>,
    frontiers: StoredValue<Vec<loro::ID>>,
    checkout_stats: RwSignal<CheckoutStats>,
) -> impl IntoView {
    let range_on_input = leptos_use::use_throttle_fn_with_arg(
        move |ev: leptos::ev::Event| {
            let new_value = event_target_value(&ev).parse::<i32>().unwrap();
            version.set(new_value);

            let ts_start: f64;

            let ts_end: f64;

            if new_value == -1 {
                ts_start = performance_now();
                doc.update_value(|doc| doc.checkout(&[].into()));
                ts_end = performance_now();
                text.set("".to_string());
            } else {
                let frontier = frontiers.with_value(|frontiers| frontiers[new_value as usize]);
                ts_start = performance_now();
                doc.update_value(|doc| doc.checkout(&frontier.into()));
                ts_end = performance_now();
                // As in the original example, we don't count the time it takes to read from the LoroDoc
                text.set(doc.with_value(|doc| doc.get_text("text").to_string()));
            }

            checkout_stats.update(|checkout_stats| {
                checkout_stats.add(ts_end - ts_start);
            });
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
fn RenderText(text: Signal<String>, checkout_stats: Signal<CheckoutStats>) -> impl IntoView {
    view! {
        <div style="display: flex; justify-content: space-between; font-family: monospace;">
            <span style="margin-right: 2em;">
                "Checkout duration: "
                {move || {
                    format!(
                        "{:.2}",
                        checkout_stats.with(|checkout_stats| checkout_stats.checkout_time.last()),
                    )
                }} " ms"
            </span>
            <span>
                "Min: "
                {move || {
                    format!("{:.2}", checkout_stats.with(|checkout_stats| checkout_stats.min.min()))
                }} " ms"
            </span>
            <span>
                "Max: "
                {move || {
                    format!("{:.2}", checkout_stats.with(|checkout_stats| checkout_stats.max.max()))
                }} " ms"
            </span>
            <span>
                "Mean: "
                {move || {
                    format!(
                        "{:.2}",
                        checkout_stats.with(|checkout_stats| checkout_stats.variance.mean()),
                    )
                }} " ± " {move || {
                    format!(
                        "{:.2}",
                        checkout_stats.with(|checkout_stats| checkout_stats.variance.error()),
                    )
                }} " ms"
            </span>
            <span>
                "Variance: "
                {move || {
                    format!(
                        "{:.2}",
                        checkout_stats
                            .with(|checkout_stats| checkout_stats.variance.sample_variance()),
                    )
                }} " ms²"
            </span>
            <span>
                "Standard Deviation: "
                {move || {
                    format!(
                        "{:.2}",
                        checkout_stats
                            .with(|checkout_stats| checkout_stats.variance.sample_variance().sqrt()),
                    )
                }} " ms"
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

use average::{concatenate, Estimate, Max, Min, Variance};

struct CheckoutTime(f64);

impl CheckoutTime {
    fn new() -> Self {
        Self(f64::INFINITY)
    }
    fn add(&mut self, value: f64) {
        self.0 = value;
    }
    fn last(&self) -> f64 {
        self.0
    }
}
impl Default for CheckoutTime {
    fn default() -> Self {
        Self::new()
    }
}

concatenate!(
    CheckoutStats,
    [CheckoutTime, checkout_time, last],
    [Variance, variance, mean, error, sample_variance],
    [Min, min, min],
    [Max, max, max]
);

fn performance_now() -> f64 {
    web_sys::window()
        .expect("should have a Window")
        .performance()
        .expect("should have a Performance")
        .now()
}
