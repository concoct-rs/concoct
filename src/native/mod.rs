use crate::{
    html::{Builder, Html, HtmlPlatform},
    IntoView, Platform, Tree, View,
};
use std::{cell::RefCell, rc::Rc};
use winit::{
    dpi::LogicalSize,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};
use wry::WebViewBuilder;

thread_local! {
    static HTML_CONTEXT: RefCell<Option<NativeContext>> = RefCell::default();
}

struct Inner {
    web_view: wry::WebView,
}

#[derive(Clone)]
pub struct NativeContext {
    inner: Rc<RefCell<Inner>>,
}

impl NativeContext {
    pub fn new(web_view: wry::WebView) -> Self {
        Self {
            inner: Rc::new(RefCell::new(Inner { web_view })),
        }
    }

    pub fn current() -> Self {
        HTML_CONTEXT
            .try_with(|cx| cx.borrow().as_ref().unwrap().clone())
            .unwrap()
    }

    pub fn enter(self) {
        HTML_CONTEXT
            .try_with(|cx| *cx.borrow_mut() = Some(self))
            .unwrap()
    }
}

pub struct NativeView;

impl Platform for NativeView {
    fn from_str(&mut self, s: &str) -> Box<dyn crate::AnyView> {
        NativeContext::current()
            .inner
            .borrow()
            .web_view
            .evaluate_script(&format!(
                r#"
                    var node = document.createTextNode("{s}");
                    document.body.appendChild(node);
                "#
            ))
            .unwrap();
        Box::new(())
    }
}

#[derive(PartialEq)]
pub struct WebView {}

impl WebView {
    pub fn new<C>(content: C) -> Self {
        Self {}
    }
}

impl View for WebView {
    fn view(&mut self) -> impl IntoView {
        todo!()
    }
}

pub fn run(content: impl IntoView) {
    let event_loop = EventLoop::new().unwrap();
    let window =
        WindowBuilder::new()
            .with_inner_size(LogicalSize::new(800, 800))
            .build(&event_loop)
            .unwrap();

    #[allow(unused_mut)]
    let mut builder = WebViewBuilder::new(&window);
    let web_view = builder
        .with_html("<html><body></body></html>")
        .unwrap()
        .build()
        .unwrap();

    let cx = NativeContext::new(web_view);
    cx.enter();

    let mut composition = Tree::new(NativeView, content);
    composition.build();

    event_loop
        .run(move |event, evl| {
            evl.set_control_flow(ControlFlow::Poll);

            #[cfg(any(
                target_os = "linux",
                target_os = "dragonfly",
                target_os = "freebsd",
                target_os = "netbsd",
                target_os = "openbsd",
            ))]
            while gtk::events_pending() {
                gtk::main_iteration_do(false);
            }

            match event {
                #[cfg(any(
                    target_os = "linux",
                    target_os = "dragonfly",
                    target_os = "freebsd",
                    target_os = "netbsd",
                    target_os = "openbsd",
                ))]
                Event::WindowEvent {
                    event: WindowEvent::Resized(size),
                    ..
                } => {
                    _webview.set_bounds(wry::Rect {
                        x: 0,
                        y: 0,
                        width: size.width,
                        height: size.height,
                    });
                }
                Event::WindowEvent {
                    event: WindowEvent::CloseRequested,
                    ..
                } => evl.exit(),
                _ => {}
            }
        })
        .unwrap();
}
