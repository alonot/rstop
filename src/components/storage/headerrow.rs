use std::sync::{Arc, Mutex};

use cncurses::{
    components::{button::Button, text::Text, view::View},
    interfaces::{Component, ComponentBuilder},
    styles::CSSStyle,
};
use ncurses::{COLOR_CYAN, COLOR_GREEN, COLOR_MAGENTA, COLOR_RED, COLOR_YELLOW};

pub struct HeaderRow {
    pub name: String,
    pub size: String,
    pub percent: String,
    pub btn1_name: String,
    pub onclick1: Option<Arc<dyn Fn() + Send + Sync>>,
    pub btn2_name: String,
    pub onclick2: Option<Arc<dyn Fn() + Send + Sync>>,
}


impl Component for HeaderRow {
    fn __call__(&mut self) -> Arc<Mutex<dyn Component>> {
        let onclick1 = self.onclick1.clone();
        let onclick2 = self.onclick2.clone();

        let btn1 = match onclick1 {
            Some(clk) => Button::new(
                Text::new(
                    self.btn1_name.to_string(),
                    CSSStyle {
                        width: "100%",
                        height: "100%",
                        color: COLOR_MAGENTA,
                        ..Default::default()
                    },
                )
                .build(),
                CSSStyle {
                    flex: 1,
                    height: "100%",
                    ..Default::default()
                },
                move |e| {
                    e.stop_propogation();
                    clk()
                },
            )
            .build(),
            None => Text::new(
                self.btn1_name.to_string(),
                CSSStyle {
                    flex: 1,
                    height: "100%",
                    color: COLOR_MAGENTA,
                    ..Default::default()
                },
            )
            .build(),
        };
        let btn2 = match onclick2 {
            Some(clk) => Button::new(
                Text::new(
                    self.btn2_name.to_string(),
                    CSSStyle {
                        width: "100%",
                        height: "100%",
                        color: COLOR_YELLOW,
                        ..Default::default()
                    },
                )
                .build(),
                CSSStyle {
                    flex: 1,
                    height: "100%",
                    ..Default::default()
                },
                move |e| {
                    e.stop_propogation();
                    clk()
                },
            )
            .build(),
            None => Text::new(
                self.btn2_name.to_string(),
                CSSStyle {
                    flex: 1,
                    height: "100%",
                    color: COLOR_YELLOW,
                    ..Default::default()
                },
            )
            .build(),
        };

        View::new(
            vec![
                Text::new(
                    self.name.to_string(),
                    CSSStyle {
                        flex: 1,
                        color: COLOR_CYAN,
                        ..Default::default()
                    },
                )
                .build(),
                Text::new(
                    self.size.to_string(),
                    CSSStyle {
                        flex: 1,
                        color: COLOR_RED,
                        ..Default::default()
                    },
                )
                .build(),
                Text::new(
                    self.percent.to_string(),
                    CSSStyle {
                        flex: 1,
                        color: COLOR_GREEN,
                        ..Default::default()
                    },
                )
                .build(),
                btn1,
                btn2
            ],
            CSSStyle {
                height: "1",
                width: "100%",
                flex_direction: "horizontal",
                ..Default::default()
            },
        )
        .build()
    }
}
