use concoct::{View, VirtualDom};
use concoct_menu::{PredefinedMenuItem, Submenu};

fn main() {
    concoct_menu::run((
        Submenu::new("A", true, PredefinedMenuItem::about(Some("B"), None)),
        Submenu::new("C", true, PredefinedMenuItem::about(Some("D"), None)),
    ));
}
