use std::{
    collections::HashMap,
    path::PathBuf,
    sync::{mpsc::Sender, Arc, Condvar, Mutex},
    time::SystemTime,
};

use cncurses::{
    components::view::View,
    interfaces::{Component, ComponentBuilder},
    styles::CSSStyle,
    use_state, LOGLn,
};

use crate::{
    components::{
        fileinfo::fileinfo::FileInfoWin, folder::folder::FolderWin, storage::storage::StorageWin,
    },
    models::data_models::{
        DIRENTHolder, DIRENTRYHolder, DirectoryOut, Dirent, Message, AGGREGATOR, DIRENTRY,
    },
    LOGLn2, APP_HANDLER,
};

pub struct App {
    pub tx_frontend: Arc<Sender<Message>>,
    pub pair: Arc<(Mutex<bool>, Condvar)>,
    pub dir: Arc<PathBuf>,
    pub init_directory: Arc<DirectoryOut>,
}

impl Component for App {
    fn __call__(&mut self) -> std::sync::Arc<std::sync::Mutex<dyn Component>> {
        let (path, setpath) = use_state(self.dir.clone());
        let (directroy, set_directory) = use_state(self.init_directory.clone());
        let (dir_entry_curr, set_dir_entry) = use_state::<Option<DIRENTRYHolder>>(None);
        let (loading, setloading) = use_state(true);

        let tx_frontend_c1 = self.tx_frontend.clone();
        let set_dir = set_directory.clone();
        let setloading2 = setloading.clone();
        let set_path_c = setpath.clone();
        let handle_message: Arc<dyn Fn(Message) + Send + Sync> =
        Arc::new(move |m: Message| match m {
            Message::UPDATEDIRINFO(dir) => {
                set_path_c(dir.dir_info.path.clone());
                set_dir(dir);
                setloading2(false);
                let _ = tx_frontend_c1.send(Message::UPDATECOMPLETE);
            }
            _ => {}
        });
        
        *APP_HANDLER.lock().unwrap() = Some(handle_message);
        let (lock, cvar) = &*self.pair;
        let mut started = lock.lock().unwrap();
        *started = true;
        cvar.notify_one();
        let directory_c = directroy.clone();
        let set_dir = set_directory.clone();
        
        let sort_by_name = move || {
            let reverse =  *directory_c.sorted_by_name .lock().unwrap() ;

            {

                let dir_entries = directory_c.dirents.clone();
                
                dir_entries.lock().unwrap().sort_by_key(|v| {
                    match v {
                        Dirent::AGGREGATE(agg_lk) => {
                            agg_lk.lock().unwrap().common_name.clone()
                        },
                        Dirent::VALUE(dir_lk) => {
                            dir_lk.lock().unwrap().name.clone()
                        },
                    }
                });

                if reverse {
                    dir_entries.lock().unwrap().reverse();
                }
                
            }

            *directory_c.sorted_by_name.lock().unwrap() = !reverse;
            set_dir(directory_c.clone());
        };
        
        let set_dir = set_directory.clone();
        let directory_c = directroy.clone();
        let sort_by_size = move || {
            let reverse =  *directory_c.sorted_by_size .lock().unwrap() ;

            let dir_entries = directory_c.dirents.clone();

            dir_entries.lock().unwrap().sort_by_key(|v| {
                let val = match v {
                    Dirent::AGGREGATE(agg_lk) => {
                        agg_lk.lock().unwrap().total_size as i128
                    },
                    Dirent::VALUE(dir_lk) => {
                        dir_lk.lock().unwrap().size as i128
                    },
                };
                if reverse {
                    -val
                } else {
                    val
                }
            });

            *directory_c.sorted_by_size .lock().unwrap() = !reverse;
            set_dir(directory_c.clone());
        };

        let set_dir_entry1 = set_dir_entry.clone();
        let change_fileinfo_dir_entry = move |s| {
            set_dir_entry1(Some(DIRENTRYHolder(s)));
        };

        let spath = setpath.clone();
        let setloading1 = setloading.clone();

        let set_dir_entry2 = set_dir_entry.clone();
        let tx_frontend_c = self.tx_frontend.clone();
        let change_path = move |s: String| {
            let p = Arc::new(PathBuf::from(s));
            spath(p.clone());
            let _ = tx_frontend_c.send(Message::READDIR(p.clone()));
            set_dir_entry2(None);
            setloading1(true);
        };

        let path_c = path.clone();
        let tx_frontend_c = self.tx_frontend.clone();
        let select_directory = move |s: String| {
            let p = Arc::new(path_c.join(s));
            setpath(p.clone());
            let _ = tx_frontend_c.send(Message::READDIR(p.clone()));
            set_dir_entry(None);
            setloading(true);
        };


        let (folder, entries) = {
            let dir = directroy.dirents.clone();
            (dir.clone().lock().unwrap().iter()
                .filter_map(|f| match f.clone() {
                    Dirent::AGGREGATE(_) => None,
                    Dirent::VALUE(val) => Some(val.lock().unwrap().name.clone()),
                })
                .collect::<Vec<Arc<String>>>(),dir.clone())
        };

        View::new(
            vec![
                FolderWin {
                    path: path.clone(),
                    setpath: Arc::new(change_path),
                    select_folder: Arc::new(select_directory),
                    folders: folder,
                    dir_info: directroy.dir_info.clone(),
                    loading: loading,
                }
                .build(),
                View::new(
                    vec![
                        FileInfoWin {
                            dir_entry: dir_entry_curr.clone(),
                        }
                        .build(),
                        StorageWin {
                            loading: loading,
                            entries: entries,
                            sort_by_name: Arc::new(sort_by_name),
                            sort_by_size: Arc::new(sort_by_size),
                            total_size: directroy.dir_info.total_size,
                            set_file_info_direntry: Arc::new(change_fileinfo_dir_entry),
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
