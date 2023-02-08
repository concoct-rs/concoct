use concoct::{
    composable::{state, Container, Text},
    Tester,
};

#[test]
fn it_updates_state_and_recomposes() {
    let mut tester = Tester::new(|| {
        Container::column(|| {
            let count = state(|| 0);

            Text::new(count.get().cloned().to_string());

            *count.get().as_mut() += 1;
        })
    });

    for count in 0..5 {
        assert!(tester.get_text(count.to_string()).is_some());
    }
}

/*
#[test]
fn it_triggers_click_events_and_recomposes() {
    let mut tester = Tester::new(|| {
        container(Modifier, || {
            let count = state(|| 0);

            container(
                Modifier
                    .clickable(move || *count.get().as_mut() += 1)
                    .merge_descendants()
                    .role(Role::Button),
                move || Text::new( count.get().cloned().to_string()),
            )
        })
    });

    for count in 0..5 {
        tester.get_text(count.to_string()).unwrap().click();
    }
}
*/

#[test]
fn it_removes_unused_widgets() {
    let mut tester = Tester::new(|| {
        Container::column(|| {
            let is_shown = state(|| true);

            if is_shown.get().cloned() {
                Text::new("toggle");
            }

            *is_shown.get().as_mut() = false;
        })
    });

    assert!(tester.get_text("toggle").is_some());
    assert!(tester.get_text("toggle").is_none());
}

#[test]
fn it_removes_nested_unused_widgets() {
    let mut tester = Tester::new(|| {
        Container::column(|| {
            let is_shown = state(|| true);

            if is_shown.get().cloned() {
                Container::column(|| {
                    Text::new("toggle");
                })
            }

            *is_shown.get().as_mut() = false;
        })
    });

    assert!(tester.get_text("toggle").is_some());
    assert!(tester.get_text("toggle").is_none());
}

#[test]
fn it_inserts_new_widgets() {
    let mut tester = Tester::new(|| {
        Container::column(|| {
            let is_shown = state(|| false);

            if is_shown.get().cloned() {
                Text::new("A");
            }

            *is_shown.get().as_mut() = true;
        })
    });

    assert!(tester.get_text("A").is_none());
    assert!(tester.get_text("A").is_some());
}

#[test]
fn it_replaces_widgets() {
    let mut tester = Tester::new(|| {
        Container::column(|| {
            let is_a_shown = state(|| true);

            if is_a_shown.get().cloned() {
                Text::new("A");
            } else {
                Text::new("B");
            }

            *is_a_shown.get().as_mut() = false;
        })
    });

    assert!(tester.get_text("A").is_some());
    assert!(tester.get_text("B").is_some());
}

#[test]
fn it_nests_containers() {
    let tester = Tester::new(|| {
        Container::column(|| {
            Text::new("A");

            Container::column(|| {
                Text::new("B");
                Text::new("C");
            });

            Text::new("D");
        })
    });

    dbg!(&tester.semantics);
}
