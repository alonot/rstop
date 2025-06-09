use std::sync::Arc;

use cncurses::{components::{text::Text, view::View}, interfaces::{Component, ComponentBuilder, Document}, styles::CSSStyle};
use ncurses::{COLOR_RED, COLOR_YELLOW};



pub struct FileInfoValue {
    pub field_name:String ,
    pub value: String
}

impl Component for FileInfoValue {
    fn __call__(&mut self) -> std::sync::Arc<std::sync::Mutex<dyn Component>> {
        View::new(
            vec![
                Text::new(
                    self.field_name.to_string(),
                    CSSStyle{
                        color: COLOR_YELLOW,
                        flex: 1,
                        ..Default::default()
                    }
                ).build(),
                Text::new(
                    self.value.to_string(),
                    CSSStyle{
                        flex:2,
                        color: Document::get_color(210, 160, 120),
                        ..Default::default()
                    }
                ).build()
            ],
            CSSStyle{
                height: "1",
                width: "50%",
                flex_direction: "horizontal",
                ..Default::default()
            }
        ).build()
    }
}