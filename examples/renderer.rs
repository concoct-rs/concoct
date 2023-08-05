use concoct::render::{
    element::{Canvas, Group},
    renderer::Renderer,
    Tree,
};
use skia_safe::{Color4f, Paint};
use taffy::{prelude::Size, style::Style};

fn main() {
    let mut tree = Tree::default();

    let a = {
        let mut elem = Canvas::new(Box::new(|_taffy, canvas| {
            canvas.draw_circle(
                (50., 50.),
                50.,
                &Paint::new(Color4f::new(0., 1., 0., 1.), None),
            );
        }));
        elem.style.size = Size::from_points(100., 100.);
        tree.insert(Box::new(elem))
    };

    let b = {
        let mut elem = Canvas::new(Box::new(|_taffy, canvas| {
            canvas.draw_circle(
                (50., 50.),
                50.,
                &Paint::new(Color4f::new(0., 0., 1., 1.), None),
            );
        }));
        elem.style.size = Size::from_points(100., 100.);
        tree.insert(Box::new(elem))
    };

    let root = {
        let mut elem = Group::new(Style::default(), vec![a, b]);
        elem.style.size = Size::from_points(1000., 1000.);
        tree.insert(Box::new(elem))
    };

    let app = Renderer::new(tree, root);
    app.run(|event| {
        dbg!(event);
    });
}
