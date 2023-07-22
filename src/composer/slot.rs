use crate::Composer;
use std::{
    any::{Any, TypeId},
    fmt,
    rc::Rc,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SlotKind {
    RestartGroup,
    ReplaceGroup,
    Node,
    Data,
}

pub enum GroupKind {
    Restart {
        f: Option<Box<dyn FnMut(&mut Composer) >>,
    },
    Replace,
}

impl fmt::Debug for GroupKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Restart { f: _ } => f.debug_struct("Restart").finish(),
            Self::Replace {} => f.debug_struct("Replace").finish(),
        }
    }
}

pub enum Slot {
    Group {
        id: TypeId,
        len: usize,
        kind: GroupKind,
    },
    Data {
        value: Option<Box<dyn Any>>,
    },
    Node {
        id: Rc<dyn Any>,
    },
}

impl Slot {
    pub fn kind(&self) -> SlotKind {
        match self {
            Self::Group {
                id: _,
                len: _,
                kind,
            } => match kind {
                GroupKind::Replace => SlotKind::ReplaceGroup,
                GroupKind::Restart { f: _ } => SlotKind::RestartGroup,
            },
            Self::Data { value: _ } => SlotKind::Data,
            Self::Node { id: _ } => SlotKind::Node,
        }
    }
}

impl fmt::Debug for Slot {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Group { id: _, len, kind } => f
                .debug_struct("Group")
                .field("len", len)
                .field("kind", kind)
                .finish(),
            Self::Data { value: data } => f.debug_struct("Data").field("data", data).finish(),
            Self::Node { id: _ } => f.debug_struct("Node").finish(),
        }
    }
}
