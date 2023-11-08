use concoct::{use_signal, Html, View};

fn button(label: impl View + 'static, mut on_click: impl FnMut() + 'static) -> impl View {
    Html::button().on_click(move |_| on_click()).view(label)
}

fn app() -> impl View {
    let selection = use_signal(|| "A");

    Html::div().view((
        move || button("A", move || *selection.write() = "A"),
        move || button("B", move || *selection.write() = "B"),
        move || button("C", move || *selection.write() = "C"),
    ))
}

fn main() {
    console_error_panic_hook::set_once();
    dioxus_logger::init(log::LevelFilter::Info).expect("failed to init logger");
    concoct::run(app);
}
