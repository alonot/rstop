use std::sync::{Arc, Mutex};

use cncurses::{
    components::{text::Text, view::View},
    interfaces::{Component, ComponentBuilder},
    styles::CSSStyle,
};
use ncurses::COLOR_YELLOW;

use crate::models::data_models::{DIRENTRYHolder, DIRENTRY};

pub struct DirEntryRow {
    pub name: String,
    pub size: String,
    pub percent: f32,
    pub set_file_info_direntry: Arc<dyn Fn(String) + Send + Sync>
}

impl Component for DirEntryRow {
    fn __call__(&mut self) -> std::sync::Arc<std::sync::Mutex<dyn Component>> {
        let set_dirent = self.set_file_info_direntry.clone();
        let name = self.name.clone();
        View::new(
            vec![
                Text::new(
                    self.name.to_string(),
                    CSSStyle {
                        flex: 1,
                        ..Default::default()
                    },
                )
                .build(),
                Text::new(
                    self.size.to_string(),
                    CSSStyle {
                        flex: 1,
                        ..Default::default()
                    },
                )
                .build(),
                Text::new(
                    format!("{}%", self.percent),
                    CSSStyle {
                        flex: 1,
                        ..Default::default()
                    },
                )
                .build(),
                View::new(
                    vec![View::new(
                        vec![],
                        CSSStyle {
                            height:"100%",
                            background_color: COLOR_YELLOW,
                            width: &format!("{}%", self.percent),
                            ..Default::default()
                        },
                    )
                    .build()],
                    CSSStyle {
                        flex: 2,
                        height: "100%",
                        ..Default::default()
                    },
                )
                .build(),
            ],
            CSSStyle {
                height: "1",
                width: "100%",
                flex_direction: "horizontal",
                ..Default::default()
            },
        )
        .onclick(move |e| {
            set_dirent(name.clone());
            e.prevent_default();
        }, false)
        .build()
    }
}
