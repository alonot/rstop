use std::{path::PathBuf, sync::{Arc, Mutex}};

use cncurses::{components::{text::Text, view::View}, interfaces::{Component, ComponentBuilder, Document}, styles::CSSStyle, LOGLn};
use ncurses::{COLOR_BLUE, COLOR_CYAN, COLOR_WHITE};

use crate::{components::folder::folder_name::FolderName, models::data_models::DirInfo, utils::total_size_to_string};




pub struct FolderInfo {
    pub path: Arc<PathBuf>,
    pub setpath: Arc<dyn Fn(String) + Send + Sync>,
    pub dir_info: Arc<DirInfo>,
}

impl Component for FolderInfo  {
    fn __call__(&mut self) -> Arc<Mutex<dyn Component>> {
        let val = &self.dir_info;
        
        View::new(
            vec![
                FolderName{ path: self.path.clone(), setpath: self.setpath.clone() }.build(),
                Text::new(
                    format!("Total Size: {}", total_size_to_string(val.total_size)),
                    CSSStyle {
                        width:"100%",
                        height: "1",
                        color: Document::get_color(255, 140, 40),
                        ..Default::default()
                    }
                ).build(),
                Text::new(
                    format!("Number Of Files: {}", val.no_files),
                    CSSStyle {
                        width:"100%",
                        height: "1",
                        color: COLOR_WHITE,
                        ..Default::default()
                    }
                ).build(),
                Text::new(
                    format!("Number of Folders: {}", val.no_folders),
                    CSSStyle {
                        width:"100%",
                        height: "1",
                        color: Document::get_color(100, 255, 150),
                        ..Default::default()
                    }
                ).build(),
            ], CSSStyle {
                width: "100%",
                height: "6",
                margin: "-1 0 -1 0",
                border: 1,
                border_color: COLOR_BLUE,
                ..Default::default()
            }).build()
    }
}