use crate::{
    html::{Builder, Html, HtmlPlatform},
    IntoView, Platform, Tree,
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
    static HTML_CONTEXT: RefCell<Option<WebContext>> = RefCell::default();
}

struct Inner {}

#[derive(Clone)]
pub struct WebContext {
    inner: Rc<RefCell<Inner>>,
}

impl WebContext {
    pub fn new() -> Self {
        Self {
            inner: Rc::new(RefCell::new(Inner {})),
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

pub fn div<C>(child: C) -> Html<WebHtml, C> {
    Html::new(WebHtml {}, child)
}

#[derive(PartialEq, Eq)]
pub struct WebHtml {}

impl HtmlPlatform for WebHtml {
    fn html(&mut self, html: &mut Builder) -> impl IntoView {}
}

pub struct Web;

impl Platform for Web {
    fn from_str(&mut self, s: &str) -> Box<dyn crate::AnyView> {
        Box::new(())
    }
}

pub fn run(content: impl IntoView) {
    let cx = WebContext::new();
    cx.enter();

    let mut composition = Tree::new(Web, content);
    composition.build();

    let event_loop = EventLoop::new().unwrap();
    let window =
        WindowBuilder::new()
            .with_inner_size(LogicalSize::new(800, 800))
            .build(&event_loop)
            .unwrap();

    #[allow(unused_mut)]
    let mut builder = WebViewBuilder::new(&window);
    let _webview = builder
        .with_url("https://tauri.app")
        .unwrap()
        .build()
        .unwrap();

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
