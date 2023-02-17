use concoct::{
    composable::{state, Container, TextField},
    render::run,
};

fn app() {
    Container::column(|| {
        let name = state(String::new);

        TextField::new(name.cloned(), move |value| name.set(value.to_owned()))
    })
}

#[tokio::main]
async fn main() {
    run(app)
}
