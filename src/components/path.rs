use std::sync::{Arc, Mutex};

use cncurses::{
    components::{button::Button, text::Text, view::View}, interfaces::{Component, ComponentBuilder, Document}, styles::CSSStyle, use_state, LOGLn
};
use ncurses::{COLOR_CYAN, COLOR_MAGENTA, COLOR_YELLOW};

pub struct PathComp
{
    pub path: String,
    pub name: String,
    pub setpath: Arc<dyn Fn(String) + Send + Sync>,
}

impl Component for PathComp
{
    fn __call__(&mut self) -> std::sync::Arc<std::sync::Mutex<dyn Component>> {

        let (color, setcolor) = use_state(-1);
        let set_color_c = setcolor.clone();
        let set_color_c1 = setcolor.clone();

        let path = self.path.clone();
        let path_c = self.path.clone();
        let path_c1 = self.path.clone();
        let set_path = self.setpath.clone();

        Button::new(
            Text::new(
                self.name.clone(),
                CSSStyle {
                    color:Document::get_color(255, 0, 255),
                    ..Default::default()
                },
            )
            .build(),
            CSSStyle {
                background_color: color,
                taborder: 0,
                ..Default::default()
            },
            move |_e: &mut cncurses::interfaces::EVENT| {

                set_path(path.clone());
            },
        ).onfocus(move |_e| {
            set_color_c(Document::get_color(150, 10, 100));
        }).onunfocus(move |_e| {
            set_color_c1(-1);
        })
        .build()
    }
}
