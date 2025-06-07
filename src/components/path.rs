use std::sync::{Arc, Mutex};

use cncurses::{
    components::{button::Button, text::Text, view::View},
    interfaces::{Component, ComponentBuilder},
    styles::CSSStyle, LOGLn,
};
use ncurses::COLOR_YELLOW;

pub struct PathComp
{
    pub path: String,
    pub name: String,
    pub setpath: Arc<Mutex<dyn FnMut(String)>>,
}

impl Component for PathComp
{
    fn __call__(&mut self) -> std::sync::Arc<std::sync::Mutex<dyn Component>> {

        let path = self.path.clone();
        let set_path = self.setpath.clone();

        Button::new(
            Some(self.path.clone()),
            Text::new(
                self.name.clone(),
                CSSStyle {
                    color:COLOR_YELLOW,
                    ..Default::default()
                },
            )
            .build(),
            CSSStyle {
                ..Default::default()
            },
            move |_e| {

                set_path.lock().unwrap()(path.clone());
            },
        )
        .build()
    }
}

unsafe impl Send for PathComp {}
