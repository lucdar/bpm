use leptos::prelude::*;

fn main() {
    console_error_panic_hook::set_once();
    mount_to_body(App);
}

#[component]
fn App() -> impl IntoView {
    let (count, set_count) = signal(0);

    let double_count = move || count.get() * 2;

    view! {
        <button
            on:click=move |_| {
                *set_count.write() += 1
            }
            class:red=move || count.get() % 2 == 1
            style="position: absolute"
            style:left=move || format!("{}px", double_count() + 100)
            style:background-color=move || format!("rgb({}, {}, 100)", count.get(), 100)
            style:max-width="400px"
            style=("--columns", move || count.get().to_string())
        >
            "Click me: "
            {double_count}
        </button>
        <ProgressBar progress=count/>
    }
}

#[component]
fn ProgressBar(progress: ReadSignal<i32>) -> impl IntoView {
    view! {
        <progress
            max="50"
            value=progress
        />
    }
}
