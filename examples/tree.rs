use concoct::{composable, compose, node, remember, Apply, Composer, State};

#[composable]
fn app() {
    let count = compose!(remember(|| State::new(0)));

    compose!(node(*count.get()));

    count.update(|count| *count += 1);
}

pub struct Tree {}

impl Apply for Tree {
    type NodeId = ();

    fn root(&mut self) -> Self::NodeId {}

    fn insert(&mut self, _parent_id: Self::NodeId, _node: Box<dyn std::any::Any>) -> Self::NodeId {
        dbg!("insert!");
    }

    fn update(&mut self, _node_id: Self::NodeId, _node: Box<dyn std::any::Any>) {
        dbg!("update!");
    }
}

#[tokio::main]
async fn main() {
    let mut composer = Composer::new(Tree {});
    composer.compose(app());

    composer.recompose().await;
}
