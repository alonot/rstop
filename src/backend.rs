use std::collections::HashMap;
use std::fs::{read_dir, Metadata};
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

use crate::models;
use crate::models::models::{Item, Message, MessageType, State};

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

fn read_directory(path: Arc<String>, no_threads: Arc<Mutex<i32>>) -> Result<Directory, String> {
    let binding = path.to_string();
    let dir_path = Path::new(&binding);
    let mut directory = Directory {
        name: path.clone(),
        path: path.clone(),
        total_size: 0,
        no_files: 0,
        no_folders: 0,
        dirents: vec![],
    };

    let mut threads: Vec<JoinHandle<()>> = vec![];
    let mut aggregators: HashMap<String, Arc<Mutex<AGGREGATOR>>> = HashMap::new();

    if dir_path.exists() && dir_path.is_dir() {
        for entry in read_dir(dir_path).map_err(|e| format!("READ DIR ERROR: {}", e))? {
            match entry {
                Ok(entry) => {
                    let path = match entry.path().to_str() {
                        Some(val) => val.to_owned(),
                        None => "".to_owned(),
                    };
                    if path.is_empty() {
                        continue;
                    }
                    match entry.metadata() {
                        Ok(metadata) => {
                            // to store following infos
                            let dir_entry_val =
                                dir_entry_data(&metadata, path).map_err(|e| format!("{e:?}"))?;

                            let device_number = metadata.dev();
                            let major = (device_number >> 8) & 0xff; // Extract major number
                                                                     // let minor = device_number & 0xff;
                            if major == 0 {
                                // ignoring pseudu files
                                continue;
                            }
                            let dir_entry_mutex = Arc::new(Mutex::new(dir_entry_val));
                            let dir_entry = Arc::clone(&dir_entry_mutex);
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

                                let no_threads_clone = Arc::clone(&no_threads);
                                let dir_entry_clone = Arc::clone(&dir_entry_mutex);

                                if thread_available {
                                    let thread = thread::spawn(move || {
                                        // println!("Going with {:?}", entry.path());
                                        let threads_no_clone = no_threads_clone.clone();
                                        match calculate_size(
                                            entry.path(),
                                            Some(dir_entry_clone),
                                            no_threads_clone,
                                        ) {
                                            Ok(_) => {}
                                            Err(e) => {
                                                println!("Unable to process: {:?}", e);
                                            }
                                        };
                                        {
                                            let mut no_thread = threads_no_clone.lock().unwrap();
                                            *no_thread -= 1;
                                        }
                                    });

                                    threads.push(thread);
                                } else {
                                    match calculate_size(
                                        entry.path(),
                                        Some(dir_entry_clone),
                                        no_threads_clone,
                                    ) {
                                        Ok(_) => {}
                                        Err(e) => {
                                            println!("Unable to process: {:?}", e);
                                        }
                                    };
                                }
                                directory.dirents.push(Dirent::VALUE(dir_entry));
                            } else {
                                directory.no_files += 1;
                                let file_path = entry.path();
                                let file_extension = file_path.extension();
                                let extension = file_extension
                                    .and_then(|val| val.to_str())
                                    .unwrap_or("")
                                    .to_owned(); // Convert to an owned String

                                let aggregator = aggregators.get_mut(&extension);
                                match aggregator {
                                    Some(aggregator) => {
                                        // add this dir_entry to this aggregator
                                        let mut aggreg = aggregator.lock().unwrap();
                                        aggreg.dirents.push(dir_entry.clone());
                                        aggreg.total_size += dir_entry.lock().unwrap().size;
                                    }
                                    None => {
                                        let aggregator = Arc::new(Mutex::new(AGGREGATOR {
                                            common_name: format!("*.{}", extension),
                                            total_size: 0,
                                            dirents: vec![dir_entry.clone()],
                                        }));
                                        aggregators.insert(extension, aggregator.clone());
                                        directory.dirents.push(Dirent::AGGREGATE(aggregator));
                                    }
                                }
                                // println!("{:?} {}", entry.path(), total_size_to_string(size));
                            }
                        }
                        Err(e) => println!("Unable to process: {:?} {e:?}", entry.path()),
                    }
                }
                Err(e) => {
                    println!("Unable to process: {:?}", e);
                }
            }
        }
    }
    // println!("Joining");
    for ele in threads {
        ele.join().map_err(|e| format!("{e:?}"))?;
    }
    directory.total_size = directory.dirents.iter().fold(0, |acc, v| match v {
        Dirent::AGGREGATE(aggregator) => acc + aggregator.lock().unwrap().total_size,
        Dirent::VALUE(mutex) => acc + mutex.lock().unwrap().size,
    });
    Ok(directory)
}

/**
 * Runs backend in another thread
 */
pub fn run_backend(
    content_with_lock: Arc<RwLock<HashMap<Arc<String>, State>>>,
    folder_win: Arc<String>,
    file_win: Arc<String>,
    progress_win: Arc<String>,
    rx_frontend: Receiver<Message>,
    tx_backend: Sender<Message>,
) {
    thread::spawn(move || {
        let no_threads = Arc::new(Mutex::new(0));
        let mut directory: Option<Directory> = None;

        let folder_content = State::LIST(vec![]);

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
            content.insert(folder_win.clone(), folder_content);
            content.insert(file_win.clone(), file_content);
        }

        // waits and reads the messages in the channel
        loop {
            for received in &rx_frontend {
                let no_threads_lc = Arc::clone(&no_threads);
                match received.mtype {
                    MessageType::READDIR => {
                        directory = match read_directory(Arc::new("/".to_owned()), no_threads_lc) {
                            Ok(val) => Some(val),
                            Err(e) => {
                                println!("{e}");
                                None
                            }
                        };
                        let folder_content: &mut State;
                        {
                            let mut content = content_with_lock.write().unwrap();
                            folder_content =
                                content.get_mut(&folder_win).expect("Folder not found");

                            let names = match folder_content {
                                State::LIST(states) => states,
                                State::VALUE(_) => {
                                    panic!("")
                                }
                            };
                            names.clear();
                            match directory {
                                Some(directory) => {
                                    directory.dirents.iter().for_each(|dirent| {
                                        match dirent {
                                            Dirent::AGGREGATE(_) => {}
                                            Dirent::VALUE(mutex) => {
                                                // println!("{:?}", mutex.lock().unwrap().name);
                                                names.push(State::VALUE(Item::STRING(
                                                    mutex.lock().unwrap().name.clone(),
                                                )));
                                            }
                                        }
                                    });
                                    names.push(names[0].clone());
                                }
                                None => {}
                            }
                        }
                    }
                }
            }
        }
    });
}
