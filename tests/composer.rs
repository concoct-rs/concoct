use accesskit::{Action, Role};
use concoct::{container, text, Composer, Modifier, Semantics};

#[test]
fn it_works() {
    fn app() {
        container(
            Modifier::default()
                .clickable(|action_request| {
                    dbg!(action_request);
                })
                .merge_descendants()
                .role(Role::Button),
            || {
                text(String::from("Hello World!"));
            },
        )
    }

    app();

    Composer::with(|composer| {
        let mut cx = composer.borrow_mut();

        let mut semantics = Semantics::default();
        cx.semantics(&mut semantics);

        for handler in semantics.handlers.values_mut() {
            handler(Action::Default)
        }
    });
}
