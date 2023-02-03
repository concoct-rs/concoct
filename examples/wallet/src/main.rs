use accesskit::{Node, NodeId, Role};
use concoct::composable::material::button;
use concoct::composable::{remember, stream};
use concoct::modify::keyboard_input::KeyboardHandler;
use concoct::state::{state, State};
use concoct::{composer::Composer, semantics::LayoutNode, Semantics, Widget};
use concoct::{container, render::run, Modifier};
use futures::{stream, Stream, StreamExt};
use rust_decimal::Decimal;
use serde::Deserialize;
use skia_safe::RGB;
use skia_safe::{Color4f, ColorSpace, Font, FontStyle, Paint, TextBlob, Typeface};
use std::fmt::{self, Write};
use std::ops::Not;
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::Duration;
use std::{any, panic::Location, sync::Arc};
use taffy::prelude::Rect;
use taffy::style::{AlignItems, Dimension, FlexDirection};
use taffy::{
    prelude::{AvailableSpace, Size},
    style::Style,
};
use tokio::time::interval;
use tokio_stream::wrappers::IntervalStream;
use winit::event::{ElementState, VirtualKeyCode};

#[derive(Clone, Copy)]
enum Currency {
    Bitcoin,
    USD,
}

impl Currency {
    fn convert(self, value: &str, rate: Decimal) -> Decimal {
        let value: Decimal = value.parse().unwrap_or_default();
        match self {
            Currency::Bitcoin => (value * rate).round_dp(2),
            Currency::USD => (value / rate).round_dp(8),
        }
    }
}

impl Not for Currency {
    type Output = Self;

    fn not(self) -> Self::Output {
        match self {
            Self::Bitcoin => Self::USD,
            Self::USD => Self::Bitcoin,
        }
    }
}

impl fmt::Display for Currency {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let c = match self {
            Self::Bitcoin => '₿',
            Self::USD => '$',
        };
        f.write_char(c)
    }
}

#[derive(Deserialize)]
struct RateResponseData {
    #[serde(rename = "rateUsd")]
    rate: Decimal,
}

#[derive(Deserialize)]
struct RateResponse {
    data: RateResponseData,
}

async fn make_stream() -> impl Stream<Item = Decimal> {
    Box::pin(
        IntervalStream::new(interval(Duration::from_secs(5))).then(|_| async {
            let res: RateResponse = reqwest::get("https://api.coincap.io/v2/rates/bitcoin")
                .await
                .unwrap()
                .json()
                .await
                .unwrap();
            res.data.rate
        }),
    )
}

fn app() {
    container(
        Modifier::default()
            .align_items(AlignItems::Center)
            .flex_direction(FlexDirection::Column)
            .flex_grow(1.),
        || {
            let currency = state(|| Currency::Bitcoin);
            let value = state(|| String::from(""));
            let rate = state(|| Decimal::ZERO);

            state(|| {
                stream(make_stream(), move |value| {
                    dbg!(value);
                    *rate.get().as_mut() = value;
                })
            });

            container(
                Modifier::default()
                    .align_items(AlignItems::Center)
                    .flex_direction(FlexDirection::Column)
                    .size(Size {
                        width: Dimension::Percent(1.),
                        height: Dimension::Points(400.),
                    })
                    .keyboard_handler(CurrencyInputKeyboardHandler::new(value)),
                move || {
                    container(
                        Modifier::default()
                            .align_items(AlignItems::Center)
                            .flex_direction(FlexDirection::Column)
                            .margin(Rect::from_points(20., 20., 20., 20.))
                            .size(Size {
                                width: Dimension::Percent(1.),
                                height: Dimension::Points(200.),
                            }),
                        move || {
                            flex_text(format!(
                                "{}{}",
                                currency.get().cloned(),
                                value.get().as_ref()
                            ));
                        },
                    );

                    button(
                        format!(
                            "{}{}",
                            !currency.get().cloned(),
                            currency
                                .get()
                                .cloned()
                                .convert(&*value.get().as_ref(), rate.get().cloned())
                        ),
                        move || {
                            let converted = currency
                                .get()
                                .cloned()
                                .convert(&*value.get().as_ref(), rate.get().cloned())
                                .to_string();
                            *value.get().as_mut() = converted;
                            *currency.get().as_mut() = !currency.get().cloned();
                        },
                    );
                },
            );

            container(
                Modifier::default().flex_direction(FlexDirection::Row),
                || {
                    button("Send", || {
                        dbg!("press");
                    });
                    button("Request", || {
                        dbg!("press");
                    });
                },
            )
        },
    );
}

#[tokio::main]
async fn main() {
    run(app)
}

#[track_caller]
pub fn flex_text(string: impl Into<String>) {
    let location = Location::caller();
    Composer::with(|composer| {
        let mut cx = composer.borrow_mut();
        let id = cx.id(location);

        let typeface = Typeface::new("Noto Sans", FontStyle::bold()).unwrap();

        if let Some(widget) = cx.get_mut::<TextWidget>(&id) {
            widget.text = string.into();
            cx.children.push(id);
        } else {
            let widget = TextWidget {
                text: string.into(),
                node_id: None,
                layout_id: None,
                typeface,
                font_size: Arc::new(AtomicU32::new(400)),
            };
            cx.insert(id, widget, None);
        }
    })
}

pub struct CurrencyInputKeyboardHandler {
    value: State<String>,
}

impl CurrencyInputKeyboardHandler {
    fn new(value: State<String>) -> Self {
        Self { value }
    }

    fn push_char(&mut self, c: char) {
        if self.value.get().as_ref().parse::<f32>().unwrap_or_default() < 1000. {
            if let Some(pos) = self
                .value
                .get()
                .cloned()
                .chars()
                .rev()
                .position(|c| c == '.')
            {
                if pos <= 8 {
                    self.value.get().as_mut().push(c)
                }
            } else {
                self.value.get().as_mut().push(c)
            }
        }
    }
}

impl KeyboardHandler for CurrencyInputKeyboardHandler {
    fn handle_keyboard_input(&mut self, state: ElementState, virtual_keycode: VirtualKeyCode) {
        if state == ElementState::Pressed {
            match virtual_keycode {
                VirtualKeyCode::Key0 | VirtualKeyCode::Numpad0 => self.push_char('0'),
                VirtualKeyCode::Key1 | VirtualKeyCode::Numpad1 => self.push_char('1'),
                VirtualKeyCode::Back => {
                    self.value.get().as_mut().pop();
                }
                VirtualKeyCode::Period => {
                    if !self.value.get().as_ref().contains('.') {
                        self.value.get().as_mut().push('.');
                    }
                }
                _ => {}
            }
        }
    }
}

pub struct TextWidget {
    text: String,
    node_id: Option<NodeId>,
    layout_id: Option<LayoutNode>,
    typeface: Typeface,
    font_size: Arc<AtomicU32>,
}

impl Widget for TextWidget {
    fn layout(&mut self, semantics: &mut Semantics) {
        let font_size = self.font_size.clone();
        let typeface = self.typeface.clone();
        let text = self.text.clone();
        let on_measure = move |_known_dimensions, available_space: Size<AvailableSpace>| {
            let max_width = match available_space.width {
                AvailableSpace::Definite(px) => px,
                AvailableSpace::MaxContent => f32::MAX,
                AvailableSpace::MinContent => f32::MIN,
            };
            let max_height = match available_space.height {
                AvailableSpace::Definite(px) => px,
                AvailableSpace::MaxContent => f32::MAX,
                AvailableSpace::MinContent => f32::MIN,
            };

            let mut font_size_value = 400;
            let bounds = loop {
                let font = Font::new(&typeface, font_size_value as f32);
                let (_, bounds) = font.measure_str(&text, None);

                if bounds.width() <= max_width && bounds.height() <= max_height {
                    break bounds;
                }

                font_size_value -= 10;
            };

            font_size.store(font_size_value, Ordering::SeqCst);

            Size {
                width: bounds.width(),
                height: bounds.height(),
            }
        };

        if let Some(layout_id) = self.layout_id {
            semantics
                .taffy
                .set_measure(
                    layout_id,
                    Some(taffy::node::MeasureFunc::Boxed(Box::new(on_measure))),
                )
                .unwrap();

            semantics
                .layout_children
                .last_mut()
                .unwrap()
                .push(layout_id);
        } else {
            let layout_id = semantics.insert_layout_with_measure(
                Style {
                    padding: Rect {
                        top: Dimension::Points(40.),
                        left: Dimension::Undefined,
                        right: Dimension::Undefined,
                        bottom: Dimension::Points(40.),
                    },
                    ..Default::default()
                },
                on_measure,
            );
            self.layout_id = Some(layout_id);
        }
    }

    fn semantics(&mut self, semantics: &mut Semantics) {
        let node = Node {
            role: Role::StaticText,
            value: Some(self.text.clone().into_boxed_str()),
            ..Node::default()
        };

        if let Some(node_id) = self.node_id {
            semantics.update(node_id, node);
        } else {
            let id = semantics.insert(node);
            self.node_id = Some(id);
        }
    }

    fn paint(&mut self, semantics: &Semantics, canvas: &mut skia_safe::Canvas) {
        let paint = Paint::new(Color4f::from(RGB::from((0, 0, 0))), &ColorSpace::new_srgb());

        let font = Font::new(&self.typeface, self.font_size.load(Ordering::SeqCst) as f32);
        if let Some(text_blob) = TextBlob::new(&self.text, &font) {
            let layout = semantics.taffy.layout(self.layout_id.unwrap()).unwrap();
            let (_, bounds) = font.measure_str(&self.text, Some(&paint));

            canvas.draw_text_blob(
                &text_blob,
                (
                    layout.location.x - bounds.left + (layout.size.width - bounds.width()) / 2.,
                    layout.location.y + bounds.height(),
                ),
                &paint,
            );
        }
    }

    fn remove(&mut self, semantics: &mut Semantics) {
        if let Some(node_id) = self.node_id {
            semantics.remove(node_id);
        }
    }

    fn any(&self) -> &dyn any::Any {
        self
    }

    fn any_mut(&mut self) -> &mut dyn any::Any {
        self
    }
}
