use concoct::{composable, compose, composer::SlotKind, remember, Composer};

#[test]
fn it_inserts_a_group() {
    #[composable]
    fn app() {}

    let mut composer = Composer::default();
    composer.compose(app());

    assert!(composer.slots().eq([SlotKind::RestartGroup]));
}

#[test]
fn it_composes_remember() {
    #[composable]
    fn app() {
        compose!(remember(|| {}));
    }

    let mut composer = Composer::default();
    composer.compose(app());

    assert!(composer.slots().eq([
        SlotKind::RestartGroup,
        SlotKind::ReplaceGroup,
        SlotKind::Node
    ]));
}
