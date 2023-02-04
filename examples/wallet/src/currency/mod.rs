use concoct::{
    composable::{container, material::button, state::State},
    Modifier,
};
use rust_decimal::Decimal;
use std::{
    fmt::{self, Write},
    ops::Not,
};
use taffy::{
    prelude::{Rect, Size},
    style::{AlignItems, Dimension, FlexDirection},
};

mod flex_text;
use flex_text::flex_text;

#[track_caller]
pub fn currency_text(currency: State<Currency>, value: State<String>, rate: State<Decimal>) {
    container(
        Modifier::default()
            .align_items(AlignItems::Center)
            .flex_direction(FlexDirection::Column)
            .size(Size {
                width: Dimension::Percent(1.),
                height: Dimension::Points(400.),
            }),
        move || {
            container(
                Modifier::default()
                    .align_items(AlignItems::Center)
                    .flex_direction(FlexDirection::Column)
                    .margin(Rect::from_points(20., 20., 50., 50.))
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
}

#[derive(Clone, Copy)]
pub enum Currency {
    Bitcoin,
    USD,
}

impl Currency {
    pub fn convert(self, value: &str, rate: Decimal) -> Decimal {
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
