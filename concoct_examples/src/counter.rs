use concoct::{use_signal, Html, View};

pub fn app() -> impl View {
    let mut count = use_signal(|| 0);

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
