use concoct::{html::div, use_context, use_context_provider, use_signal, Scope, View};
use gloo_timers::callback::Interval;

fn app() -> impl View {
    let count = use_signal(|| 0);

    Scope::current().use_hook(|| Interval::new(500, move || *count.write() += 1));

    use_context_provider(|| 0);

    div().view(move || {
        (
            format!("High five count: {}", count),
            div().view(String::from("Up high!")),
            div().view(String::from("Down low!"))
        )
       
    })
}

fn main() {
    console_error_panic_hook::set_once();
    dioxus_logger::init(log::LevelFilter::Info).expect("failed to init logger");
    concoct::run(app);
}
