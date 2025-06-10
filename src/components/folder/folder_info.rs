use std::{path::PathBuf, sync::{Arc, Mutex}};

use cncurses::{components::{text::Text, view::View}, interfaces::{Component, ComponentBuilder}, styles::CSSStyle};
use ncurses::{COLOR_BLUE, COLOR_CYAN};

use crate::{components::folder::folder_name::FolderName, utils::total_size_to_string};




pub struct FolderInfo {
    pub path: Box<PathBuf>,
    pub setpath: Arc<dyn Fn(String) + Send + Sync>,
    pub size: u64,
}

impl Component for FolderInfo  {
    fn __call__(&mut self) -> Arc<Mutex<dyn Component>> {
        View::new(
            vec![
                FolderName{ path: self.path.clone(), setpath: self.setpath.clone() }.build(),
                Text::new(
                    format!("Total Size: {}", total_size_to_string(self.size)),
                    CSSStyle {
                        width:"100%",
                        height: "1",
                        ..Default::default()
                    }
                ).build(),
            ], CSSStyle {
                width: "100%",
                height: "4",
                margin: "-1 0 -1 0",
                border: 1,
                border_color: COLOR_BLUE,
                ..Default::default()
            }).build()
    }
}