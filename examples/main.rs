use concoct::render::{
    element::{Canvas, Group},
    renderer::run,
    Tree,
};
use skia_safe::{Color4f, Paint};
use taffy::{prelude::Size, style::Style};

fn main() {
    let mut tree = Tree::default();

    let mut elem = Canvas::new(Box::new(|_taffy, canvas| {
        canvas.draw_circle(
            (50., 50.),
            50.,
            &Paint::new(Color4f::new(0., 1., 0., 1.), None),
        );
    }));
    elem.style.size = Size::from_points(100., 100.);
    let a = tree.insert(Box::new(elem));

    let mut elem = Canvas::new(Box::new(|_taffy, canvas| {
        canvas.draw_circle(
            (50., 50.),
            50.,
            &Paint::new(Color4f::new(0., 0., 1., 1.), None),
        );
    }));
    elem.style.size = Size::from_points(100., 100.);
    let b = tree.insert(Box::new(elem));

    let mut elem = Group::new(Style::default(), vec![a, b]);
    elem.style.size = Size::from_points(1000., 1000.);
    let root = tree.insert(Box::new(elem));

    run(tree, root);
}
