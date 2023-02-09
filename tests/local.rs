use concoct::{
    composable::{local, provider, state, Container},
    Tester,
};

#[test]
fn it_stores_locals() {
    let _tester = Tester::new(|| {
        Container::column(|| {
            provider(1, || {
                assert_eq!(*local::<i32>().unwrap(), 1);
            })
        })
    });
}

#[test]
fn it_stores_locals_for_recomposition() {
    let mut tester = Tester::new(|| {
        Container::column(|| {
            provider(1, || {
                Container::column(|| {
                    let should_recompose = state(|| false);

                    assert_eq!(*local::<i32>().unwrap(), 1);

                    *should_recompose.get().as_mut() = true;
                })
            })
        })
    });

    for _ in 0..2 {
        tester.request_recompose();
    }
}
