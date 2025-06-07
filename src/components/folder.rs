use std::{
    path::{Path, PathBuf},
    sync::{mpsc::Sender, Arc, Mutex},
};

use cncurses::{
    components::{text::Text, view::View},
    interfaces::{Component, ComponentBuilder, EVENT},
    styles::CSSStyle,
    use_state, LOGLn,
};
use ncurses::{COLOR_BLACK, COLOR_MAGENTA, COLOR_RED, COLOR_YELLOW, KEY_BTAB};

use crate::{components::folder_name::FolderName, Message};

pub struct FolderWin {
    pub setpath: Arc<Mutex<dyn FnMut(String)>>,
    pub path: Box<PathBuf>,
}

impl Component for FolderWin {
    fn __call__(&mut self) -> std::sync::Arc<std::sync::Mutex<dyn Component>> {

        View::new(
            vec![
                Text::new(
                    "Folder".to_string(),
                    CSSStyle {
                        color: COLOR_YELLOW,
                        left: "2",
                        position: "relative",
                        ..Default::default()
                    },
                )
                .build(),
                FolderName {
                    path: self.path.clone(),
                    setpath: self.setpath.clone(),
                }
                .build(),
            ],
            CSSStyle {
                flex: 1,
                boxsizing: "border-box",
                height: "100%",
                border: 1,
                background_color: -1,
                border_color: COLOR_RED,
                taborder: 0,
                ..Default::default()
            },
        )
        .build()
    }
}

unsafe impl Send for FolderWin {}