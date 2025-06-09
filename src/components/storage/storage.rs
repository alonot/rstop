use cncurses::{
    components::{text::Text, view::View},
    interfaces::{Component, ComponentBuilder, Document, EVENT},
    styles::CSSStyle,
    use_state, LOGLn,
};
use ncurses::{COLOR_BLACK, COLOR_MAGENTA, COLOR_RED, KEY_BTAB};

pub struct StorageWin;

impl Component for StorageWin {
    fn __call__(&mut self) -> std::sync::Arc<std::sync::Mutex<dyn Component>> {
        View::new(
            vec![Text::new(
                "Storage".to_string(),
                CSSStyle {
                    color: Document::get_color(255, 110, 0),
                    left: "2",
                    top: "-1",
                    position: "relative",
                    ..Default::default()
                },
            )
            .build()],
            CSSStyle {
                flex: 3,
                width: "100%",
                border: 1,
                boxsizing: "border-box",
                background_color: -1,
                border_color: COLOR_RED,
                // taborder: 0,
                ..Default::default()
            },
        )
        .build()
    }
}
