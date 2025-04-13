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

#[component]
fn App() -> impl IntoView {
    const RESET_SECS: u64 = 2;

    let (tap_data, set_tap_data) = signal::<TapData>(TapData::default());
    let (active_timeout, set_active_timeout) = signal::<Option<TimeoutHandle>>(None);

    let handle_beat_input = move || {
        let now = Instant::now();
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
    };

    let _cleanup = use_event_listener(use_document(), keydown, move |evt: KeyboardEvent| {
        let disabled_keys = [
            0,  // Unidentified
            9,  // Tab
            12, // Clear
            16, // Shift
            17, // Control
            18, // Alt
            20, // CapsLock
            27, // Escape
            91, // Meta
            92, // Meta
        ];
        if !disabled_keys.contains(&evt.key_code()) {
            handle_beat_input();
        }
    });

    view! {
        <div class="flex flex-col h-screen select-none" on:click={move |_| handle_beat_input()}>
            <div class="flex justify-center items-center min-h-screen">
                <pre class="font-mono bg-zinc-800 select-text text-white p-5" on:click={move |e| {
                    e.prevent_default();
                    e.stop_propagation();
                }}>
                    <Header />
                    <BpmTable tap_data />
                    <Footer />
                </pre>
            </div>
        </div>
    }
}

#[component]
fn BpmTable(tap_data: ReadSignal<TapData>) -> impl IntoView {
    // creates a <p> with the bpm calculated by the suppled function, labeled with the supplied label
    macro_rules! render_bpm_metric {
        ($label:expr, $algorithm:expr) => {
            view! {
                <span class="text-green-400">{format!("{:>12}: ", $label)}</span>
                <span class="text-violet-400">
                    {move || {
                        match $algorithm(&tap_data.read().timestamps)
                            .inspect_err(|e| log!("{e:?}"))
                            .ok()
                        {
                            Some(bpm) => format!("{bpm:6.2}\n"),
                            None => "000.00\n".into(),
                        }
                    }}
                </span>
                <span class="text-zinc-400"></span>
            }
        };
    }

    view! {
        <span>
            {format!("{:>12}: ", "n")}
            {move || format!("{:6.2}\n", tap_data.read().timestamps.len())}
        </span>
        {render_bpm_metric!("direct", bpm::direct_count)}
        {render_bpm_metric!("lin-reg", bpm::simple_regression)}
        {render_bpm_metric!("thiel-sen", bpm::thiel_sen)}
    }
}

#[component]
fn Header() -> impl IntoView {
    // TODO: make this
    view! {<span>"lucdar's bpm counter\n\n"</span>}
}

#[component]
fn Footer() -> impl IntoView {
    // TODO: make this
    view! {<span>
        "\n                                     "
        "blog"
        " | "
        "source"
    </span>}
}
