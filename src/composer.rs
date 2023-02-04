use crate::{composable::container::ContainerWidget, Semantics, Widget};
use skia_safe::{Canvas, Point};
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

impl<T> AsRef<T> for WidgetNode
where
    T: 'static,
{
    fn as_ref(&self) -> &T {
        self.widget.any().downcast_ref().unwrap()
    }
}

impl<T> AsMut<T> for WidgetNode
where
    T: 'static,
{
    fn as_mut(&mut self) -> &mut T {
        self.widget.any_mut().downcast_mut().unwrap()
    }
}

pub trait Visitor {
    fn visit_child(&mut self, widget: &mut Box<dyn Widget>);

    fn visit_group(&mut self, node: &mut WidgetNode);

    fn visit_group_end(&mut self, widget: &mut Box<dyn Widget>);
}

#[derive(Default)]
pub struct Composer {
    pub widgets: HashMap<Id, WidgetNode>,
    pub children: Vec<Id>,
    pub current_group_id: Id,
    pub states: SlotMap<DefaultKey, Id>,
    pub changed: HashSet<(DefaultKey, Id)>,
    pub scale_factor: f32,
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
    pub fn get_node_mut(&mut self, id: &Id) -> Option<&mut WidgetNode> {
        self.widgets.get_mut(id)
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
            GroupStart(Id),
            Child(Id),
        }

        let mut items: Vec<_> = self
            .children
            .iter()
            .map(|id| Item::Child(id.clone()))
            .collect();

        while let Some(item) = items.pop() {
            match item {
                Item::GroupStart(id) => {
                    let node = self.widgets.get_mut(&id).unwrap();
                    visitor.visit_group(node);
                }
                Item::Group(id) => {
                    let node = self.widgets.get_mut(&id).unwrap();
                    visitor.visit_group_end(&mut node.widget)
                }
                Item::Child(id) => {
                    if let Some(node) = self.widgets.get_mut(&id) {
                        if let Some(children) = &node.children {
                            items.push(Item::Group(id.clone()));

                            for child in children
                                .iter()
                                .rev()
                                .map(|id| Item::Child(id.clone()))
                                .clone()
                            {
                                items.push(child);
                            }

                            items.push(Item::GroupStart(id.clone()));
                        } else {
                            visitor.visit_child(&mut node.widget);
                        }
                    }
                }
            }
        }
    }

    pub fn layout(&mut self, semantics: &mut Semantics) {
        semantics.layout_children = vec![Vec::new(), Vec::new()];

        let visitor = LayoutVisitor::new(semantics);
        self.visit(visitor);
    }

    pub fn semantics(&mut self, semantics: &mut Semantics) {
        semantics.points = vec![Point::new(0., 0.)];

        let visitor = SemanticsVisitor::new(semantics);
        self.visit(visitor);
    }

    pub fn paint(&mut self, semantics: &mut Semantics, canvas: &mut Canvas) {
        semantics.points = vec![Point::new(0., 0.)];

        let visitor = PaintVisitor::new(semantics, canvas);
        self.visit(visitor);
    }

    pub fn recompose(semantics: &mut Semantics) {
        Self::with(|composer| {
            let mut cx = composer.borrow_mut();
            if let Some((_key, parent_id)) = cx
                .changed
                .iter()
                .min_by_key(|(_key, id)| id.path.len())
                .cloned()
            {
                let container: &mut ContainerWidget = cx.get_mut(&parent_id).unwrap();
                let mut f = container.f.take().unwrap();
                cx.current_group_id = parent_id.clone();
                drop(cx);

                let (children, removed) = Self::group(&parent_id, &mut f);
                if let Some(mut removed) = removed {
                    for node in &mut removed {
                        node.widget.remove(semantics);
                    }
                }

                let mut cx = composer.borrow_mut();
                let node = cx.get_node_mut(&parent_id).unwrap();
                node.children = Some(children);

                let container: &mut ContainerWidget = node.as_mut();
                container.f = Some(f);

                let layout_id = container.layout_id.unwrap();
                let mut layout_children = semantics.layout_children.pop().unwrap();
                layout_children.reverse();
                semantics
                    .taffy
                    .set_children(layout_id, &layout_children)
                    .unwrap();
            }
        });
    }
}

impl fmt::Debug for Composer {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
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
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for id in self.children {
            let widget = &self.composer.widgets[id];

            let mut debug_struct = f.debug_struct("Widget");
            debug_struct.field("id", id);

            if let Some(ref children) = widget.children {
                debug_struct.field(
                    "children",
                    &[Wrap {
                        children,
                        composer: self.composer,
                    }],
                );
            }

            debug_struct.finish()?;
        }

        Ok(())
    }
}

pub struct LayoutVisitor<'a> {
    semantics: &'a mut Semantics,
}

impl<'a> LayoutVisitor<'a> {
    pub fn new(semantics: &'a mut Semantics) -> Self {
        Self { semantics }
    }
}

impl Visitor for LayoutVisitor<'_> {
    fn visit_child(&mut self, widget: &mut Box<dyn Widget>) {
        widget.layout(self.semantics);
    }

    fn visit_group(&mut self, _node: &mut WidgetNode) {
        self.semantics.layout_children.push(Vec::new());
    }

    fn visit_group_end(&mut self, widget: &mut Box<dyn Widget>) {
        widget.layout(self.semantics);
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

    fn visit_group(&mut self, node: &mut WidgetNode) {
        let widget: &ContainerWidget = node.as_ref();
        if let Some(layout_id) = widget.layout_id {
            let layout = self.semantics.layout(layout_id);
            self.semantics
                .points
                .push(Point::new(layout.location.x, layout.location.y));
        }

        self.semantics.layout_children.push(Vec::new());
        self.semantics.start_group()
    }

    fn visit_group_end(&mut self, widget: &mut Box<dyn Widget>) {
        self.semantics.points.pop().unwrap();
        widget.semantics(self.semantics);
    }
}

pub struct PaintVisitor<'a> {
    semantics: &'a mut Semantics,
    canvas: &'a mut Canvas,
}

impl<'a> PaintVisitor<'a> {
    pub fn new(semantics: &'a mut Semantics, canvas: &'a mut Canvas) -> Self {
        Self { semantics, canvas }
    }
}

impl Visitor for PaintVisitor<'_> {
    fn visit_child(&mut self, widget: &mut Box<dyn Widget>) {
        widget.paint(self.semantics, self.canvas);
    }

    fn visit_group(&mut self, node: &mut WidgetNode) {
        let widget: &mut ContainerWidget = node.as_mut();
        if let Some(layout_id) = widget.layout_id {
            let layout = self.semantics.layout(layout_id);

            widget.modify.paint(&layout, self.canvas);

            self.semantics
                .points
                .push(Point::new(layout.location.x, layout.location.y));
        }
    }

    fn visit_group_end(&mut self, widget: &mut Box<dyn Widget>) {
        self.semantics.points.pop().unwrap();
        widget.paint(self.semantics, self.canvas);
    }
}
