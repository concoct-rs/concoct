use concoct::{Composable, Context};

fn app() -> impl Composable<i32> {
    concoct::from_fn(|cx| cx.send(())).map(|()| 2)
}

#[tokio::main]
async fn main() {
    let (mut cx, mut rx) = Context::new();
    let mut state = app().compose(&mut cx);
    app().recompose(&mut cx, &mut state);

    dbg!(rx.recv().await);
}
