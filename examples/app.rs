use concoct::{use_state, Composable, Composition, Debugger};

fn counter() -> impl Composable {
    let (count, set_count) = use_state(|| 0);

    set_count(count.cloned() + 1);

    Debugger::new(count)
}

fn main() {
    let mut composition = Composition::new(counter);
    composition.build();
    composition.rebuild();
}
