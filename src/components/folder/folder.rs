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

use crate::{components::folder::{folder_button::FolderButton, folder_info::FolderInfo}, models::data_models::DirInfo, Message};

pub struct FolderWin {
    pub setpath: Arc<dyn Fn(String) + Send + Sync>,
    pub select_folder: Arc<dyn Fn(String) + Send + Sync>,
    pub folders: Vec<Arc<String>>,
    pub path: Arc<PathBuf>,
    pub dir_info: Arc<DirInfo>,
    pub loading: bool,
}

impl Component for FolderWin {
    

    fn __call__(&mut self) -> std::sync::Arc<std::sync::Mutex<dyn Component>> {
        let mut children = vec![
            FolderInfo {
                path: self.path.clone(),
                setpath: self.setpath.clone(),
                dir_info: self.dir_info.clone(),
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

        if self.loading {
            children.push(
                Text::new(
                    "Loading".to_string(),
                    CSSStyle {
                        width: "100%",
                        flex_grow: true,
                        overflow: "scroll",
                        padding: "50% 0 47% 0",
                        top : "6",
                        left: "-1",
                        position: "relative",
                        z_index: -1,
                        // boxsizing: "border-box",
                        background_color: -1,
                        ..Default::default()
                    },
                )
                .build(),
            );
        } else {
            let folder_btns = self
                .folders
                .iter()
                .map(|f| {
                    let select_folder_c = self.select_folder.clone();
                    FolderButton{
                        f: f.clone(),
                        select_folder: select_folder_c.clone()
                    }.build()
                })
                .collect();

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
        }

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
    
    fn __key__(&self) -> Option<String> {
        let mut str = String::new();
        let str1 = self.folders.iter().fold( str, |mut f,v| {
            f.push_str(v);
            f
        });
        Some(str1)
    }
}
