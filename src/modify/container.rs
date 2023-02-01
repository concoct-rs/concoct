use crate::Modify;
use accesskit::Role;
use taffy::style::Style;

pub struct ContainerModifier {
    pub merge_descendants: bool,
    pub role: Role,
    pub style: Style
}



impl Default for ContainerModifier {
    fn default() -> Self {
        Self {
            merge_descendants: false,
            role: Role::default(),
            style: Style::default()
        }
    }
}

impl AsMut<Role> for ContainerModifier {
    fn as_mut(&mut self) -> &mut Role {
        &mut self.role
    }
}

impl AsMut<Style> for ContainerModifier {
    fn as_mut(&mut self) -> &mut Style {
        &mut self.style
    }
}

pub struct MergeDescendants;

impl Modify<ContainerModifier> for MergeDescendants {
    fn modify(&mut self, value: &mut ContainerModifier) {
        value.merge_descendants = true;
    }
}
