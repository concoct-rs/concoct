use concoct::slot_table::SlotTable;

#[test]
fn it_inserts_groups() {
    let slot_table = SlotTable::default();
    assert!(slot_table.is_empty());

    let mut writer = slot_table.writer();
    writer.begin_insert();
    writer.end_insert();

    // assert!(!slot_table.is_empty());
}
