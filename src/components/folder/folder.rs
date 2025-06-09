use std::{
    path::{Path, PathBuf},
    sync::{mpsc::Sender, Arc, Mutex},
};

use cncurses::{
    components::{button::Button, text::Text, view::View},
    interfaces::{Component, ComponentBuilder, EVENT},
    styles::CSSStyle,
    use_state, LOGLn,
};
use ncurses::{COLOR_BLACK, COLOR_MAGENTA, COLOR_RED, COLOR_YELLOW, KEY_BTAB};

use crate::{
    components::folder::folder_info::FolderInfo, Message
};

pub struct FolderWin {
    pub setpath: Arc<Mutex<dyn FnMut(String)>>,
    pub select_folder: Arc<Mutex<dyn FnMut(String)>>,
    pub folders: Vec<Arc<String>>,
    pub path: Box<PathBuf>,
    pub size: u64,
}

impl Component for FolderWin {
    fn __call__(&mut self) -> std::sync::Arc<std::sync::Mutex<dyn Component>> {
        let mut children = vec![
            FolderInfo {
                path: self.path.clone(),
                setpath: self.setpath.clone(),
                size: self.size,
            }
            .build(),
            Text::new(
                "Folder".to_string(),
                CSSStyle {
                    color: COLOR_RED,
                    left: "2",
                    top: "-1",
                    position: "relative",
                    ..Default::default()
                },
            )
            .build(),
        ];
        let mut folder_btns = vec![];
        for i in 0..20 {
            let f_btns: Vec<Arc<Mutex<dyn Component>>> = self
                .folders
                .iter()
                .enumerate()
                .map(|(idx, f)| {
                    let select_folder_c = self.select_folder.clone();
                    let f_c = f.clone();
                    Button::new(
                        Some(f.to_string()),
                        Text::new(
                            format!("{}_{}",f, i * 3 + idx),
                            CSSStyle {
                                ..Default::default()
                            },
                        )
                        .build(),
                        CSSStyle {
                            ..Default::default()
                        },
                        move |_e| {
                            select_folder_c.lock().unwrap()(f_c.to_string());
                        },
                    )
                    .build()
                })
                .collect();

            folder_btns.extend(f_btns.clone());
        }

        children.push(
            View::new(
                folder_btns,
                CSSStyle {
                    flex_grow: true,
                    overflow: "scroll",
                    ..Default::default()
                },
            )
            .build(),
        );

        View::new(
            children,
            CSSStyle {
                flex: 1,
                height: "100%",
                border: 1,
                // padding: "1 1 1 1",
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

unsafe impl Send for FolderWin {}
