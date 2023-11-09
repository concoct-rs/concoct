use concoct::{use_signal, Html, View};

pub fn app() -> impl View {
    let label = use_signal(|| String::new());

    Html::input()
        .attr("value", label.read().clone())
        .on_input(move |event| {
            event.prevent_default();
            *label.write() = event.target().unwrap().value();
        })
}
