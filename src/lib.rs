use accesskit::{NodeBuilder, Role};

pub trait Semantics {
    fn is_changed(&self, old: &Self) -> bool;

    fn build(&mut self) -> NodeBuilder;

    fn modify<F>(self, f: F) -> ModifySemantics<Self, F>
    where
        Self: Sized,
        F: FnMut(&mut NodeBuilder),
    {
        ModifySemantics { semantics: self, f }
    }
}

pub struct Text {
    string: String,
}

impl Semantics for Text {
    fn is_changed(&self, old: &Self) -> bool {
        self.string != old.string
    }

    fn build(&mut self) -> NodeBuilder {
        let mut builder = NodeBuilder::new(Role::StaticText);
        builder.set_value(self.string.clone());
        builder
    }
}

pub struct ModifySemantics<T, F> {
    semantics: T,
    f: F,
}

impl<T, F> Semantics for ModifySemantics<T, F>
where
    T: Semantics,
    F: FnMut(&mut NodeBuilder),
{
    fn is_changed(&self, old: &Self) -> bool {
        self.semantics.is_changed(&old.semantics)
    }

    fn build(&mut self) -> NodeBuilder {
        let mut builder = self.semantics.build();
        (self.f)(&mut builder);
        builder
    }
}
