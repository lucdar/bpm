use leptos::ev::{keydown, KeyboardEvent};
use leptos::logging::log;
use leptos::prelude::*;
use leptos::wasm_bindgen::JsCast;
use leptos::web_sys::Element;
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
    /// Returns true if the bpm count has been reset
    pub fn is_reset(&self) -> bool {
        self.start.is_none() && !self.timestamps.is_empty()
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
        <div class="flex flex-col h-screen" on:click={move |_| handle_beat_input()}>
            <div class="flex justify-center items-center min-h-screen bg-zinc-800">
                <pre
                    // change border color if reset has happened
                    class={move || {
                        let pre_class = "font-mono bg-zinc-800 border-2 select-text text-white p-5";
                        if tap_data.read().is_reset() {
                            format!("{pre_class} border-orange-400")
                        } else {
                            format!("{pre_class} border-white")
                        }
                    }}
                    on:click={move |e| {
                        // prevent from triggering bpm click
                        e.stop_propagation();
                        // if target is not a button or link, prevent default (selecting text)
                        if let Some(target) = e.target() {
                            if let Some(el) = target.dyn_ref::<Element>() {
                                let tag_name = el.tag_name().to_lowercase();
                                if tag_name != "a" && tag_name != "button" {
                                    e.prevent_default();
                                }
                            }
                        }
                    }}
                >
                    <span>"lucdar's bpm counter""\n\n"</span>
                    <BpmTable tap_data />
                    <Footer tap_data />
                </pre>
            </div>
        </div>
    }
}

#[component]
fn BpmTable(tap_data: ReadSignal<TapData>) -> impl IntoView {
    // creates a row with formatted calculations
    macro_rules! render_bpm_metric {
        ($label:expr, $algorithm:expr, $description:expr) => {
            view! {
                // align and color the label
                <span class="text-green-400">
                    {format!("{:>12}: ", $label)}
                </span>
                // conditionally color the measurement
                <span class="text-violet-400">
                    {move || {
                        match $algorithm(&tap_data.read().timestamps)
                            .inspect_err(|e| log!("{e:?}"))
                            .ok()
                        {
                            Some(bpm) => format!("{bpm:6.2} "),
                            None => "000.00 ".into(),
                        }
                    }}
                </span>
                <span class="text-zinc-400">"# "{$description}"\n"</span>
            }
        };
    }

    fn slice_len(ts: &[u64]) -> Result<u64, bpm::BpmCalculationError> {
        Ok(ts.len() as u64)
    }

    view! {
        {render_bpm_metric!("n", slice_len, "the total count of beats")}
        {render_bpm_metric!("direct", bpm::direct_count, "n - 1 divided by delta t")}
        {render_bpm_metric!("lin-reg", bpm::simple_regression, "simple linear regression")}
        {render_bpm_metric!("thiel-sen", bpm::thiel_sen, "the \"median\" of the bpms")}
    }
}

#[component]
fn Footer(tap_data: ReadSignal<TapData>) -> impl IntoView {
    let link_class = "hover:text-violet-400 transition-all duration-150";
    view! {<span>
        "\n"
        <span class="text-orange-400">
            {move ||
                if tap_data.read().is_reset() {
                    "reset!"
                } else {
                    "      "
                }
            }
        </span>
        {" ".repeat(31)}
        <a href="https://laclark.me/blog/bpm/" class={link_class}>blog</a>
        " | "
        <a href="https://github.com/lucdar/bpm-leptos/" class={link_class}>source</a>
    </span>}
}
