use accesskit::Role;
use concoct::{container, state::state, text, Modifier, Tester};

#[test]
fn it_updates_state_and_recomposes() {
    let mut tester = Tester::new(|| {
        container(Modifier::default(), || {
            let count = state(|| 0);

            text(String::from(count.get().cloned().to_string()));

            *count.get().as_mut() += 1;
        })
    });

    for count in 0..5 {
        assert!(tester.get_text(count.to_string()).is_some());
    }
}

#[test]
fn it_triggers_click_events_and_recomposes() {
    let mut tester = Tester::new(|| {
        container(Modifier::default(), || {
            let count = state(|| 0);

            container(
                Modifier::default()
                    .clickable(move |_action_request| *count.get().as_mut() += 1)
                    .merge_descendants()
                    .role(Role::Button),
                move || text(String::from(count.get().cloned().to_string())),
            )
        })
    });

    for count in 0..5 {
        tester.get_text(count.to_string()).unwrap().click();
    }
}

#[test]
fn it_removes_unused_widgets() {
    let mut tester = Tester::new(|| {
        container(Modifier::default(), || {
            let is_shown = state(|| true);

            if is_shown.get().cloned() {
                text(String::from("toggle"));
            }

            *is_shown.get().as_mut() = false;
        })
    });

    assert!(tester.get_text("toggle").is_some());
    assert!(tester.get_text("toggle").is_none());
}
