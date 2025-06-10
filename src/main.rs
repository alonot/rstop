use std::{
    path::PathBuf,
    sync::{
        mpsc::{channel, Receiver, Sender},
        Arc, Condvar, Mutex,
    },
    thread,
    time::{Duration, SystemTime},
};

use cncurses::{
    components::view::View,
    interfaces::{Component, ComponentBuilder},
    run,
    styles::CSSStyle,
    use_state, LOGLn,
};
use ncurses::COLOR_BLACK;

use crate::{
    backend::backend::run_backend,
    components::{
        fileinfo::fileinfo::FileInfoWin, folder::folder::FolderWin, storage::storage::StorageWin,
    },
    models::data_models::{DIRENTRYHolder, Dirent, Message, AGGREGATOR, DIRENTRY},
    utils::total_size_to_string,
};

mod backend;
mod components;
mod models;
mod utils;

struct App {
    tx_frontend: Arc<Sender<Message>>,
    pair: Arc<(Mutex<bool>, Condvar)>,
}

impl Component for App {
    fn __call__(&mut self) -> std::sync::Arc<std::sync::Mutex<dyn Component>> {
        let (path,setpath) = use_state(PathBuf::from("/home/alonot/asdf/asdfasdc/asdfa/asdfasd/asdfsd/dfsdf/asdf/asdf//asd/as/das/ddas/das/da/sdf/asd/f/asd/d/a/d/ds/sd/sd/sfd/dg/g/gtr/h/fg/f/d/"));
        let (size, setsize) = use_state(10000);

        let c = setsize.clone();

        let handle_message: Arc<dyn Fn(Message) + Send + Sync> = Arc::new(move |m: Message| {
            match m {
                Message::OPENCONN => {
                    c(1000000);
                }
                Message::CLOSECONN => {}
                Message::READDIR(_) => {}
                Message::RELOADDIR(_) => {}
                Message::UPDATECOMPLETE => {}
                Message::UPDATEDIRINFO(directory) => {}
                Message::SENDREPONSE(direntryout) => {}
            }
        });

        *APP_HANDLER.lock().unwrap() = Some(handle_message);
        let (lock, cvar) = &*self.pair;
        let mut started = lock.lock().unwrap();
        *started = true;
        LOGLn2!("Done bros");
        self.pair.1.notify_one();

        let dir_entry = DIRENTRY {
            percent: 10.40,
            name: "Ram".to_string(),
            path: "./home/alonot/Ram".to_string(),
            size: size,
            blksize: size,
            gid: 12,
            dev: 1,
            nlink: 4,
            ino: 1912843,
            file_type: "DIR".to_string(),
            modified: SystemTime::now(),
            created: SystemTime::now(),
            accessed: SystemTime::now(),
            mode: "1231234".to_string(),
        };

        let dir_entry_holder = DIRENTRYHolder(Arc::new(Mutex::new(dir_entry.clone())));

        let (dir_entry_curr, set_dir_entry) = use_state(dir_entry_holder);

        let p = dir_entry.clone();

        let change_dir_entry = move |s| {
            LOGLn!("Changed to {}", s);
            let mut p1 = p.clone();
            p1.name = s;
            set_dir_entry(DIRENTRYHolder(Arc::new(Mutex::new(p1))));
        };

        let entries = (0..20)
            .map(|i| {
                if i % 10 == 0 {
                    let mut p = dir_entry.clone();
                    p.name = "RAMA".to_string();
                    Arc::new(Mutex::new(Dirent::AGGREGATE(Arc::new(Mutex::new(
                        AGGREGATOR {
                            common_name: Arc::new("*.c".to_string()),
                            sorted_by_name: false,
                            sorted_by_size: false,
                            total_size: 1000000,
                            percent: 40.,
                            expanded: false,
                            dirents: vec![
                                Arc::new(Mutex::new(p.clone())),
                                Arc::new(Mutex::new(p.clone())),
                                Arc::new(Mutex::new(p.clone())),
                                Arc::new(Mutex::new(p.clone())),
                                Arc::new(Mutex::new(p.clone())),
                                Arc::new(Mutex::new(p.clone())),
                                Arc::new(Mutex::new(p.clone())),
                                Arc::new(Mutex::new(p.clone())),
                                Arc::new(Mutex::new(p.clone())),
                                Arc::new(Mutex::new(p.clone())),
                            ],
                        },
                    )))))
                } else {
                    Arc::new(Mutex::new(Dirent::VALUE(Arc::new(Mutex::new(
                        dir_entry.clone(),
                    )))))
                }
            })
            .collect::<Vec<Arc<Mutex<Dirent>>>>();

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

        View::new(
            vec![
                FolderWin {
                    path: Box::new(path),
                    setpath: Arc::new(change_path),
                    select_folder: Arc::new(select_path),
                    folders: vec![
                        Arc::new("Box".to_string()),
                        Arc::new("Box".to_string()),
                        Arc::new("Box".to_string()),
                    ],
                    size: size,
                }
                .build(),
                View::new(
                    vec![
                        FileInfoWin {
                            dir_entry: dir_entry_curr.clone(),
                        }
                        .build(),
                        StorageWin {
                            entries: entries,
                            set_file_info_direntry: Arc::new(change_dir_entry),
                        }
                        .build(),
                    ],
                    CSSStyle {
                        height: "100%",
                        flex: 2,
                        ..Default::default()
                    },
                )
                .build(),
            ],
            CSSStyle {
                flex_direction: "horizontal",
                height: "100%",
                width: "100%",
                boxsizing: "border-box",
                overflow: "scroll",
                ..Default::default()
            },
        )
        .build()
    }
}

static APP_HANDLER: Mutex<Option<Arc<dyn Fn(Message) + Send + Sync>>> = Mutex::new(None);

fn main() {
    let (tx_frontend, rx_frontend) = {
        let (tx, rx) = channel::<Message>();
        (Arc::new(tx), rx)
    };
    let (tx_backend, rx_backend) = { channel::<Message>() };
    
    let pair = Arc::new((Mutex::new(false), Condvar::new()));
    let pair2: Arc<(Mutex<bool>, Condvar)> = Arc::clone(&pair);
    
    thread::spawn(move || -> ! {
        loop {
            let f_opt = APP_HANDLER.lock().unwrap();
            let func = match f_opt.clone() {
                Some(f) => f,
                None => {
                    let (lock, cvar) = &*pair;
                    let mut started = lock.lock().unwrap();
                    // wait until handler is mounted
                    drop(f_opt);
                    LOGLn!("Here2");
                    while !*started {
                        started = cvar.wait(started).unwrap();
                    }
                    LOGLn!("Here3");
                    let f_opt = APP_HANDLER.lock().unwrap();
                    let Some(f) = f_opt.clone() else {
                        panic!("No Handler")
                    };
                    f
                }
            };
            for message in &rx_backend {
                func(message);
            }
        }
    });
    let _ = std::fs::write("debug.txt", "");
    let _ = std::fs::write("debug2.txt", "");
    
    thread::sleep(Duration::from_millis(200));
    LOGLn!("DONE");
    run_backend(rx_frontend, tx_backend);

    tx_frontend.send(Message::OPENCONN);
    
    run(App {
        tx_frontend,
        pair: pair2,
    });
}
