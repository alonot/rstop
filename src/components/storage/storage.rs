use std::sync::{Arc, Mutex};

use cncurses::{
    components::{text::Text, view::View},
    interfaces::{Component, ComponentBuilder, Document, EVENT},
    styles::CSSStyle,
    use_state, LOGLn,
};
use ncurses::{COLOR_BLACK, COLOR_MAGENTA, COLOR_RED, KEY_BTAB};

use crate::{components::storage::{aggregator::AggregatorRow, direntryrow::DirEntryRow, headerrow::HeaderRow}, models::data_models::{DIRENTRYHolder, Dirent, DIRENTRY}};

pub struct StorageWin{
    pub entries: Vec<Arc<Mutex<Dirent>>>,
    pub set_file_info_direntry: Arc<dyn Fn(String) + Send + Sync>
}

impl Component for StorageWin {
    fn __call__(&mut self) -> std::sync::Arc<std::sync::Mutex<dyn Component>> {

        let sort_by_name = || {};
        let sort_by_size = || {};

        let mut children = vec![
            HeaderRow{
                name: "Name".to_string(),
                size: "Size".to_string(),
                percent: "Percent".to_string(),
                btn1_name: "Sort By Name".to_string(),
                onclick1: Some(Arc::new(sort_by_name)),
                btn2_name: "Sort By Size".to_string(),
                onclick2: Some(Arc::new(sort_by_size)),
            }.build()
        ];

        children.extend(self.entries.iter().map(|d_lk| {
            let d = d_lk.lock().unwrap();
            match d.clone() {
                Dirent::AGGREGATE(agg_lk) => {
                    AggregatorRow{
                        agg_lk: agg_lk.clone(),
                        set_file_info_direntry: self.set_file_info_direntry.clone()
                    }.build()
                },
                Dirent::VALUE(dir_lk) => {
                    let dir_entry = dir_lk.lock().unwrap();
                    DirEntryRow {
                        name: dir_entry.name.to_string(),
                        size: dir_entry.size.to_string(),
                        percent: dir_entry.percent,
                        set_file_info_direntry: self.set_file_info_direntry.clone()
                    }.build()
                },
            }
        }).collect::<Vec<Arc<Mutex<dyn Component>>>>());

        children.push(Text::new(
                "Storage".to_string(),
                CSSStyle {
                    color: Document::get_color(255, 110, 0),
                    left: "2",
                    top: "-1",
                    position: "relative",
                    ..Default::default()
                },
            )
            .build());

        View::new(
            children,
            CSSStyle {
                flex: 3,
                width: "100%",
                overflow: "scroll",
                border: 1,
                boxsizing: "border-box",
                background_color: -1,
                border_color: COLOR_RED,
                color: COLOR_MAGENTA,
                // taborder: 0,
                ..Default::default()
            },
        )
        .build()
    }
}
