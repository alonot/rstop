use cncurses::{
    components::view::View, interfaces::{Component, ComponentBuilder, EVENT}, styles::CSSStyle, use_state, LOGLn
};
use ncurses::{COLOR_BLACK, COLOR_MAGENTA, COLOR_RED, KEY_BTAB};

pub struct FileInfoWin;

impl Component for FileInfoWin {
    fn __call__(&mut self) -> std::sync::Arc<std::sync::Mutex<dyn Component>> {

        View::new(
            vec![],
            CSSStyle {
                flex: 1,
                width: "100%",
                border: 1,
                boxsizing: "border-box",
                background_color: -1,
                border_color: COLOR_RED,
                taborder: 0,
                ..Default::default()
            },
        )
        .build()
    }
}