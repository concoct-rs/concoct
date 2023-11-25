use concoct::{Composable, Composition, IntoComposable};

#[test]
fn it_creates_new_scopes_for_multiple_composables() {
    #[derive(Clone, PartialEq)]
    struct Component;
    impl Composable for Component {
        fn compose(&mut self) {}
    }

    fn app() -> impl IntoComposable {
        (Component, Component)
    }

    let mut composition = Composition::new(app);
    composition.build();
    assert_eq!(composition.len(), 3);
}
