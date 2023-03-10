use crate::{composable::container::ContainerWidget, Semantics, Widget};
use skia_safe::{Canvas, Point};
use slotmap::{new_key_type, SlotMap};
use std::{
    any::TypeId,
    cell::RefCell,
    collections::{HashMap, HashSet},
    fmt, mem,
    panic::Location,
    rc::Rc,
};
use tracing::debug;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct IdSegment {
    key: Option<u64>,
    location: &'static Location<'static>,
}

impl fmt::Display for IdSegment {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(key) = self.key {
            write!(f, "({}) ", key)?;
        }

        self.location.fmt(f)
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct Id {
    path: Rc<Box<[IdSegment]>>,
}

impl fmt::Display for Id {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.path
            .iter()
            .map(|segment| format!("{}", segment))
            .collect::<Vec<_>>()
            .join("->")
            .fmt(f)
    }
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

new_key_type! {
    pub struct StateKey;
}

#[derive(Default)]
pub struct Composer {
    pub widgets: HashMap<Id, WidgetNode>,
    pub children: Vec<Id>,
    pub current_group_id: Id,
    pub states: SlotMap<StateKey, Id>,
    pub changed: HashSet<(StateKey, Id)>,
    pub scale_factor: f32,
    pub contexts: HashMap<TypeId, Id>,
    pub removed: Vec<WidgetNode>,
    pub next_key: Option<u64>,
}

impl Composer {
    #[track_caller]
    pub fn with<R>(f: impl FnOnce(&RefCell<Self>) -> R) -> R {
        thread_local! {
             static COMPOSER: RefCell<Composer> = RefCell::new(Composer::default());
        }

        COMPOSER.try_with(f).unwrap()
    }

    pub fn id(&mut self, location: &'static Location<'static>) -> Id {
        let key = self.next_key.take();
        let id_segment = IdSegment { key, location };

        let mut path = Vec::from(&**self.current_group_id.path);
        path.push(id_segment);

        Id {
            path: Rc::new(path.into_boxed_slice()),
        }
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

    pub fn group(id: &Id, f: impl FnOnce()) -> Vec<Id> {
        Composer::with(|composer| {
            let mut cx = composer.borrow_mut();
            let parent_children = mem::take(&mut cx.children);
            let parent_group_id = mem::replace(&mut cx.current_group_id, id.clone());
            drop(cx);

            f();

            let mut cx = composer.borrow_mut();
            cx.current_group_id = parent_group_id;
            let children = mem::replace(&mut cx.children, parent_children);

            let removed = if let Some(node) = cx.widgets.get(&id) {
                Some(
                    node.children
                        .as_ref()
                        .unwrap()
                        .iter()
                        .filter(|id| !children.contains(id))
                        .cloned()
                        .collect::<Vec<Id>>()
                        .into_iter()
                        .map(|id| cx.widgets.remove(&id).unwrap())
                        .collect::<Vec<_>>(),
                )
            } else {
                None
            };

            if let Some(removed) = removed {
                cx.removed.extend(removed);
            }

            children
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
                debug!("Recomposing {}", &parent_id);

                let container: &mut ContainerWidget = cx.get_mut(&parent_id).unwrap();
                let mut content = container.content.take().unwrap();
                cx.current_group_id = parent_id.clone();
                cx.changed.clear();
                drop(cx);

                let children = Self::group(&parent_id, &mut content);
                let mut cx = composer.borrow_mut();
                let node = cx.get_node_mut(&parent_id).unwrap();
                node.children = Some(children);

                let container: &mut ContainerWidget = node.as_mut();
                container.content = Some(content);

                let layout_id = container.layout_id.unwrap();
                let mut layout_children = semantics.layout_children.pop().unwrap();
                layout_children.reverse();
                semantics
                    .taffy
                    .set_children(layout_id, &layout_children)
                    .unwrap();

                let mut removed = mem::take(&mut cx.removed);
                drop(cx);

                for child in &mut removed {
                    child.widget.remove(semantics);
                }
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
        self.canvas.save();
        widget.paint(self.semantics, self.canvas);
        self.canvas.restore();
    }

    fn visit_group(&mut self, node: &mut WidgetNode) {
        let widget: &mut ContainerWidget = node.as_mut();
        if let Some(layout_id) = widget.layout_id {
            let layout = self.semantics.layout(layout_id);

            self.canvas.save();
            widget.modifier.paint(&layout, self.canvas);
            self.canvas.restore();

            self.semantics
                .points
                .push(Point::new(layout.location.x, layout.location.y));
        }
    }

    fn visit_group_end(&mut self, widget: &mut Box<dyn Widget>) {
        self.semantics.points.pop().unwrap();

        self.canvas.save();
        widget.paint(self.semantics, self.canvas);
        self.canvas.restore();
    }
}
