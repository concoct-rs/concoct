use concoct::slot_table::SlotTable;
use std::any::TypeId;

#[test]
fn it_is_empty() {
    let slots = SlotTable::default();
    assert!(slots.is_empty());

    let slots = slots.write(|writer| {
        writer.begin_insert();
        writer.start_group(TypeId::of::<()>(), None);
        writer.end_group();
        writer.end_insert();
    });
    assert!(!slots.is_empty());
}

#[test]
fn it_can_insert() {
    let slots = SlotTable::default();
    slots.write(|writer| {
        writer.begin_insert();
        writer.start_group(TypeId::of::<()>(), None);
        writer.update(Some(Box::new(1)));
        writer.end_group();
        writer.end_insert();
    });
}
