use concoct::{html::div, use_signal, Scope, View};
use gloo_timers::callback::Interval;

fn app() -> impl View {
    let count = use_signal(|| 0);

    Scope::current().use_hook(|| Interval::new(500, move || *count.write() += 1));

    div().view(move || {
        (
            format!("High five count: {}", count.read()),
            div()
                .on_click(move || *count.write() += 1)
                .view(|| "Up high!"),
            div()
                .on_click(move || *count.write() -= 1)
                .view(|| "Down low!"),
        )
    })
}

fn main() {
    console_error_panic_hook::set_once();
    dioxus_logger::init(log::LevelFilter::Info).expect("failed to init logger");
    concoct::run(app);
}
