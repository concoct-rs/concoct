use concoct::{
    task::{self, Task},
    System,
};

fn app(_count: &mut i32) -> impl Task<i32> {
    task::from_fn(|_| dbg!("Hello World!"))
}

fn main() {
    let mut system = System::new(0, app);
    system.build();
    system.rebuild();
}
