use std::{os::unix::fs::FileTypeExt, sync::Arc};

use cncurses::{
    components::{text::Text, view::View}, interfaces::{Component, ComponentBuilder, Document, EVENT}, styles::CSSStyle, use_state, LOGLn
};
use ncurses::{COLOR_BLACK, COLOR_CYAN, COLOR_MAGENTA, COLOR_RED, KEY_BTAB};

use crate::{components::fileinfo::file_info_value::FileInfoValue, models::data_models::DIRENTRY, utils::system_time_to_string};

pub struct FileInfoWin{
    pub dir_entry: Arc<DIRENTRY>,
}

impl Component for FileInfoWin {
    fn __call__(&mut self) -> std::sync::Arc<std::sync::Mutex<dyn Component>> {

        let mut children = vec![
                Text::new(
                "File Info".to_string(),
                CSSStyle {
                    color: Document::get_color(255, 110, 0),
                    left: "2",
                    top: "-1",
                    position: "relative",
                    ..Default::default()
                },
            )
            .build(),
            ];
            
            children.extend([
                Text::new(self.dir_entry.name.to_string(), CSSStyle{color: COLOR_CYAN, width:"100%",..Default::default()}).build(),
                View::new(vec![
                    FileInfoValue{ field_name: "File Type:".to_string() , value: self.dir_entry.file_type.clone()}.build(),
                    FileInfoValue{ field_name: "Size:".to_string() , value:self.dir_entry.size.to_string() }.build(),
                    FileInfoValue{ field_name: "Permission:".to_string() , value:self.dir_entry.mode.to_string() }.build(),
                    FileInfoValue{ field_name: "Last Accessed:".to_string() , value: system_time_to_string(self.dir_entry.accessed) }.build(),
                    FileInfoValue{ field_name: "Created:".to_string() , value: system_time_to_string(self.dir_entry.created) }.build(),
                    FileInfoValue{ field_name: "Last Modified:".to_string() , value: system_time_to_string(self.dir_entry.modified) }.build(),
                    FileInfoValue{ field_name: "Number of Links:".to_string() , value:self.dir_entry.nlink.to_string() }.build(),
                    FileInfoValue{ field_name: "Dev:".to_string() , value:self.dir_entry.dev.to_string() }.build(),
                    FileInfoValue{ field_name: "Ino:".to_string() , value:self.dir_entry.ino.to_string() }.build(),
                    FileInfoValue{ field_name: "Block Size:".to_string() , value:self.dir_entry.blksize.to_string() }.build(),
                ], CSSStyle{flex_wrap: true,width:"100%",flex_grow:true,..Default::default()}).build()
            ]);

        View::new(
            children,
            CSSStyle {
                flex: 1,
                width: "100%",
                border: 1,
                boxsizing: "border-box",
                background_color: -1,
                overflow:"scroll",
                border_color: COLOR_RED,
                // taborder: 0,
                ..Default::default()
            },
        )
        .build()
    }
}