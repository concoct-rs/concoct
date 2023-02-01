//! Render engine

use crate::{Composer, Semantics};
use accesskit::{NodeId, Role};
use accesskit_winit::{ActionRequestEvent, Adapter};
use gl::types::*;
use glutin::{
    config::{Config, ConfigTemplateBuilder},
    context::{ContextApi, ContextAttributesBuilder, PossiblyCurrentContext},
    display::GetGlDisplay,
    prelude::{GlConfig, GlDisplay, NotCurrentGlContextSurfaceAccessor, PossiblyCurrentGlContext},
    surface::{GlSurface, Surface, SurfaceAttributesBuilder, WindowSurface},
};
use glutin_winit::DisplayBuilder;
use raw_window_handle::HasRawWindowHandle;
use skia_safe::{
    gpu::{gl::FramebufferInfo, BackendRenderTarget, FlushInfo, SurfaceOrigin},
    Color, ColorType,
};
use std::{
    ffi::CString,
    num::{NonZeroU128, NonZeroU32},
    sync::Arc,
};
use taffy::{prelude::Size, style::Style};
use winit::{
    event::{Event, KeyboardInput, WindowEvent},
    event_loop::{ControlFlow, EventLoopBuilder},
    window::{Window, WindowBuilder},
};

#[derive(Debug)]
pub enum UserEvent {
    ActionRequest(ActionRequestEvent),
}

impl From<ActionRequestEvent> for UserEvent {
    fn from(value: ActionRequestEvent) -> Self {
        Self::ActionRequest(value)
    }
}

pub struct GlWindow {
    // XXX the surface must be dropped before the window.
    pub surface: Surface<WindowSurface>,
    pub window: Window,
}

impl GlWindow {
    pub fn new(window: Window, config: &Config) -> Self {
        let (width, height): (u32, u32) = window.inner_size().into();
        let raw_window_handle = window.raw_window_handle();
        let attrs = SurfaceAttributesBuilder::<WindowSurface>::new().build(
            raw_window_handle,
            NonZeroU32::new(width).unwrap(),
            NonZeroU32::new(height).unwrap(),
        );

        let surface = unsafe {
            config
                .display()
                .create_window_surface(config, &attrs)
                .unwrap()
        };

        Self { window, surface }
    }
}

// Guarantee the drop order inside the FnMut closure. `WindowedContext` _must_ be dropped after
// `DirectContext`.
//
// https://github.com/rust-skia/rust-skia/issues/476
struct Env {
    surface: skia_safe::Surface,
    gr_context: skia_safe::gpu::DirectContext,
    windowed_context: GlWindow,
    gl_context: PossiblyCurrentContext,
}

#[track_caller]
pub fn run(view_builder: fn()) {
    let event_loop_builder = EventLoopBuilder::with_user_event();
    run_with_event_loop_builder(view_builder, event_loop_builder)
}

#[track_caller]
pub fn run_with_event_loop_builder(
    view_builder: fn(),
    mut event_loop_builder: EventLoopBuilder<UserEvent>,
) {
    view_builder();

    let event_loop = event_loop_builder.build();

    let window_builder = if cfg!(wgl_backend) {
        Some(WindowBuilder::new())
    } else {
        None
    };

    let template = ConfigTemplateBuilder::new().with_alpha_size(8);
    let display_builder = DisplayBuilder::new().with_window_builder(window_builder);

    let (mut window, gl_config) = display_builder
        .build(&event_loop, template, |configs| {
            configs
                .reduce(|accum, config| {
                    if config.num_samples() > accum.num_samples() {
                        config
                    } else {
                        accum
                    }
                })
                .unwrap()
        })
        .unwrap();

    let raw_window_handle = window.as_ref().map(|window| window.raw_window_handle());
    let gl_display = gl_config.display();
    let context_attributes = ContextAttributesBuilder::new().build(raw_window_handle);
    let fallback_context_attributes = ContextAttributesBuilder::new()
        .with_context_api(ContextApi::Gles(None))
        .build(raw_window_handle);
    let mut not_current_gl_context = Some(unsafe {
        gl_display
            .create_context(&gl_config, &context_attributes)
            .unwrap_or_else(|_| {
                gl_display
                    .create_context(&gl_config, &fallback_context_attributes)
                    .expect("failed to create context")
            })
    });

    let mut env = None;
    let mut scale_factor = 1.;

    let mut cursor = None;

    let _root = Arc::new(accesskit::Node {
        role: Role::Window,
        children: vec![],
        name: Some("WINDOW_TITLE".into()),
        ..Default::default()
    });
    let proxy = event_loop.create_proxy();

    let mut adapter = None;

    const WINDOW_ID: NodeId = NodeId(unsafe { NonZeroU128::new_unchecked(1) });

    let mut semantics = Semantics::default();

    event_loop.run(move |event, window_target, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            Event::Resumed => {
                let window = window.take().unwrap_or_else(|| {
                    let window_builder = WindowBuilder::new().with_transparent(true);
                    glutin_winit::finalize_window(window_target, window_builder, &gl_config)
                        .unwrap()
                });

                scale_factor = window.scale_factor();

                let tree_update = semantics.tree_update();
                adapter = Some(Adapter::new(&window, move || tree_update, proxy.clone()));

                let gl_window = GlWindow::new(window, &gl_config);
                let gl_context = not_current_gl_context
                    .take()
                    .unwrap()
                    .make_current(&gl_window.surface)
                    .unwrap();
                gl::load_with(|s| gl_display.get_proc_address(&CString::new(s).unwrap()));

                let mut gr_context = skia_safe::gpu::DirectContext::new_gl(None, None).unwrap();
                let fb_info = {
                    let mut fboid: GLint = 0;
                    unsafe { gl::GetIntegerv(gl::FRAMEBUFFER_BINDING, &mut fboid) };

                    FramebufferInfo {
                        fboid: fboid.try_into().unwrap(),
                        format: skia_safe::gpu::gl::Format::RGBA8.into(),
                    }
                };
                let surface =
                    create_surface(&gl_window.window, &gl_config, &fb_info, &mut gr_context);

                env = Some(Env {
                    surface,
                    gr_context,
                    windowed_context: gl_window,
                    gl_context,
                });
            }
            Event::Suspended => {
                let env = env.take().unwrap();
                assert!(not_current_gl_context
                    .replace(env.gl_context.make_not_current().unwrap())
                    .is_none());
            }
            Event::LoopDestroyed => {}
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::Resized(physical_size) => {
                    if let Some(env) = &env {
                        env.windowed_context.surface.resize(
                            &env.gl_context,
                            physical_size.width.try_into().unwrap(),
                            physical_size.width.try_into().unwrap(),
                        );
                    }
                }
                WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                WindowEvent::CursorMoved { position, .. } => {
                    cursor = Some(position);
                }
                WindowEvent::MouseInput {
                    device_id: _,
                    state: _,
                    button: _,
                    ..
                } => {}
                WindowEvent::Touch(_touch) => {}
                WindowEvent::KeyboardInput {
                    input:
                        KeyboardInput {
                            state: _,
                            virtual_keycode: _,
                            ..
                        },
                    ..
                } => {}
                _ => (),
            },
            Event::RedrawRequested(_) => {
                if let Some(env) = &mut env {
                    Composer::with(|composer| composer.borrow_mut().semantics(&mut semantics));

                    let root = semantics
                        .taffy
                        .new_with_children(
                            Style::DEFAULT,
                            semantics.layout_children.first().unwrap(),
                        )
                        .unwrap();
                    taffy::compute_layout(&mut semantics.taffy, root, Size::MAX_CONTENT).unwrap();

                    let mut canvas = env.surface.canvas();
                    canvas.clear(Color::WHITE);

                    Composer::with(|composer| composer.borrow_mut().paint(&semantics, &mut canvas));

                    env.gr_context.flush(&FlushInfo::default());
                    env.windowed_context
                        .surface
                        .swap_buffers(&env.gl_context)
                        .unwrap();
                }
            }
            Event::MainEventsCleared => {}
            Event::UserEvent(user_event) => match user_event {
                UserEvent::ActionRequest(action_request) => {
                    dbg!(action_request);
                }
            },
            _ => (),
        }
    });
}

fn create_surface(
    window: &Window,
    config: &Config,
    fb_info: &FramebufferInfo,
    gr_context: &mut skia_safe::gpu::DirectContext,
) -> skia_safe::Surface {
    let size = window.inner_size();
    let backend_render_target = BackendRenderTarget::new_gl(
        (
            size.width.try_into().unwrap(),
            size.height.try_into().unwrap(),
        ),
        1,
        config.stencil_size().try_into().unwrap(),
        *fb_info,
    );
    skia_safe::Surface::from_backend_render_target(
        gr_context,
        &backend_render_target,
        SurfaceOrigin::BottomLeft,
        ColorType::RGBA8888,
        None,
        None,
    )
    .unwrap()
}
