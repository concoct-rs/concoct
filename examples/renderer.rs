use concoct::{element::Canvas, Renderer, Tree};
use futures_signals::signal::Mutable;
use skia_safe::{Color4f, Paint};
use std::time::Duration;
use taffy::prelude::Size;

#[tokio::main]
async fn main() {
    let mut tree = Tree::default();

    let is_red = Mutable::new(false);

    let a = {
        let mut elem = Canvas::new(is_red.read_only(), |is_red, _layout, canvas| {
            let color = if is_red {
                Color4f::new(1., 0., 0., 1.)
            } else {
                Color4f::new(0., 1., 0., 1.)
            };

            canvas.draw_circle((50., 50.), 50., &Paint::new(color, None));
        });
        elem.style.size = Size::from_points(100., 100.);
        tree.insert(Box::new(elem))
    };

    tokio::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_secs(1)).await;
            is_red.set(!is_red.get());
        }
    });

    let app = Renderer::new(tree, a);
    app.run(|_tree, event| {
        dbg!(event);
    });
}
