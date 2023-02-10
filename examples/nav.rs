use concoct::{
    composable::{
        material::{NavigationBar, NavigationBarItem},
        Text,
    },
    render::run,
};

fn app() {
    NavigationBar::new(|| {
        NavigationBarItem::new(|| Text::new("I"), || Text::new("Label"));

        NavigationBarItem::new(|| Text::new("L"), || Text::new("Longer label"))
    })
}

#[tokio::main]
async fn main() {
    run(app)
}
