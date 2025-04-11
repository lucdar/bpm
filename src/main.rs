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

#[derive(Default)]
struct TapData {
    start: Option<Instant>,
    timestamps: Vec<u64>,
}

impl TapData {
    /// Appends a new tap datapoint to the end of the vector
    /// If self.start is None, records `now` as the start and resets timestamps
    pub fn record(&mut self, now: Instant) {
        match self.start {
            Some(start) => self
                .timestamps
                .push(now.duration_since(start).as_millis() as u64),
            None => {
                self.start = Some(now);
                self.timestamps = vec![0];
            }
        }
    }
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

    let (tap_data, set_tap_data) = signal::<TapData>(TapData::default());
    let (active_timeout, set_active_timeout) = signal::<Option<TimeoutHandle>>(None);

    let _cleanup = use_event_listener(use_document(), keydown, move |evt: KeyboardEvent| {
        let now = Instant::now();
        if !is_tap_key(&evt.key_code()) {
            return;
        }
        if let Some(handle) = active_timeout.get() {
            handle.clear();
        }
        let new_timeout = set_timeout_with_handle(
            move || {
                set_tap_data.write().start = None;
            },
            Duration::from_secs(RESET_SECS),
        )
        .expect("Set timeout should not fail");
        set_active_timeout.set(Some(new_timeout));
        set_tap_data.write().record(now);
    });

    view! {
        <BpmTable tap_data />
    }
}

#[component]
fn BpmTable(tap_data: ReadSignal<TapData>) -> impl IntoView {
    // creates a <p> with the bpm calculated by the suppled function, labeled with the supplied label
    macro_rules! render_bpm_metric {
        ($label:expr, $algorithm:expr) => {
            view! {
                <p>{$label}": "{move || {
                    match $algorithm(&tap_data.read().timestamps)
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
            <p>"Total Beats: "{move || tap_data.read().timestamps.len()}</p>
            {render_bpm_metric!("Direct Count Average: ", bpm::direct_count)}
            {render_bpm_metric!("Least Squares Estimate: ", bpm::simple_regression)}
            {render_bpm_metric!("Thiel-Sen Estimate: ", bpm::thiel_sen)}
        </div>
    }
}
