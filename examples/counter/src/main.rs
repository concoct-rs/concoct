use concoct::{html::div, use_context, use_context_provider, use_signal, Scope, View};
use gloo_timers::callback::Interval;

fn app() -> impl View {
    let mut count = use_signal(|| 0);

    log::info!("{:?}", count.read());

    div().view(move || {
        (
            format!("High five count: {}", count),
            div()
                .view(String::from("Up high!"))
                .on_click(move || count += 1),
            div()
                .view(String::from("Down low!"))
                .on_click(move || count -= 1),
        )
    })
}

fn main() {
    console_error_panic_hook::set_once();
    dioxus_logger::init(log::LevelFilter::Info).expect("failed to init logger");
    concoct::run(app);
}
