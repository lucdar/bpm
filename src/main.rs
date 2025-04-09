use leptos::ev::{keydown, KeyboardEvent};
use leptos::logging::log;
use leptos::prelude::*;
use leptos_use::{use_document, use_event_listener};
use web_time::{Duration, Instant};

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

#[component]
fn App() -> impl IntoView {
    const RESET_SECS: u64 = 2;

    let (timestamps, set_timestamps) = signal::<Vec<Instant>>(Vec::new());
    let (active_timeout, set_active_timeout) = signal::<Option<TimeoutHandle>>(None);

    let _cleanup = use_event_listener(use_document(), keydown, move |evt: KeyboardEvent| {
        let now = Instant::now();
        if !is_tap_key(&evt.key_code()) {
            return;
        }
        match active_timeout.get() {
            Some(handle) => handle.clear(),
            // clear timestamps if no active timeout
            None => set_timestamps.set(Vec::new()),
        }
        let new_timeout = set_timeout_with_handle(
            move || {
                set_active_timeout.set(None);
            },
            Duration::from_secs(RESET_SECS),
        )
        .expect("Set timeout should not fail");
        set_active_timeout.set(Some(new_timeout));
        set_timestamps.write().push(now);
    });

    view! {
        <BpmTable timestamps />
    }
}

#[component]
fn BpmTable(timestamps: ReadSignal<Vec<Instant>>) -> impl IntoView {
    // creates a <p> with the bpm calculated by the suppled function, labeled with the supplied label
    macro_rules! render_bpm_metric {
        ($label:expr, $algorithm:expr) => {
            view! {
                <p>{$label}": "{move || {
                    match $algorithm(&timestamps.read())
                        .inspect_err(|e| log!("{e:?}"))
                        .ok()
                    {
                        Some(bpm) => format!("{bpm:.2}"),
                        None => "Not Enough Data".into(),
                    }
                }}</p>
            }
        };
    }

    view! {
        <div class="m-auto max-w-3xl text-center font-mono text-3xl">
            <p>"Total Beats: "{move || timestamps.get().len()}</p>
            {render_bpm_metric!("Direct Count Average: ", bpm::direct_count)}
            {render_bpm_metric!("Least Squares Estimate: ", bpm::simple_regression)}
            {render_bpm_metric!("Thiel-Sen Estimate: ", bpm::thiel_sen)}
        </div>
    }
}
