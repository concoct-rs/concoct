use concoct::{html::div, use_signal, View};

fn app() -> impl View {
    let mut count = use_signal(|| 0);

    div().view((
        move || format!("High five count: {}", count),
        div().view("Up high!").on_click(move || count += 1),
        div().view("Down low!").on_click(move || count -= 1),
    ))
}

fn main() {
    console_error_panic_hook::set_once();
    dioxus_logger::init(log::LevelFilter::Info).expect("failed to init logger");
    concoct::run(app);
}
