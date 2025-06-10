use std::sync::{Arc, Mutex};

use cncurses::{
    components::{button::Button, view::View},
    interfaces::{Component, ComponentBuilder},
    styles::CSSStyle,
    use_state, LOGLn,
};
use ncurses::{COLOR_BLACK, COLOR_CYAN};

use crate::{
    components::storage::{direntryrow::DirEntryRow, headerrow::HeaderRow},
    models::data_models::{DIRENTRYHolder, AGGREGATOR, DIRENTRY},
    utils::total_size_to_string,
};

pub struct AggregatorRow {
    pub agg_lk: Arc<Mutex<AGGREGATOR>>,
    pub set_file_info_direntry: Arc<dyn Fn(String) + Send + Sync>
}

impl Component for AggregatorRow {
    fn __call__(&mut self) -> Arc<Mutex<dyn Component>> {
        let (opened, setopened) = use_state(false);
        let agg = self.agg_lk.lock().unwrap();

        let setopened_c = setopened.clone();

        let sort_by_name = || {};
        let sort_by_size = || {};

        let children = agg
            .dirents
            .iter()
            .map(|d_lk| {
                let d = d_lk.lock().unwrap();
                DirEntryRow {
                    name: d.name.to_string(),
                    size: d.size.to_string(),
                    percent: d.percent,
                    set_file_info_direntry: self.set_file_info_direntry.clone()
                }
                .build()
            })
            .collect::<Vec<Arc<Mutex<dyn Component>>>>();

        if opened {
            View::new_key(
                Some(agg.common_name.to_string()),
                vec![
                    Button::new(
                        HeaderRow {
                            name: format!("v {}", agg.common_name),
                            size: total_size_to_string(agg.total_size),
                            percent: format!("{}%", agg.percent),
                            btn1_name: "Sort By Name".to_string(),
                            onclick1: Some(Arc::new(sort_by_name)),
                            btn2_name: "Sort By Size".to_string(),
                            onclick2: Some(Arc::new(sort_by_size)),
                        }
                        .build()
                        , CSSStyle{
                            height: "1",
                            width: "100%",
                            ..Default::default()
                        }
                        , move |_e| {
                            setopened_c(false);
                        }
                    ).build(),
                    View::new(
                        children,
                        CSSStyle {
                            width: "100%",
                            border: 1,
                            left: "-1",
                            border_color: COLOR_CYAN,
                            ..Default::default()
                        },
                    )
                    .build(),
                ],
                CSSStyle {
                    width: "100%",
                    border_color: COLOR_BLACK,
                    ..Default::default()
                },
            )
            .build()
        } else {
            Button::new(
                DirEntryRow {
                    name: format!("> {}", agg.common_name),
                    size: total_size_to_string(agg.total_size),
                    percent: agg.percent,
                    set_file_info_direntry: self.set_file_info_direntry.clone()
                }
                .build(),
                CSSStyle {
                    height: "1",
                    width: "100%",
                    ..Default::default()
                },
                move |_e| {
                    setopened(true);
                },
            )
            .build()
        }
    }
}
