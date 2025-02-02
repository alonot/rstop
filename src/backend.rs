use std::collections::HashMap;
use std::fs::{self, read_dir, Metadata};
use std::io::Error;
use std::os::unix::fs::{MetadataExt as UMetaExt, PermissionsExt};
use std::path::{Path, PathBuf};
use std::sync::mpsc::{Receiver, Sender};
use std::sync::Mutex;
use std::sync::{Arc, RwLock};
use std::task::Context;
use std::thread::JoinHandle;
use std::time::SystemTime;
use std::{io, thread};

use models::data_models::{DirEntry, Directory, Dirent, AGGREGATOR};
use ncurses::ll::clear;

use crate::models::models::{Item, Message, MessageType, State, WinType};
use crate::{models, LOG};

fn mode_to_octal(mut mode: u32) -> String {
    let mut octal = format!("");
    while mode != 0 {
        octal += &(mode % 8).to_string();
        mode = mode / 8;
    }
    octal = octal.chars().rev().collect();
    octal
}

fn calculate_size(
    dir_path: PathBuf,
    dir_entry: Option<Arc<Mutex<DirEntry>>>,
    no_threads: Arc<Mutex<i32>>,
) -> io::Result<u64> {
    // let size = read_dir(dir_path)?.try_fold(0, |acc, file| {
    //     let file = file?;
    //     let size = match file.metadata()? {
    //         data if data.is_dir() => {
    //             let threads =  no_threads.lock().unwrap();
    //             let no_threads_clone= Arc::clone(&no_threads);
    //             calculate_size(file.path(), None, no_threads_clone)?
    //         },
    //         data => data.size(),
    //     };
    //     Ok(acc + size)
    // });
    let mut threads: Vec<JoinHandle<u64>> = vec![];
    let mut size = 0;
    for file in read_dir(dir_path.clone())? {
        // println!("___{:?}", file);
        let file = file?;
        // print!("__{:?}",file.path());
        size += match file.metadata()? {
            data if data.is_dir() => {
                let device_number = data.dev();
                let major = (device_number >> 8) & 0xff; // Extract major number
                                                         // let minor = device_number & 0xff;
                if major == 0 {
                    // ignoring pseudu files
                    0
                } else {
                    let no_threads_clone = Arc::clone(&no_threads);
                    let mut thread_available = false;
                    {
                        let mut threads_no = no_threads.lock().unwrap();
                        if *threads_no < 5 {
                            *threads_no += 1;
                            thread_available = true;
                        }
                    }
                    if thread_available {
                        threads.push(thread::spawn(move || {
                            // println!("Calculating with {:?}", file.path());
                            let size = match calculate_size(file.path(), None, no_threads_clone) {
                                Ok(val) => val,
                                Err(e) => {
                                    println!("Unable to process: {:?}", e);
                                    0
                                }
                            };
                            size + data.size()
                        }));
                        0
                    } else {
                        calculate_size(file.path(), None, no_threads_clone)? + data.size()
                    }
                }
            }
            data => {
                let device_number = data.dev();
                let major = (device_number >> 8) & 0xff; // Extract major number
                                                         // let minor = device_number & 0xff;
                if major == 0 {
                    // ignoring pseudu files
                    0
                } else {
                    data.size()
                }
            }
        };
    }
    for thread in threads {
        size += match thread.join() {
            Ok(val) => val,
            Err(_) => 0,
        }
    }
    match dir_entry {
        Some(val) => {
            val.lock().unwrap().size = size;
        }
        None => {}
    };
    // println!("{:?} {:?}", dir_path, total_size_to_string(size));
    // println!("{size:?}");
    Ok(size)
}

fn dir_entry_data(metadata: &Metadata, path: String) -> Result<DirEntry, Error> {
    let dir_entry = DirEntry {
        path: path.clone(),
        name: path,
        size: metadata.size(),
        blksize: metadata.blksize(),
        percent: 0.,
        gid: metadata.gid(),
        dev: metadata.dev(),
        nlink: metadata.nlink(),
        ino: metadata.ino(),
        mode: mode_to_octal(metadata.permissions().mode()),
        file_type: metadata.file_type(),
        modified: metadata.modified()?,
        accessed: metadata.accessed()?,
        created: match metadata.created() {
            Ok(val) => val,
            Err(_) => SystemTime::now(),
        },
    };
    Ok(dir_entry)
}

fn read_directory(
    path: Arc<String>,
    no_threads: Arc<Mutex<i32>>,
    content_with_lock: Arc<RwLock<HashMap<WinType, State>>>,
    tx_backend: &Sender<Message>,
) -> Result<Directory, String> {
    let dir_path = Path::new(&*path);
    let mut directory = Directory {
        name: path.clone(),
        path: path.clone(),
        total_size: 0,
        no_files: 0,
        no_folders: 0,
        selected: None,
        dirents: vec![],
    };
    let mut threads = Vec::new();
    let mut aggregators = HashMap::new();

    {
        let mut content = content_with_lock.write().unwrap();
        if let Some(State::LIST(names)) = content.get_mut(&WinType::FOLDERWIN) {
            names.clear();
            names.push(State::LIST(vec![
                State::VALUE(Item::STRING(directory.name.to_string())),
                State::VALUE(Item::STRING("BACK".to_string())),
            ]));
        }
    }

    if dir_path.exists() && dir_path.is_dir() {
        let mut i = 0;
        for entry in read_dir(dir_path).map_err(|e| format!("READ DIR ERROR: {}", e))? {
            if let Ok(entry) = entry {
                let path = entry.path();
                if let Ok(metadata) = entry.metadata() {
                    if metadata.dev() >> 8 & 0xff == 0 {
                        continue;
                    } // Skip pseudo-files

                    let dir_entry = Arc::new(Mutex::new(
                        dir_entry_data(&metadata, path.to_string_lossy().to_string())
                            .map_err(|e| e.to_string())?,
                    ));
                    let dir_entry_clone = Arc::clone(&dir_entry);

                    if metadata.is_dir() {
                        directory.no_folders += 1;
                        let mut thread_available = false;
                        {
                            let mut no_thread = no_threads.lock().unwrap();
                            if *no_thread < 5 {
                                thread_available = true;
                                *no_thread += 1;
                            }
                        }
                        if thread_available {
                            let no_threads_clone = Arc::clone(&no_threads);
                            threads.push(thread::spawn(move || {
                                let no_threads_clone2 = no_threads_clone.clone();
                                if calculate_size(
                                    entry.path(),
                                    Some(dir_entry_clone),
                                    no_threads_clone,
                                )
                                .is_err()
                                {
                                    eprintln!("Error processing directory");
                                }
                                *no_threads_clone2.lock().unwrap() -= 1;
                            }));
                        } else {
                            calculate_size(
                                entry.path(),
                                Some(dir_entry_clone),
                                Arc::clone(&no_threads),
                            )
                            .ok();
                        }
                        directory.dirents.push(Arc::new(Dirent::VALUE(dir_entry)));
                    } else {
                        directory.no_files += 1;
                        let ext = path
                            .extension()
                            .and_then(|val| val.to_str())
                            .unwrap_or("")
                            .to_owned();
                        LOG!(ext);
                        aggregators.entry(ext.clone()).or_insert_with(|| {
                            let agg = Arc::new(Mutex::new(AGGREGATOR {
                                common_name: format!("{}", ext),
                                percent: 0.,
                                total_size: 0,
                                dirents: vec![],
                            }));
                            directory
                                .dirents
                                .push(Arc::new(Dirent::AGGREGATE(Arc::clone(&agg))));
                            agg
                        });
                        let aggregator = aggregators.get(&ext).unwrap();
                        let mut agg = aggregator.lock().unwrap();
                        agg.dirents.push(dir_entry.clone());
                        agg.total_size += dir_entry.lock().unwrap().size;
                    }
                }
            }
            if i % 5 == 0 {
                aggregate_n_send(
                    Arc::clone(&content_with_lock),
                    tx_backend,
                    &mut directory,
                    i,
                    5,
                );
            }
            i += 1;
        }
    }

    for thread in threads {
        thread.join().map_err(|e| format!("{:?}", e))?;
    }
    aggregate_n_send(content_with_lock, tx_backend, &mut directory, 0, -1);
    Ok(directory)
}

fn aggregate_n_send(
    content_with_lock: Arc<RwLock<HashMap<WinType, State>>>,
    tx_backend: &Sender<Message>,
    directory: &mut Directory,
    start: usize,
    range: i32,
) {
    // Calculate total size while updating percentages
    directory.total_size = directory
        .dirents
        .iter_mut()
        .fold(0, |acc, dirent| match &**dirent {
            Dirent::AGGREGATE(aggregator) => {
                let mut agg = aggregator.lock().unwrap();
                let total_size = agg.total_size as f64;

                for dir in &mut agg.dirents {
                    let mut dir_val = dir.lock().unwrap();
                    dir_val.percent =
                        ((dir_val.size as f64 / total_size * 10000.0).round() / 100.) as f32;
                }

                acc + agg.total_size
            }
            Dirent::VALUE(mutex) => acc + mutex.lock().unwrap().size,
        });

    let mut content = content_with_lock.write().unwrap();
    let folder_content = content
        .get_mut(&WinType::FOLDERWIN)
        .expect("Folder not found");

    let names = if let State::LIST(states) = folder_content {
        states
    } else {
        panic!("Expected State::LIST");
    };

    let urange = if range < 0 {
        directory.dirents.len() - start
    } else {
        start + range as usize
    };

    if start == 0 {
        names.clear();
        names.push(State::LIST(vec![
            State::VALUE(Item::STRING(directory.name.clone().to_string())),
            State::VALUE(Item::STRING("<<<BACK<<<".to_string())),
        ]));
    }

    let total = directory.total_size as f64;

    for dirent in directory.dirents.iter().skip(start).take(urange) {
        match &**dirent {
            Dirent::AGGREGATE(mutex) => {
                let mut val = mutex.lock().unwrap();
                val.percent = ((val.total_size as f64 / total * 10000.0).round() / 100.) as f32;
                directory.selected.get_or_insert_with(|| dirent.clone());
            }
            Dirent::VALUE(mutex) => {
                let mut val = mutex.lock().unwrap();
                names.push(State::VALUE(Item::STRING(val.name.clone())));
                val.percent = ((val.size as f64 / total * 10000.0).round() / 100.) as f32;
            }
        }
    }

    let storage_content = content
        .get_mut(&WinType::STORAGEWIN)
        .expect("Storage Window not found");

    let storage_names = if let State::LIST(states) = storage_content {
        states
    } else {
        panic!("Expected State::LIST");
    };

    if start == 0 {
        storage_names.clear();
    }

    storage_names.extend(
        directory
            .dirents
            .iter()
            .skip(start)
            .take(urange)
            .map(|dirent| State::VALUE(Item::DIRECTORY(dirent.clone()))),
    );

    let _ = tx_backend.send(Message {
        content: None,
        mtype: MessageType::READDIR,
    });
}

/**
 * Runs backend in another thread
 */
pub fn run_backend(
    content_with_lock: Arc<RwLock<HashMap<WinType, State>>>,
    rx_frontend: Receiver<Message>,
    tx_backend: Sender<Message>,
) {
    let folder_content = State::LIST(vec![]);

    let storage_content = State::LIST(vec![]);

    let file_content = State::LIST(vec![
        State::VALUE(Item::STRING(format!("File Path"))),
        State::LIST(vec![
            State::VALUE(Item::STRING(format!("File Type: "))),
            State::VALUE(Item::STRING(format!("Size: "))),
            State::VALUE(Item::STRING(format!("Permission: "))),
            State::VALUE(Item::STRING(format!("Last Accessed: "))),
            State::VALUE(Item::STRING(format!("Created: "))),
            State::VALUE(Item::STRING(format!("Last Modified: "))),
            State::VALUE(Item::STRING(format!("Number of Links: "))),
            State::VALUE(Item::STRING(format!("Dev: "))),
            State::VALUE(Item::STRING(format!("Ino: "))),
            State::VALUE(Item::STRING(format!("Block Size: "))),
        ]),
    ]);
    {
        let mut content = content_with_lock.write().unwrap();
        content.insert(WinType::FOLDERWIN, folder_content);
        content.insert(WinType::FILEWIN, file_content);
        content.insert(WinType::STORAGEWIN, storage_content);
    }

    thread::spawn(move || -> ! {
        let no_threads = Arc::new(Mutex::new(0));
        let mut directory: Option<Directory> = None;

        // waits and reads the messages in the channel
        loop {
            for received in &rx_frontend {
                let no_threads_lc = Arc::clone(&no_threads);
                let content_with_lock_clone = Arc::clone(&content_with_lock);
                match received.mtype {
                    MessageType::READDIR => {
                        let dir = match received.content {
                            Some(val) => val,
                            None => Arc::new("/".to_owned()),
                        };
                        directory = match read_directory(
                            dir,
                            no_threads_lc,
                            content_with_lock_clone,
                            &tx_backend,
                        ) {
                            Ok(val) => {
                                LOG!(format!("{}", val.name));
                                Some(val)
                            }
                            Err(e) => {
                                println!("{e}");
                                None
                            }
                        };
                    }
                    MessageType::GOBACK => {
                        let prev_dir = match directory {
                            Some(ref dir) => {
                                let name = dir.name.to_string();
                                let path = Path::new(&name);
                                match path.parent() {
                                    Some(val) => Arc::new(
                                        val.to_str().expect("can't convert to string").to_string(),
                                    ),
                                    None => Arc::new("/".to_owned()),
                                }
                            }
                            None => Arc::new("/".to_owned()),
                        };

                        directory = match read_directory(
                            prev_dir,
                            no_threads_lc,
                            content_with_lock_clone,
                            &tx_backend,
                        ) {
                            Ok(val) => {
                                LOG!(format!("{}", val.name));
                                Some(val)
                            }
                            Err(e) => {
                                println!("{e}");
                                None
                            }
                        };
                    }
                }
            }
        }
    });
}
