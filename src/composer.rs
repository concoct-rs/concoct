use crate::Widget;
use std::{cell::RefCell, collections::HashMap, fmt, panic::Location};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct IdSegment {
    key: Option<u64>,
    location: &'static Location<'static>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct Id {
    path: Vec<IdSegment>,
}

pub struct WidgetNode {
    widget: Box<dyn Widget>,
    children: Option<Vec<Id>>,
}

#[derive(Default)]
pub struct Composer {
    pub widgets: HashMap<Id, WidgetNode>,
    pub children: Vec<Id>,
    pub current_group_id: Id,
}

impl Composer {
    #[track_caller]
    pub fn with<R>(f: impl FnOnce(&RefCell<Self>) -> R) -> R {
        thread_local! {
             static COMPOSER: RefCell<Composer> = RefCell::new(Composer::default());
        }

        COMPOSER.try_with(f).unwrap()
    }

    pub fn id(&self, location: &'static Location<'static>) -> Id {
        let id_segment = IdSegment {
            key: None,
            location,
        };
        let mut path = self.current_group_id.path.clone();
        path.push(id_segment);
        Id { path }
    }

    #[track_caller]
    pub fn insert_or_update<W>(
        &mut self,
        on_insert: impl FnOnce() -> W,
        on_update: impl FnOnce(&mut W),
    ) where
        W: Widget + 'static,
    {
        let id = Id {
            path: vec![IdSegment {
                key: None,
                location: Location::caller(),
            }],
        };
        self.children.push(id.clone());

        if let Some(widget) = self.widgets.get_mut(&id) {
            on_update(widget.widget.any_mut().downcast_mut().unwrap());
        } else {
            let widget = on_insert();
            self.widgets.insert(
                id,
                WidgetNode {
                    widget: Box::new(widget),
                    children: None,
                },
            );
        }
    }

    #[track_caller]
    pub fn get_mut<W>(&mut self, id: &Id) -> Option<&mut W>
    where
        W: Widget + 'static,
    {
        self.widgets
            .get_mut(id)
            .map(|widget| widget.widget.any_mut().downcast_mut().unwrap())
    }

    #[track_caller]
    pub fn insert(&mut self, id: Id, widget: impl Widget + 'static, children: Option<Vec<Id>>) {
        self.children.push(id.clone());
        self.widgets.insert(
            id,
            WidgetNode {
                widget: Box::new(widget),
                children,
            },
        );
    }
}

impl fmt::Debug for Composer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Composer")
            .field(
                "widgets",
                &Wrap {
                    children: &self.children,
                    composer: self,
                },
            )
            .finish()
    }
}

struct Wrap<'a> {
    children: &'a [Id],
    composer: &'a Composer,
}

impl fmt::Debug for Wrap<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for id in self.children {
            let widget = &self.composer.widgets[id];

            let mut debug_struct = f.debug_struct("Widget");
            debug_struct.field("id", id);

            if let Some(ref children) = widget.children {
                debug_struct.field(
                    "children",
                    &Wrap {
                        children,
                        composer: self.composer,
                    },
                );
            }

            debug_struct.finish()?;
        }

        Ok(())
    }
}
