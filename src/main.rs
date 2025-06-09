use std::{path::PathBuf, sync::{mpsc::{channel, Sender}, Arc, Mutex}, time::SystemTime};

use cncurses::{components::view::View, interfaces::{Component, ComponentBuilder}, run, styles::CSSStyle, use_state, LOGLn};
use ncurses::COLOR_BLACK;

use crate::{components::{fileinfo::fileinfo::FileInfoWin, folder::folder::FolderWin, storage::storage::StorageWin}, models::data_models::DIRENTRY, utils::total_size_to_string};

mod models;
mod components;
mod utils;

pub struct Message {

}

struct App;

impl Component for App {
    fn __call__(&mut self) -> std::sync::Arc<std::sync::Mutex<dyn Component>> {
        let (path,setpath) = use_state(PathBuf::from("/home/alonot/asdf/asdfasdc/asdfa/asdfasd/asdfsd/dfsdf/asdf/asdf//asd/as/das/ddas/das/da/sdf/asd/f/asd/d/a/d/ds/sd/sd/sfd/dg/g/gtr/h/fg/f/d/"));
        LOGLn!("GOTONE");
        let (size, setsize) = use_state(10000);

        // let (tx_frontend, rx_frontend) = channel::<Message>();
        // let (tx_backend, rx_backend) = channel::<Message>();

        // let tx_frontend_arc: Arc<Sender<Message>> = Arc::new(tx_frontend);

        let dir_entry = Arc::new(DIRENTRY {
            percent: 10.40,
            name: "Ram".to_string(),
            path: "./home/alonot/Ram".to_string(),
            size: total_size_to_string(size),
            blksize: total_size_to_string(size),
            gid: 12,
            dev: 1,
            nlink: 4,
            ino: 1912843,
            file_type: "DIR".to_string(),
            modified: SystemTime::now(),
            created: SystemTime::now(),
            accessed: SystemTime::now(),
            mode: "1231234".to_string(),
        });


        // let tx_frontend_c = tx_frontend_arc.clone();
        let spath = setpath.clone();

        let change_path = move |s: String| {
            LOGLn!("GOME {}", s);
            spath(PathBuf::from(s));
            // tx_frontend_c.send(Message {  });
        };

        let path_c = path.clone();
        let select_path = move |s: String| {
            setpath(path_c.join(s));
        };

        View::new(vec![
            FolderWin{
                path: Box::new(path),
                setpath: Arc::new(Mutex::new(change_path)),
                select_folder: Arc::new(Mutex::new(select_path)),
                folders: vec![Arc::new("Box".to_string()), Arc::new("Box".to_string()), Arc::new("Box".to_string())],
                size   : size,
            }.build(),
            View::new(
                vec![
                    FileInfoWin{
                        dir_entry: dir_entry.clone()
                    }.build(),
                    StorageWin{}.build()
                ],
                CSSStyle {
                    height: "100%",
                    flex: 2,
                    ..Default::default()
                } 
            ).build()
        ], CSSStyle{
            flex_direction: "horizontal",
            height: "100%",
            width: "100%",
            boxsizing:"border-box",
            overflow: "scroll",
            ..Default::default()
        }).build()
    }
}

fn main() {
    println!();
    run(App);
}