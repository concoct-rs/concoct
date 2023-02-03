use concoct::composable::material::button;
use concoct::composable::state::State;
use concoct::composable::{container, remember, state, stream};
use concoct::modify::keyboard_input::KeyboardHandler;
use concoct::{render::run, Modifier};
use futures::{Stream, StreamExt};
use rust_decimal::Decimal;
use serde::Deserialize;
use std::time::Duration;
use taffy::style::{AlignItems, FlexDirection};
use tokio::time::interval;
use tokio_stream::wrappers::IntervalStream;
use winit::event::{ElementState, VirtualKeyCode};

mod currency;
use currency::{currency_text, Currency};

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

#[derive(Clone, Copy)]
enum Display {
    Balance,
    Send,
}

#[track_caller]
fn app() {
    container(
        Modifier::default()
            .align_items(AlignItems::Center)
            .flex_direction(FlexDirection::Column)
            .flex_grow(1.),
        || {
            let display = state(|| Display::Balance);
            let currency = state(|| Currency::Bitcoin);

            let rate = state(|| Decimal::ZERO);
            remember(&[], || {
                stream(make_stream(), move |value| {
                    *rate.get().as_mut() = value;
                })
            });

            match display.get().cloned() {
                Display::Balance => {
                    let balance = state(|| String::from("100"));

                    currency_text(currency, balance, rate);

                    container(
                        Modifier::default().flex_direction(FlexDirection::Row),
                        move || {
                            button("Send", move || {
                                *display.get().as_mut() = Display::Send;
                            });
                            button("Request", || {
                                
                            });
                        },
                    )
                }
                Display::Send => {
                    let amount = state(|| String::from(""));

                    container(
                        Modifier::default()
                            .keyboard_handler(CurrencyInputKeyboardHandler::new(amount)),
                        move || {
                            currency_text(currency, amount, rate);
                        },
                    );
                }
            }
        },
    );
}

#[tokio::main]
async fn main() {
    run(app)
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
