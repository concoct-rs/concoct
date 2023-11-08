use concoct::{use_signal, Html, View};

fn app() -> impl View {
    let mut count = use_signal(|| 0);

    log::info!("{:?}", count.read());

    Html::div().view((
        move || format!("High five count: {}", count),
        Html::button()
            .on_click(move |_| count += 1)
            .view("Up high!"),
        Html::button()
            .on_click(move |_| count -= 1)
            .view("Down low!"),
    ))
}

fn main() {
    console_error_panic_hook::set_once();
    dioxus_logger::init(log::LevelFilter::Info).expect("failed to init logger");
    concoct::run(app);
}
