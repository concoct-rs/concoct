use concoct::slot_table::SlotTable;
use std::any::TypeId;

#[test]
fn it_is_empty() {
    let table = SlotTable::default();
    assert!(table.is_empty());

    let table = table.write(|writer| {
        writer.begin_insert();

        writer.start_group(TypeId::of::<()>(), None);
        writer.end_group();

        writer.end_insert();
    });

    assert!(!table.is_empty());
}
