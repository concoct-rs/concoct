use concoct::{use_hook, AnyComposable, Composable, Composition};
use std::any::Any;

struct Text {}

impl Composable for Text {
    type State = ();

    fn build(&mut self, cx: &mut concoct::BuildContext) -> Self::State {}

    fn rebuild(&mut self, state: &mut Self::State) {}
}

fn counter() -> impl Composable {
    let count = use_hook(|| 0);

    dbg!(count.borrow().downcast_ref::<i32>());

    *count.borrow_mut().downcast_mut::<i32>().unwrap() += 1;

    Text {}
}

fn app() -> impl Composable {
    (counter, counter)
}

fn main() {
    let mut composition = Composition::new(app);
    composition.build();
    composition.rebuild();
}
