use crate::{container::ContainerWidget, Semantics, Widget};
use slotmap::{DefaultKey, SlotMap};
use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
    fmt, mem,
    panic::Location,
};

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
    pub widget: Box<dyn Widget>,
    pub children: Option<Vec<Id>>,
}

pub trait Visitor {
    fn visit_child(&mut self, widget: &mut Box<dyn Widget>);

    fn visit_group(&mut self);
}

#[derive(Default)]
pub struct Composer {
    pub widgets: HashMap<Id, WidgetNode>,
    pub children: Vec<Id>,
    pub current_group_id: Id,
    pub states: SlotMap<DefaultKey, Id>,
    pub changed: HashSet<Id>,
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
    pub fn get<W>(&self, id: &Id) -> Option<&W>
    where
        W: Widget + 'static,
    {
        self.widgets
            .get(id)
            .map(|widget| widget.widget.any().downcast_ref().unwrap())
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

    pub fn group(id: &Id, f: impl FnOnce()) -> (Vec<Id>, Option<Vec<WidgetNode>>) {
        Composer::with(|composer| {
            let mut cx = composer.borrow_mut();
            let parent_children = mem::take(&mut cx.children);
            let parent_group_id = mem::replace(&mut cx.current_group_id, id.clone());
            drop(cx);

            f();

            let mut cx = composer.borrow_mut();
            cx.current_group_id = parent_group_id;
            let children = mem::replace(&mut cx.children, parent_children);

            let removed_ids = if let Some(node) = cx.widgets.get(&id) {
                let removed: Vec<_> = node
                    .children
                    .as_ref()
                    .unwrap()
                    .iter()
                    .filter(|id| !children.contains(id))
                    .cloned()
                    .collect();

                Some(removed)
            } else {
                None
            };
            let removed = removed_ids.map(|removed| {
                removed
                    .iter()
                    .map(|id| cx.widgets.remove(id).unwrap())
                    .collect()
            });
            (children, removed)
        })
    }

    pub fn visit(&mut self, mut visitor: impl Visitor) {
        enum Item {
            Group(Id),
            Child(Id),
        }

        let mut items: Vec<_> = self
            .children
            .iter()
            .map(|id| Item::Child(id.clone()))
            .collect();

        let mut idx = 0;
        while let Some(item) = items.pop() {
            match item {
                Item::Group(id) => {
                    let node = self.widgets.get_mut(&id).unwrap();
                    visitor.visit_child(&mut node.widget)
                }
                Item::Child(id) => {
                    if let Some(node) = self.widgets.get_mut(&id) {
                        if let Some(children) = &node.children {
                            visitor.visit_group();

                            let end_id = id.clone();
                            for child in children.iter().map(|id| Item::Child(id.clone())).clone() {
                                if idx < items.len() {
                                    items.insert(idx, child);
                                } else {
                                    items.push(child);
                                }
                            }

                            if idx < items.len() {
                                items.insert(idx, Item::Group(end_id))
                            } else {
                                items.push(Item::Group(end_id))
                            }
                        } else {
                            visitor.visit_child(&mut node.widget);
                        }
                    }
                }
            }

            idx += 1;
        }
    }

    pub fn semantics(&mut self, semantics: &mut Semantics) {
        let visitor = SemanticsVisitor::new(semantics);
        self.visit(visitor);
    }

    pub fn recompose(semantics: &mut Semantics) {
        Self::with(|composer| {
            let mut cx = composer.borrow_mut();
            if let Some(parent_id) = cx.changed.iter().min_by_key(|id| id.path.len()).cloned() {
                let container: &mut ContainerWidget = cx.get_mut(&parent_id).unwrap();
                let mut f = container.f.take().unwrap();
                cx.current_group_id = parent_id.clone();
                drop(cx);

                let (_children, removed) = Self::group(&parent_id, &mut f);
                if let Some(mut removed) = removed {
                    for widget in &mut removed {
                        widget.widget.remove(semantics);
                    }
                }

                let mut cx = composer.borrow_mut();
                let container: &mut ContainerWidget = cx.get_mut(&parent_id).unwrap();
                container.f = Some(f);
            }
        });
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

pub struct SemanticsVisitor<'a> {
    semantics: &'a mut Semantics,
}

impl<'a> SemanticsVisitor<'a> {
    pub fn new(semantics: &'a mut Semantics) -> Self {
        Self { semantics }
    }
}

impl Visitor for SemanticsVisitor<'_> {
    fn visit_child(&mut self, widget: &mut Box<dyn Widget>) {
        widget.semantics(self.semantics);
    }

    fn visit_group(&mut self) {
        self.semantics.start_group()
    }
}
