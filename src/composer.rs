use std::{any::Any, cell::RefCell, collections::HashMap, panic::Location};
use crate::Widget;


#[derive(PartialEq, Eq, Hash)]
pub struct IdSegment {
    key: Option<u64>,
    location: &'static Location<'static>,
}

#[derive(PartialEq, Eq, Hash)]
pub struct Id {
    path: Vec<IdSegment>,
}

#[derive(Default)]
pub struct Composer {
    pub widgets: HashMap<Id, Box<dyn Widget>>,
}

impl Composer {
    #[track_caller]
    pub fn with<R>(f: impl FnOnce(&RefCell<Self>) -> R) -> R {
        thread_local! {
             static COMPOSER: RefCell<Composer> = RefCell::new(Composer::default());
        }

        COMPOSER.try_with(f).unwrap()
    }

    #[track_caller]
    pub fn insert_or_update<W>(
        &mut self,
        on_insert: impl FnOnce() -> W,
        on_update: impl FnOnce(&mut W),
    ) where
        W: Widget  + 'static,
    {
        let id = Id {
            path: vec![IdSegment {
                key: None,
                location: Location::caller(),
            }],
        };
      
        if let Some(widget) = self.widgets.get_mut(&id) {
            on_update(widget.any_mut().downcast_mut().unwrap());
           
        } else {
            let widget = on_insert();
            self.widgets.insert(id, Box::new(widget));
        }
    }
}