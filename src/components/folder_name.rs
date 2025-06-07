use std::{path::{self, Path, PathBuf}, sync::{Arc, Mutex}};

use cncurses::{components::view::View, interfaces::{Component, ComponentBuilder}, styles::CSSStyle};

use crate::components::path::PathComp;


pub struct FolderName{
    pub path: Box<PathBuf>,
    pub setpath: Arc<Mutex<dyn FnMut(String)>>,
}

impl Component for FolderName {
    fn __call__(&mut self) -> std::sync::Arc<std::sync::Mutex<dyn Component>> {
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
                flex_direction: "horizontal",
                ..Default::default()
        }).build()
    }
}


unsafe impl Send for FolderName {}