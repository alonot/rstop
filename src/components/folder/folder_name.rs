use std::{path::{self, Path, PathBuf}, sync::{Arc, Mutex}};

use cncurses::{components::view::View, interfaces::{Component, ComponentBuilder, Document}, styles::CSSStyle, use_state, LOGLn, DOCUMENT};
use ncurses::{COLOR_BLUE, COLOR_CYAN, COLOR_MAGENTA};

use crate::{components::path::PathComp, utils::{path_last_name, path_to_string}};


pub struct FolderName{
    pub path: Arc<PathBuf>,
    pub setpath: Arc<dyn Fn(String) + Send + Sync>,
}

impl Component for FolderName {
    fn __call__(&mut self) -> std::sync::Arc<std::sync::Mutex<dyn Component>> {
        let (bd, setbd) = use_state(COLOR_CYAN);
        let setbd2 = setbd.clone();

        let mut path_comps: Vec<Arc<Mutex<dyn Component + 'static>>> = self.path.ancestors().map(|p| {
            let pf = path_to_string(p);
            let fname = path_last_name(p);
            PathComp {
                path: pf,
                name: fname,
                setpath: self.setpath.clone() ,
            }.build()
        }).collect();

        path_comps.reverse();

        View::new(
            path_comps,
            CSSStyle{
                width: "100%",
                z_index: -1,
                border: 1,
                flex_wrap: true,
                margin: "-1 0 -1 0",
                height: "2",
                overflow: "scroll",
                taborder: 0,
                border_color: bd,
                flex_direction: "horizontal",
                ..Default::default()
        })
        .onfocus(move |_e| {
            setbd(Document::get_color(120, 120, 40));
        })
        .onunfocus(move |_e| {
            setbd2(COLOR_CYAN);
        })
        .build()
    }
}
