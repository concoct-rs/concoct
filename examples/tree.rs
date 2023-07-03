use concoct::{composable, compose, node, remember, Apply, Composer, State};

#[composable]
fn app() {
    let count = compose!(remember(|| State::new(0)));

    compose!(node(*count.get()));

    count.update(|count| *count += 1);
}

pub struct Tree {}

impl Apply for Tree {
    fn root(&mut self) -> Box<dyn std::any::Any> {
        Box::new(())
    }

    fn insert(
        &mut self,
        _parent_id: &dyn std::any::Any,
        _node: Box<dyn std::any::Any>,
    ) -> Box<dyn std::any::Any> {
        dbg!("insert!");
        Box::new(())
    }

    fn update(&mut self, _node_id: &dyn std::any::Any, _node: Box<dyn std::any::Any>) {
        dbg!("update!");
    }
}

#[tokio::main]
async fn main() {
    let tree = Tree {};
    let mut composer = Composer::new(Box::new(tree));
    composer.compose(app());

    composer.recompose().await;
}
