use web_sys::Element;

pub trait View {
    fn view(&mut self) -> Node;

    fn remove(&mut self);
}

pub enum Node{
    Component(fn() -> Box<dyn View>),
    Element(Element),
}

pub fn run<V: View>(view: fn() -> V) {
    let mut stack: Vec<Box<dyn View>> = vec![Box::new(view())];
    while let Some(mut view) = stack.pop() {
        let node = view.view();
        match node {
            Node::Component(component) => stack.push(component()),
            Node::Element(elem) => {
                dbg!("elem");
            }
        }
    }
}

pub struct Div;

impl View for Div {
    fn view(&mut self) -> Node {
        todo!()
    }

    fn remove(&mut self) {
        todo!()
    }
}