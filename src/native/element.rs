use taffy::Taffy;

pub trait Element {
    fn paint(&mut self, taffy: &Taffy, canvas: &mut skia_safe::Canvas);
}
