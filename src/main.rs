use leptos::ev::{keydown, keyup, KeyboardEvent};
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
    /// Returns true if the bpm count has been reset
    pub fn is_reset(&self) -> bool {
        self.start.is_none() && !self.timestamps.is_empty()
    }
}

#[derive(Debug, Clone)]
enum BlinkColor {
    Orange,
    Violet,
}

impl BlinkColor {
    pub fn tw_class(&self) -> &str {
        match &self {
            Self::Orange => "border-orange-400",
            Self::Violet => "border-violet-400",
        }
    }
}

#[component]
fn App() -> impl IntoView {
    let (reset_sec, set_reset_sec) = signal::<u64>(2);
    let (border_state, set_border_state) = signal::<Option<BlinkColor>>(None);
    let (tap_data, set_tap_data) = signal::<TapData>(TapData::default());
    let (active_timeout, set_active_timeout) = signal::<Option<TimeoutHandle>>(None);
    let (ctrl_held, set_ctrl_held) = signal::<bool>(false);

    let blink_border = move |color: BlinkColor| {
        set_border_state.set(Some(color));
        set_timeout(
            move || set_border_state.set(None),
            Duration::from_millis(50),
        );
    };

    let handle_beat_input = move || {
        let now = Instant::now();
        if let Some(handle) = active_timeout.get() {
            handle.clear();
        }
        let new_timeout = set_timeout_with_handle(
            move || {
                set_tap_data.write().start = None;
                blink_border(BlinkColor::Orange);
            },
            Duration::from_secs(reset_sec.get()),
        )
        .expect("Set timeout should not fail");
        set_active_timeout.set(Some(new_timeout));
        set_tap_data.write().record(now);
        blink_border(BlinkColor::Violet);
    };

    let _cleanup = use_event_listener(use_document(), keyup, move |evt: KeyboardEvent| {
        // Ctrl is released
        if evt.key_code() == 17 {
            set_ctrl_held.set(false);
        }
    });

    let _cleanup = use_event_listener(use_document(), keydown, move |evt: KeyboardEvent| {
        let disabled_keys = [
            0,  // Unidentified
            9,  // Tab
            12, // Clear
            16, // Shift
            18, // Alt
            20, // CapsLock
            27, // Escape
            91, // Meta
            92, // Meta
        ];
        // Ctrl is pressed
        if evt.key_code() == 17 {
            set_ctrl_held.set(true);
        } else if !disabled_keys.contains(&evt.key_code()) && !ctrl_held.get() {
            handle_beat_input();
        }
    });

    view! {
        <div class="flex flex-col h-screen" on:mousedown={move |_| handle_beat_input()}>
            <div class="flex justify-center items-center min-h-screen bg-zinc-800 select-none w-full h-full">
                <pre
                    // set border color according to border_state
                    class={move || {
                        let pre_class = concat!(
                            "font-mono bg-zinc-800 text-white select-text",
                            "   border-[0.5vw]    px-[3.2vw]    py-[2.5vw]    text-[3.0vw] ",
                            "xl:border-[0.3vw] xl:px-[1.7vw] xl:py-[1.3vw] xl:text-[1.6vw] ",
                        );
                        match border_state.get() {
                            Some(blink_color) => format!("{pre_class} {}", blink_color.tw_class()),
                            None => format!("{pre_class} border-white transition-colors duration-400"),
                        }
                    }}
                    // prevent clicks in the ui from triggering a beat update
                    on:mousedown={move |e| e.stop_propagation()}
                >
                    <span>"lucdar's bpm counter""\n\n"</span>
                    <ResetControl reset_sec set_reset_sec />
                    <BpmTable tap_data />
                    <Footer tap_data />
                </pre>
            </div>
        </div>
    }
}

#[component]
fn ResetControl(reset_sec: ReadSignal<u64>, set_reset_sec: WriteSignal<u64>) -> impl IntoView {
    view! {
        <span class="text-green-400">"   reset-sec:  "</span>
        <button class="hover:text-violet-400" on:mousedown={move |_| {
            if reset_sec.get() < 9 {
                *set_reset_sec.write() += 1;
            }
        }}>"↑"</button>
        <span class="text-violet-400">" "{move || reset_sec.get()}" "</span>
        <button class="hover:text-violet-400" on:mousedown={move |_| {
            if reset_sec.get() > 1 {
                *set_reset_sec.write() -= 1;
            }
        }}>"↓"</button>
        <span class="text-zinc-400">" # secs before bpm is reset\n\n"</span>
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
                <span class="text-violet-400">
                    {move || {
                        match $algorithm(&tap_data.read().timestamps)
                            // .inspect_err(|e| log!("{e:?}"))
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
    let link_class = "hover:text-violet-400 transition-colors duration-150";
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
