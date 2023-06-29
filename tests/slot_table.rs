use std::any::TypeId;

use concoct::slot_table::SlotTable;

#[test]
fn it_is_empty() {
    let mut table = SlotTable::default();
    assert!(table.is_empty());

    table.write(|table, writer| {
        writer.begin_insert(table);

        writer.start_group(table, TypeId::of::<()>(), None);

        writer.end_insert(table);
    });

     assert!(!table.is_empty());
}
