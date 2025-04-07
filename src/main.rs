use leptos::ev::{keydown, KeyboardEvent};
use leptos::logging::log;
use leptos::prelude::*;
use leptos_use::{use_document, use_event_listener};
use web_time::Instant;

mod bpm;

fn main() {
    console_error_panic_hook::set_once();
    mount_to_body(App);
}

/// Helper function to determine if the supplied keycode should add another BPM tap
fn is_tap_key(key_code: &u32) -> bool {
    let disabled_keys = [
        0,  // Unidentified
        12, // Clear
        16, // Shift
        17, // Control
        18, // Alt
        20, // CapsLock
        27, // Escape
        91, // Meta
        92, // Meta
    ];
    !disabled_keys.contains(key_code)
}

fn display_bpm_signal(bpm: ReadSignal<Option<f64>>) -> String {
    match bpm.get() {
        Some(bpm) => format!("{bpm:.2}"),
        None => "Not Enough Data".into(),
    }
}

#[component]
fn App() -> impl IntoView {
    const RESET_SECS: u64 = 2;

    let (timestamps, set_timestamps) = signal::<Vec<Instant>>(Vec::new());
    let (direct_count, set_direct_count) = signal::<Option<f64>>(None);
    let (simple_regression, set_simple_regression) = signal::<Option<f64>>(None);
    let (thiel_sen, set_thiel_sen) = signal::<Option<f64>>(None);

    let mut last_tap = Instant::now();

    let _cleanup = use_event_listener(use_document(), keydown, move |evt: KeyboardEvent| {
        // log!("{evt:?}");
        let now = Instant::now();
        if is_tap_key(&evt.key_code()) {
            if last_tap.elapsed().as_secs() >= RESET_SECS {
                set_timestamps.set(vec![now]);
            } else {
                set_timestamps.write().push(now);
            }
            set_direct_count.set(
                bpm::direct_count(&timestamps.read())
                    .inspect_err(|e| log!("{e:?}"))
                    .ok(),
            );
            set_simple_regression.set(
                bpm::simple_regression(&timestamps.read())
                    .inspect_err(|e| log!("{e:?}"))
                    .ok(),
            );
            set_thiel_sen.set(
                bpm::thiel_sen(&timestamps.read())
                    .inspect_err(|e| log!("{e:?}"))
                    .ok(),
            );
        }
        last_tap = now;
    });

    view! {
        <main class="m-auto max-w-3xl text-center font-mono text-3xl">
            <p>"Total Beats: "{move || timestamps.get().len()}</p>
            <p>"Direct Count Average: "{move || display_bpm_signal(direct_count)}</p>
            <p>"Least Squares Estimate: "{move || display_bpm_signal(simple_regression)}</p>
            <p>"Thiel-Sen Estimate: "{move || display_bpm_signal(thiel_sen)}</p>
        </main>
    }
}
