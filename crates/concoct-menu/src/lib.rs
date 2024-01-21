use concoct::{
    hook::{use_context, use_provider},
    view::ViewCell,
    View, ViewBuilder, VirtualDom,
};
use muda::MenuEvent;
use muda::{
    AboutMetadata, Menu as RawMenu, PredefinedMenuItem as RawPredefinedMenuItem,
    Submenu as RawSubmenu,
};
#[cfg(target_os = "macos")]
use winit::platform::macos::EventLoopBuilderExtMacOS;
#[cfg(target_os = "windows")]
use winit::platform::windows::{EventLoopBuilderExtWindows, WindowExtWindows};
use winit::{
    event_loop::{ControlFlow, EventLoopBuilder},
    window::WindowBuilder,
};

pub struct Menu<C> {
    raw: RawMenu,
    content: ViewCell<C>,
}

impl<C: View> Menu<C> {
    pub fn new(content: C) -> Self {
        Self::from_raw(RawMenu::new(), content)
    }

    pub fn from_raw(raw: RawMenu, content: C) -> Self {
        Self {
            raw,
            content: ViewCell::new(content),
        }
    }
}

impl<C: View> ViewBuilder for Menu<C> {
    fn build(&self) -> impl View {
        use_provider(MenuContext::Menu(self.raw.clone()));

        self.content.clone()
    }
}

pub enum MenuContext {
    Menu(RawMenu),
    Submenu(RawSubmenu),
}

pub struct Submenu<C> {
    raw: RawSubmenu,
    content: ViewCell<C>,
}

impl<C: View> Submenu<C> {
    pub fn new<S>(text: S, enabled: bool, content: C) -> Self
    where
        S: AsRef<str>,
    {
        Self::from_raw(RawSubmenu::new(text, enabled), content)
    }

    pub fn from_raw(raw: RawSubmenu, content: C) -> Self {
        Self {
            raw,
            content: ViewCell::new(content),
        }
    }
}

impl<C: View> ViewBuilder for Submenu<C> {
    fn build(&self) -> impl View {
        let menu_cx = use_context::<MenuContext>().unwrap();
        match &*menu_cx {
            MenuContext::Menu(menu) => menu.append(&self.raw).unwrap(),
            MenuContext::Submenu(submenu) => submenu.append(&self.raw).unwrap(),
        }

        use_provider(MenuContext::Submenu(self.raw.clone()));

        self.content.clone()
    }
}

pub struct PredefinedMenuItem {
    raw: RawPredefinedMenuItem,
}

impl PredefinedMenuItem {
    pub fn about(text: Option<&str>, metadata: Option<AboutMetadata>) -> Self {
        Self {
            raw: RawPredefinedMenuItem::about(text, metadata),
        }
    }
}

impl ViewBuilder for PredefinedMenuItem {
    fn build(&self) -> impl View {
        let menu_cx = use_context::<MenuContext>().unwrap();
        match &*menu_cx {
            MenuContext::Menu(menu) => menu.append(&self.raw).unwrap(),
            MenuContext::Submenu(submenu) => submenu.append(&self.raw).unwrap(),
        }
    }
}

pub fn run(view: impl View) {
    let mut event_loop_builder = EventLoopBuilder::new();

    let menu_bar = RawMenu::new();

    let mut vdom = VirtualDom::new(Menu::from_raw(menu_bar.clone(), view).into_tree());
    vdom.build();

    #[cfg(target_os = "windows")]
    {
        let menu_bar = menu_bar.clone();
        event_loop_builder.with_msg_hook(move |msg| {
            use windows_sys::Win32::UI::WindowsAndMessaging::{TranslateAcceleratorW, MSG};
            unsafe {
                let msg = msg as *const MSG;
                let translated = TranslateAcceleratorW((*msg).hwnd, menu_bar.haccel(), msg);
                translated == 1
            }
        });
    }
    #[cfg(target_os = "macos")]
    event_loop_builder.with_default_menu(false);

    let event_loop = event_loop_builder.build().unwrap();

    let _window = WindowBuilder::new()
        .with_title("Window 1")
        .build(&event_loop)
        .unwrap();
    let _window2 = WindowBuilder::new()
        .with_title("Window 2")
        .build(&event_loop)
        .unwrap();

    // let window_m = Submenu::new("&Window", true);

    #[cfg(target_os = "windows")]
    {
        use winit::raw_window_handle::*;
        if let RawWindowHandle::Win32(handle) = window.window_handle().unwrap().as_raw() {
            menu_bar.init_for_hwnd(handle.hwnd.get());
        }
        if let RawWindowHandle::Win32(handle) = window2.window_handle().unwrap().as_raw() {
            menu_bar.init_for_hwnd(handle.hwnd.get());
        }
    }
    #[cfg(target_os = "macos")]
    {
        menu_bar.init_for_nsapp();
        // window_m.set_as_windows_menu_for_nsapp();
    }

    let _menu_channel = MenuEvent::receiver();

    event_loop
        .run(move |_event, event_loop| {
            event_loop.set_control_flow(ControlFlow::Wait);
        })
        .unwrap();
}
