use std::{
    collections::HashMap, env, path::PathBuf, sync::{
        mpsc::{channel, Receiver, Sender},
        Arc, Condvar, Mutex,
    }, thread, time::{Duration, SystemTime}
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
    app::App, backend::backend::run_backend, components::{
        fileinfo::fileinfo::FileInfoWin, folder::folder::FolderWin, storage::storage::StorageWin,
    }, models::data_models::{DIRENTRYHolder, DirInfo, DirectoryOut, Dirent, Message, AGGREGATOR, DIRENTRY}, utils::{path_last_name, path_to_string, total_size_to_string}
};

mod backend;
mod components;
mod models;
mod utils;
mod app;

pub static APP_HANDLER: Mutex<Option<Arc<dyn Fn(Message) + Send + Sync>>> = Mutex::new(None);

fn main() -> Result<(), String> {

    let args: Vec<String> = env::args().collect();
    let dir: Arc<PathBuf>;

    if args.len() >= 2 {
        let current_proposed = args[1].clone();
        let path = PathBuf::from(current_proposed.to_string());
        if path.exists() {
            dir = Arc::new(path.canonicalize().expect("Unable to canocalize"));
        } else {
            return Err("Give Path Does not Exists".to_string())
        }
    } else {
        dir = Arc::new(PathBuf::from("/"));
    }

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
                    while !*started {
                        started = cvar.wait(started).unwrap();
                    }
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
    let _ = std::fs::write("debug2.txt", "");
    
    thread::sleep(Duration::from_millis(200));
    // LOGLn!("DONE");
    run_backend(rx_frontend, tx_backend);

    // let _ = tx_frontend.send(Message::OPENCONN);

    let _ = tx_frontend.send(Message::READDIR(dir.clone()));

    
    run(App {
        tx_frontend,
        pair: pair2,
        dir: dir.clone(),   
        init_directory:Arc::new(DirectoryOut::new(&dir)),
    });

    Ok(())
}
