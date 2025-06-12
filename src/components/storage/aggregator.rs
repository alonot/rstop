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
    models::data_models::{DIRENTRYHolder, Dirent, AGGHOLDER, AGGREGATOR, DIRENTRY},
    utils::{percent, total_size_to_string},
};

pub struct AggregatorRow {
    pub agg_lk: Arc<Mutex<AGGREGATOR>>,
    pub set_file_info_direntry: Arc<dyn Fn(Arc<Mutex<DIRENTRY>>) + Send + Sync>,
}

impl Component for AggregatorRow {
    fn __call__(&mut self) -> Arc<Mutex<dyn Component>> {
        let (opened, setopened) = use_state(false);
        let (agg_state_lk, set_agg) = use_state(Arc::new(AGGHOLDER(self.agg_lk.clone())));

        let setopened_c = setopened.clone();
        
        let agg_lk = agg_state_lk.clone();
        let set_agg_c = set_agg.clone();

        let sort_by_name = move || {
            let mut agg = agg_lk.0.lock().unwrap();
            let reverse = agg.sorted_by_name;

            {
                let dir_entries = &mut agg.dirents;

                dir_entries.sort_by_key(|v| v.lock().unwrap().name.clone());

                if reverse {
                    dir_entries.reverse();
                }
            }


            agg.sorted_by_name = !reverse;
            set_agg_c(agg_lk.clone());
        };

        let agg_lk = agg_state_lk.clone();
        let set_agg_c = set_agg.clone();

        let sort_by_size = move || {
            let mut agg = agg_lk.0.lock().unwrap();
            let reverse = agg.sorted_by_size;

            {
                let dir_entries = &mut agg.dirents;

                dir_entries.sort_by_key(|v| {
                    let val = v.lock().unwrap().size as i128;
                    if reverse {
                        -val
                    } else {
                        val
                    }
                });
            }

            agg.sorted_by_size = !reverse;
            set_agg_c(agg_lk.clone());
        };

        let agg = agg_state_lk.0.lock().unwrap();

        let children = agg
            .dirents
            .iter()
            .map(|d_lk| {
                let d_c = d_lk.clone();
                let set_file = self.set_file_info_direntry.clone();
                let d = d_lk.lock().unwrap();
                DirEntryRow {
                    name: d.name.to_string(),
                    size: total_size_to_string(d.size),
                    percent: percent(d.size, agg.total_size),
                    set_file_info_direntry: Arc::new(move || set_file(d_c.clone())),
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
                        .build(),
                        CSSStyle {
                            height: "1",
                            width: "100%",
                            ..Default::default()
                        },
                        move |_e| {
                            setopened_c(false);
                        },
                    )
                    .build(),
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
                    set_file_info_direntry: Arc::new(move || {}),
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
