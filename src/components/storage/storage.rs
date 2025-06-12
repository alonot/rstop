use std::sync::{Arc, Mutex};

use cncurses::{
    components::{text::Text, view::View},
    interfaces::{Component, ComponentBuilder, Document, EVENT},
    styles::CSSStyle,
    use_state, LOGLn,
};
use ncurses::{COLOR_BLACK, COLOR_MAGENTA, COLOR_RED, KEY_BTAB};

use crate::{
    components::storage::{
        aggregator::AggregatorRow, direntryrow::DirEntryRow, headerrow::HeaderRow,
    },
    models::data_models::{DIRENTHolder, DIRENTRYHolder, Dirent, DIRENTRY}, utils::{percent, total_size_to_string},
};

pub struct StorageWin {
    pub entries: Arc<Mutex<Vec<Dirent>>>,
    pub set_file_info_direntry: Arc<dyn Fn(Arc<Mutex<DIRENTRY>>) + Send + Sync>,
    pub loading: bool,
    pub total_size: u64,
    pub sort_by_size: Arc<dyn Fn() + Send + Sync>,
    pub sort_by_name: Arc<dyn Fn() + Send + Sync>,
}

impl Component for StorageWin {
    fn __call__(&mut self) -> std::sync::Arc<std::sync::Mutex<dyn Component>> {


        let mut children = vec![HeaderRow {
            name: "Name".to_string(),
            size: "Size".to_string(),
            percent: "Percent".to_string(),
            btn1_name: "Sort By Name".to_string(),
            onclick1: Some(self.sort_by_name.clone()),
            btn2_name: "Sort By Size".to_string(),
            onclick2: Some(self.sort_by_size.clone()),
        }
        .build()];

        if self.loading {
            children.push(
                Text::new(
                    "Loading".to_string(),
                    CSSStyle {
                        width: "100%",
                        height: "100%",
                        overflow: "scroll",
                        padding: "50% 0 50% 0",
                        top: "1",
                        position: "relative",
                        // boxsizing: "border-box",
                        background_color: -1,
                        z_index: -2,
                        ..Default::default()
                    },
                )
                .build(),
            )
        } else {
            children.extend(
                self.entries.lock().unwrap()
                    .iter()
                    .map(|d_lk| {
                        match d_lk.clone() {
                            Dirent::AGGREGATE(agg_lk) => AggregatorRow {
                                agg_lk: agg_lk.clone(),
                                set_file_info_direntry: self.set_file_info_direntry.clone(),
                            }
                            .build(),
                            Dirent::VALUE(dir_lk) => {
                                let d_c = dir_lk.clone();
                                let dir_entry = dir_lk.lock().unwrap();
                                let set_file = self.set_file_info_direntry.clone();
                                DirEntryRow {
                                    name: dir_entry.name.to_string(),
                                    size: total_size_to_string(dir_entry.size),
                                    percent: percent(dir_entry.size , self.total_size),
                                    set_file_info_direntry: Arc::new(move || {
                                        set_file(d_c.clone());
                                    }),
                                }
                                .build()
                            }
                        }
                    })
                    .collect::<Vec<Arc<Mutex<dyn Component>>>>(),
            );
        }
        children.push(
            Text::new(
                "Storage".to_string(),
                CSSStyle {
                    color: Document::get_color(255, 110, 0),
                    left: "2",
                    top: "-1",
                    position: "relative",
                    ..Default::default()
                },
            )
            .build(),
        );

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
                // taborder: 0,
                ..Default::default()
            },
        )
        .build()
    }
}
