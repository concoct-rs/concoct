use concoct::{composable, compose, node, remember, Apply, Composer, State};
use std::any::Any;

#[composable]
fn app() {
    let count = compose!(remember(|| State::new(0)));
    count.update(|count| *count += 1);

    if *count.get() == 0 {
        compose!(node(*count.get()));
    }
}

pub struct Tree {}

impl Apply for Tree {
    fn root(&mut self) -> Box<dyn Any> {
        Box::new(())
    }

    fn insert(&mut self, parent_id: &dyn Any, _node: Box<dyn Any>) -> Box<dyn Any> {
        println!("insert: {:?}", parent_id);
        Box::new(())
    }

    fn update(&mut self, node_id: &dyn Any, _node: Box<dyn Any>) {
        println!("update: {:?}", node_id);
    }

    fn remove(&mut self, node_id: &dyn Any) {
        println!("remove: {:?}", node_id);
    }
}

#[tokio::main]
async fn main() {
    let tree = Tree {};
    let mut composer = Composer::new(Box::new(tree));

    composer.compose(app());
    dbg!(&composer);

    composer.recompose().await;
    dbg!(&composer);
}
