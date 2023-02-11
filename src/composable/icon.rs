use skia_safe::{Paint, Path};
use taffy::prelude::Size;

use crate::{modify::ModifyExt, Modifier, Widget};

use super::Container;

pub struct Icon {
    path: Path,
    paint: Paint,
}

impl Icon {
    pub fn build(svg: &str, paint: Paint) -> Self {
        let path = Path::from_svg(svg).unwrap();
        Self { path, paint }
    }

    #[track_caller]
    pub fn view(self) {
        Container::build_row(|| {})
            .size(Size::from_points(50., 50.))
            .modifier(Modifier.draw(move |layout, canvas| {
                canvas.draw_path(&self.path, &self.paint);
            }))
            .view();
    }
}
