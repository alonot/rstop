use std::sync::{Arc, Mutex};

use cncurses::{
    components::{text::Text, view::View},
    interfaces::{Component, ComponentBuilder, Document},
    styles::CSSStyle, use_state,
};
use ncurses::{COLOR_CYAN, COLOR_YELLOW};

use crate::models::data_models::{DIRENTRYHolder, Dirent, DIRENTRY};

pub struct DirEntryRow {
    pub name: String,
    pub size: String,
    pub percent: f32,
    pub set_file_info_direntry: Arc<dyn Fn() + Send + Sync>
}

impl Component for DirEntryRow {
    fn __call__(&mut self) -> std::sync::Arc<std::sync::Mutex<dyn Component>> {
        let (color, setcolor) = use_state(-1);
        let set_color_c = setcolor.clone();
        let set_dirent = self.set_file_info_direntry.clone();
        let set_dirent_c = self.set_file_info_direntry.clone();
        View::new(
            vec![
                Text::new(
                    self.name.to_string(),
                    CSSStyle {
                        flex: 1,
                        flex_direction: "horizontal",
                        overflow: "scroll",
                        color: color,
                        padding: "0 0 0 1",
                        boxsizing: "border-box",
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
                taborder: 0,
                flex_direction: "horizontal",
                ..Default::default()
            },
        )
        .onfocus(move |_e| {
            set_color_c(Document::get_color(120, 120, 40));
        })
        .onunfocus(move |_e| {
            setcolor(-1);
        })
        .onclick(move |e| {
            set_dirent();
            e.prevent_default();
        }, false)
        .onenter(move |e| {
            set_dirent_c();
            e.prevent_default();
        })
        .build()
    }
}
