use std::{path::PathBuf, sync::{mpsc::{channel, Sender}, Arc, Mutex}};

use cncurses::{components::view::View, interfaces::{Component, ComponentBuilder}, run, styles::CSSStyle, use_state, LOGLn};

use crate::components::{fileinfo::FileInfoWin, folder::FolderWin, storage::StorageWin};


mod components;

pub struct Message {

}

struct App;

impl Component for App {
    fn __call__(&mut self) -> std::sync::Arc<std::sync::Mutex<dyn Component>> {
        let (path,setpath) = use_state(PathBuf::from("/home/alonot"));


        // let (tx_frontend, rx_frontend) = channel::<Message>();
        // let (tx_backend, rx_backend) = channel::<Message>();

        // let tx_frontend_arc: Arc<Sender<Message>> = Arc::new(tx_frontend);


        // let tx_frontend_c = tx_frontend_arc.clone();
        let spath = setpath.clone();

        let change_path = move |s: String| {
            LOGLn!("GOME {}", s);
            spath(PathBuf::from(s));
            // tx_frontend_c.send(Message {  });
        };

        View::new(vec![
            FolderWin{
                path: Box::new(path),
                setpath: Arc::new(Mutex::new(change_path))
            }.build(),
            View::new(
                vec![
                    FileInfoWin{}.build(),
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
            scroll: "scroll",
            ..Default::default()
        }).build()
    }
}

fn main() {
    println!();
    run(App);
}