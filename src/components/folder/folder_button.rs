use std::sync::Arc;

use cncurses::{
    components::{button::Button, text::Text},
    interfaces::{Component, ComponentBuilder, Document},
    styles::CSSStyle, use_state, LOGLn,
};
use ncurses::COLOR_MAGENTA;

pub struct FolderButton {
    pub f: Arc<String>,
    pub select_folder: Arc<dyn Fn(String) + Send + Sync>,
}

impl Component for FolderButton {
    fn __call__(&mut self) -> std::sync::Arc<std::sync::Mutex<dyn Component>> {

        let (color, setcolor) = use_state(-1);
        let set_color_c = setcolor.clone();
        let set_color_c1 = setcolor.clone();
        
        let f = self.f.clone();
        let f_c = self.f.clone();
        let select_folder_c = self.select_folder.clone();
        Button::new(
            Text::new(
                f.to_string(),
                CSSStyle {
                    ..Default::default()
                },
            )
            .build(),
            CSSStyle {
                taborder: 0,
                background_color: color,
                ..Default::default()
            },
            move |_e| {
                select_folder_c(f_c.to_string());
            },
        )
        .onfocus(move |_e| {
            set_color_c(Document::get_color(150, 10, 100));
        })
        .onunfocus(move |_e| {
            set_color_c1(-1);
        })
        .build()
    }
}
