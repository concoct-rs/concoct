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
        assert!(tester
            .get(|_id, node| node.value.as_deref() == Some(&count.to_string()))
            .is_some());
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
        tester
            .get(|_id, node| node.value.as_deref() == Some(&count.to_string()))
            .unwrap()
            .click();
    }
}
