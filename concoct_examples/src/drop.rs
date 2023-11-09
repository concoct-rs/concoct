use concoct::{use_on_drop, use_signal, Html, View};

pub fn app() -> impl View {
    let mut count = use_signal(|| 0);

    Html::div().view((
        move || format!("High five count: {}", count),
        Html::button()
            .on_click(move |_| count += 1)
            .view("Up high!"),
        if *count.read() % 2 == 0 {
            Some(|| use_on_drop(|| log::info!("Dropped!")))
        } else {
            None
        },
    ))
}
