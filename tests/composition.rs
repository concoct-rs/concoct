use concoct::{Composable, Composition};

#[test]
fn it_reuses_scopes_for_tuples() {
    fn app() -> impl Composable {
        ((), ())
    }

    let mut composition = Composition::new(app);
    composition.build();
    assert_eq!(composition.len(), 1);
}

#[test]
fn it_creates_new_scopes_for_multiple_composables() {
    #[derive(Clone, PartialEq)]
    struct Component;
    impl Composable for Component {
        fn compose(&mut self) {}
    }

    fn app() -> impl Composable {
        (Component, Component)
    }

    let mut composition = Composition::new(app);
    composition.build();
    assert_eq!(composition.len(), 3);
}
