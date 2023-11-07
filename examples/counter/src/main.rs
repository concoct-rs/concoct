use concoct::{html::Div, Signal, View};
use gloo_timers::callback::Interval;
use std::mem;

fn app() -> impl View {
    let count = Signal::new(0);

    log::info!("app");

    mem::forget(Interval::new(500, move || {
        log::info!("timer: {}", count.read());
        *count.write() += 1;
    }));

    Div::new().child(move || {
        log::info!("child");
        format!("High five count: {}", count.read())
    })
}

fn main() {
    console_error_panic_hook::set_once();
    dioxus_logger::init(log::LevelFilter::Info).expect("failed to init logger");
    concoct::run(app);
}
