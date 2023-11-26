use concoct::{native::WebView, use_state, webview::div, IntoView, View};

#[derive(PartialEq)]
struct App;

impl View for App {
    fn view(&mut self) -> impl IntoView {
        ("Native text", WebView::new("WebView text"))
    }
}

fn main() {
    concoct::native::run(App)
}
