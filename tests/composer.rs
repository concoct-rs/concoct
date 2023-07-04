use concoct::{composable, compose, composer::SlotKind, remember, Composer};

#[test]
fn it_inserts_a_group() {
    #[composable]
    fn app() {}

    let mut composer = Composer::default();
    composer.compose(app());

    let slots: Vec<_> = composer.slots().map(|slot| slot.kind()).collect();
    assert_eq!(slots, [SlotKind::RestartGroup]);
}

#[test]
fn it_grows_slot_table() {
    #[composable]
    fn inner() {}

    #[composable]
    fn app() {
        compose!(inner());
    }

    let mut composer = Composer::with_capacity(Box::new(()), 1);
    composer.compose(app());

    let slots: Vec<_> = composer.slot_kinds().collect();
    assert_eq!(slots, [SlotKind::RestartGroup, SlotKind::RestartGroup]);
}

#[test]
fn it_composes_remember() {
    #[composable]
    fn app() {
        compose!(remember(|| {}));
    }

    let mut composer = Composer::default();
    composer.compose(app());

    let slots: Vec<_> = composer.slot_kinds().collect();
    assert_eq!(
        slots,
        [
            SlotKind::RestartGroup,
            SlotKind::ReplaceGroup,
            SlotKind::Data
        ]
    );
}
