use std::{path::{self, Path, PathBuf}, sync::{Arc, Mutex}};

use cncurses::{components::view::View, interfaces::{Component, ComponentBuilder}, styles::CSSStyle, use_state, LOGLn};
use ncurses::{COLOR_BLUE, COLOR_MAGENTA};

use crate::components::path::PathComp;


pub struct FolderName{
    pub path: Box<PathBuf>,
    pub setpath: Arc<Mutex<dyn FnMut(String)>>,
}

impl Component for FolderName {
    fn __call__(&mut self) -> std::sync::Arc<std::sync::Mutex<dyn Component>> {
        let (bd, setbd) = use_state(COLOR_MAGENTA);
        let setbd2 = setbd.clone();


        let mut path_comps: Vec<Arc<Mutex<dyn Component + 'static>>> = self.path.ancestors().map(|p| {
            let pf = p.as_os_str().to_str().map_or("/", |f| f).to_string();
            let fname = p.file_name().map_or("", |f| f.to_str().map_or("", |f| f)).to_string() + "/";
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
                taborder: 1,
                border_color: bd,
                flex_direction: "horizontal",
                ..Default::default()
        })
        .onfocus(move || {
            setbd(COLOR_BLUE);
        })
        .onunfocus(move || {
            LOGLn!("UNFOCUS");
            setbd2(COLOR_MAGENTA);
        })
        .build()
    }
}


unsafe impl Send for FolderName {}