use concoct::slot_table::SlotTable;

#[test]
fn it_is_empty() {
    let mut table = SlotTable::default();
    assert!(table.is_empty());

    table.write(|table, writer| {
        writer.begin_insert(table);
    });

    assert!(!table.is_empty());
}
