use concoct::{composable, compose, Apply, Composer};

#[composable]
fn app() {
    composer.create_node(Box::new(()));
}

pub struct Tree {}

impl Apply for Tree {
    type NodeId = ();

    fn root(&mut self) -> Self::NodeId {}

    fn insert(&mut self, parent_id: Self::NodeId, node: Box<dyn std::any::Any>) -> Self::NodeId {
        dbg!(parent_id, node);
    }
}

#[tokio::main]
async fn main() {
    let mut composer = Composer::new(Tree {});
    composer.compose(app());
}
