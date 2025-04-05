use leptos::ev::{keydown, KeyboardEvent};
use leptos::logging::log;
use leptos::prelude::*;
use leptos_use::{use_document, use_event_listener};
use web_time::{Duration, Instant};

fn main() {
    console_error_panic_hook::set_once();
    mount_to_body(App);
}

/// Shows progres towards a goal
#[component]
fn ProgressBar(
    /// The maximum value of the progress bar
    #[prop(default = 100)]
    max: u16,
    /// How much progress should be displayed
    progress: ReadSignal<u32>,
) -> impl IntoView {
    view! {
        <progress
            max=max
            value=move || progress.get()
        />
        // Add a line-break to avoid overlap
        <br/>
    }
}

/// Helper function to determine if the supplied keycode should add another BPM tap
fn is_tap_key(key_code: &u32) -> bool {
    // List of disabled keys
    ![
        0,  // Unidentified
        12, // Clear
        16, // Shift
        17, // Control
        18, // Alt
        20, // CapsLock
        27, // Escape
        91, // Meta
        92, // Meta
    ]
    .contains(key_code)
}

#[component]
fn App() -> impl IntoView {
    const RESET_SECS: u64 = 2;

    let (count, set_count) = signal(0);
    let (first_tap, set_first_tap) = signal(Instant::now());
    let (bpm_avg, set_bpm_avg) = signal::<Option<f64>>(None);

    let mut last_tap = Instant::now();

    let _cleanup = use_event_listener(use_document(), keydown, move |evt: KeyboardEvent| {
        log!("{evt:?}");
        if is_tap_key(&evt.key_code()) {
            // Reset count if RESET_SECS elapsed
            if last_tap.elapsed().as_secs() > RESET_SECS {
                set_count.set(0);
            }
            if count.get() == 0 {
                // First Beat - reset state
                set_first_tap.set(Instant::now());
                set_bpm_avg.set(None);
            } else {
                set_bpm_avg.set(Some(
                    (60000 * count.get()) as f64 / first_tap.get().elapsed().as_millis() as f64,
                ));
            }
            *set_count.write() += 1;
            last_tap = Instant::now();
        }
    });

    view! {
        <p>Total Beats: {move || count.get()}</p>
        <p>Average BPM: {move || bpm_avg.get()}</p>
    }
}
