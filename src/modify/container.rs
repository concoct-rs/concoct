use crate::Modify;
use accesskit::Role;

pub struct ContainerModifier {
    pub merge_descendants: bool,
    pub role: Role,
}

impl Default for ContainerModifier {
    fn default() -> Self {
        Self {
            merge_descendants: false,
            role: Role::default(),
        }
    }
}

impl AsMut<Role> for ContainerModifier {
    fn as_mut(&mut self) -> &mut Role {
        &mut self.role
    }
}

pub struct MergeDescendants;

impl Modify<ContainerModifier> for MergeDescendants {
    fn modify(&mut self, value: &mut ContainerModifier) {
        value.merge_descendants = true;
    }
}
