use accesskit::Node;
use concoct::Semantics;

#[test]
fn it_works() {
    let mut semantics = Semantics::default();

    semantics.start_group();

    semantics.insert(Node {
        value: Some("Hello World!".into()),
        ..Node::default()
    });

    semantics.end_group();

    dbg!(semantics);
}

#[test]
fn it_updates() {
    let mut semantics = Semantics::default();

    semantics.start_group();

    let text_id = semantics.insert(Node {
        value: Some("Hello World!".into()),
        ..Node::default()
    });

    let group_id = semantics.end_group();

    dbg!(semantics.tree_update());

    semantics.start_group();
    semantics.remove(text_id);
    semantics.end_group_update(group_id);

    dbg!(semantics.tree_update());
}
