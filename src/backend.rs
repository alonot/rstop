use std::collections::hash_map::Entry;
use std::collections::{vec_deque, HashMap, VecDeque};
use std::fs::{ read_dir, FileType, Metadata};
use std::io::Error;
use std::fs;
use std::os::unix::fs::{FileTypeExt, MetadataExt as UMetaExt, PermissionsExt};
use std::path::{Path, PathBuf};
use std::sync::mpsc::{Receiver, Sender};
use std::sync::Mutex;
use std::sync::{Arc, RwLock};
use std::thread::JoinHandle;
use std::time::SystemTime;
use std::{io, thread, vec};

use chrono::{DateTime, Utc};
use models::data_models::{DirEntry, Directory, Dirent, AGGREGATOR};

use crate::models::models::{Item, Message, MessageType, SortButton, State, WinType};
use crate::{models, total_size_to_string, LOG};

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

fn dir_entry_data(metadata: &Metadata, path: &PathBuf) -> Result<DirEntry, Error> {
    let dir_entry = DirEntry {
        path: path.to_string_lossy().to_string(),
        name: match path.file_name() {
            Some(val) => val.to_string_lossy().to_string(),
            None => format!(""),
        },
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
        sorted_by_name: false,
        sorted_by_size: false,
    };
    let mut threads = Vec::new();
    let mut aggregators = HashMap::new();

    {
        let mut content = content_with_lock.write().unwrap();
        if let Some(State::LIST(names)) = content.get_mut(&WinType::FOLDERWIN) {
            names.clear();
            names.push(State::LIST(vec![
                State::VALUE(Item::STRING(directory.name.to_string())),
                State::VALUE(Item::STRING("<<<BACK<<<".to_string())),
            ]));
        }
        let _ = tx_backend.send(Message {
            content: None,
            mtype: MessageType::READDIR,
        });
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
                        dir_entry_data(&metadata, &path).map_err(|e| e.to_string())?,
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
                        // LOG!(format!("aggate: {}", ext));
                        aggregators.entry(ext.clone()).or_insert_with(|| {
                            let agg = Arc::new(Mutex::new(AGGREGATOR {
                                common_name: ext.clone(),
                                percent: 0.,
                                total_size: 0,
                                dirents: vec![],
                                sorted_by_name: false,
                                sorted_by_size: false,
                                expanded: false,
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

fn aggregate(
    content_with_lock: Arc<RwLock<HashMap<WinType, State>>>,
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
    {
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
                    directory
                        .selected
                        .get_or_insert_with(|| val.dirents[0].clone());
                }
                Dirent::VALUE(mutex) => {
                    let mut val = mutex.lock().unwrap();
                    names.push(State::VALUE(Item::STRING(val.name.clone())));
                    val.percent = ((val.size as f64 / total * 10000.0).round() / 100.) as f32;
                    directory.selected.get_or_insert_with(|| mutex.clone());
                }
            }
        }

        let storage_content = content
            .get_mut(&WinType::STORAGEWIN)
            .expect("Storage Window not found");

        let storage_names = if let State::LIST(states) = storage_content {
            match &mut states[1] {
                State::LIST(states) => states,
                State::VALUE(_) => panic!("Expected State::LIST"),
            }
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
    }
    change_selected(content_with_lock.clone(), directory);
}

fn get_file_type_as_string(file_type: &FileType) -> String{
    if (file_type.is_file()) {
        format!("File")
    } else if file_type.is_dir() {
        format!("Dir")
    } else if file_type.is_symlink() {
        format!("Symlink")
    } else if file_type.is_block_device() {
        format!("Block Device")
    } else if file_type.is_char_device() {
        format!("Char Device")
    } else if file_type.is_fifo() {
        format!("Fifo")
    } else if file_type.is_socket() {
        format!("Socket")
    } else {
        format!("None")
    }
}

fn system_time_to_string(system_time: &SystemTime) -> String {
    let datetime: DateTime<Utc> = (*system_time).into();
    let formatted_time = datetime.format("%d/%m/%Y %H:%M").to_string();
    return formatted_time;
}

fn change_selected(
    content_with_lock: Arc<RwLock<HashMap<WinType, State>>>,
    directory: &mut Directory,
) {
    let mut content = content_with_lock.write().unwrap();
    match &directory.selected {
        Some(dirent_arc) => {
            let dirent = (&**dirent_arc).lock().unwrap();
            let file_content = State::LIST(vec![
                State::VALUE(Item::STRING(format!("{}", dirent.name))),
                State::LIST(vec![
                    State::VALUE(Item::STRING(format!("File Type: {}", get_file_type_as_string(&dirent.file_type)))),
                    State::VALUE(Item::STRING(format!("Size: {}", total_size_to_string(dirent.size)))),
                    State::VALUE(Item::STRING(format!("Permission: {}", dirent.mode))),
                    State::VALUE(Item::STRING(format!(
                        "Last Accessed: {}",
                        system_time_to_string(&dirent.accessed)
                    ))),
                    State::VALUE(Item::STRING(format!("Created: {}", system_time_to_string(&dirent.created)))),
                    State::VALUE(Item::STRING(format!(
                        "Last Modified: {}",
                        system_time_to_string(&dirent.modified)
                    ))),
                    State::VALUE(Item::STRING(format!("Number of Links: {}", dirent.nlink))),
                    State::VALUE(Item::STRING(format!("Dev: {}", dirent.dev))),
                    State::VALUE(Item::STRING(format!("Ino: {}", dirent.ino))),
                    State::VALUE(Item::STRING(format!("Block Size: {}", total_size_to_string(dirent.size)))),
                ]),
            ]);
            content.insert(WinType::FILEWIN, file_content);
        }
        None => {}
    }
}

fn aggregate_n_send(
    content_with_lock: Arc<RwLock<HashMap<WinType, State>>>,
    tx_backend: &Sender<Message>,
    directory: &mut Directory,
    start: usize,
    range: i32,
) {
    aggregate(content_with_lock, directory, start, range);
    // LOG!("Hr");
    let _ = tx_backend.send(Message {
        content: None,
        mtype: MessageType::READDIR,
    });
}

pub fn sort_dirents_by_size(dirents: &mut Vec<Arc<Dirent>>, desc: bool) {
    dirents.sort_by(|d1, d2| {
        let size1 = match &**d1 {
            Dirent::AGGREGATE(mutex) => mutex.lock().unwrap().total_size,
            Dirent::VALUE(mutex) => mutex.lock().unwrap().size,
        };
        let size2 = match &**d2 {
            Dirent::AGGREGATE(mutex) => mutex.lock().unwrap().total_size,
            Dirent::VALUE(mutex) => mutex.lock().unwrap().size,
        };
        if desc {
            size2.cmp(&size1)
        } else {
            size1.cmp(&size2)
        }
    });
}

pub fn sort_dirents_by_name(dirents: &mut Vec<Arc<Dirent>>, desc: bool) {
    dirents.sort_by(|d1, d2| {
        let name1 = match &**d1 {
            Dirent::AGGREGATE(mutex) => mutex.lock().unwrap().common_name.clone(),
            Dirent::VALUE(mutex) => mutex.lock().unwrap().name.clone(),
        };
        let name2 = match &**d2 {
            Dirent::AGGREGATE(mutex) => mutex.lock().unwrap().common_name.clone(),
            Dirent::VALUE(mutex) => mutex.lock().unwrap().name.clone(),
        };
        if desc {
            name2.cmp(&name1)
        } else {
            name1.cmp(&name2)
        }
    });
}

pub fn sort_dir_entry_by_name(dirents: Arc<Dirent>) {
    // LOG!(format!("Here"));
    match &*dirents {
        Dirent::AGGREGATE(mutex) => {
            let agg = &mut mutex.lock().unwrap();
            let desc = agg.sorted_by_name;
            let dirent_vec = &mut agg.dirents;
            dirent_vec.sort_by(|d1, d2| {
                let name1 = &(&**d1).lock().unwrap().name;
                let name2 = &(&**d2).lock().unwrap().name;
                if desc {
                    name2.cmp(&name1)
                } else {
                    name1.cmp(&name2)
                }
            });
            agg.sorted_by_name = !agg.sorted_by_name;
        }
        Dirent::VALUE(_) => {}
    }
}

pub fn sort_dir_entry_by_size(dirents: Arc<Dirent>) {
    match &*dirents {
        Dirent::AGGREGATE(mutex) => {
            let agg = &mut mutex.lock().unwrap();
            let desc = agg.sorted_by_name;
            let dirent_vec = &mut agg.dirents;
            dirent_vec.sort_by(|d1, d2| {
                let size1 = (&**d1).lock().unwrap().size;
                let size2 = (&**d2).lock().unwrap().size;
                if desc {
                    size2.cmp(&size1)
                } else {
                    size1.cmp(&size2)
                }
            });
            agg.sorted_by_size = !agg.sorted_by_size;
        }
        Dirent::VALUE(_) => {}
    }
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
    let root = Arc::new("/home/alonot".to_owned());
    let storage_content = State::LIST(vec![
        State::LIST(vec![
            State::VALUE(Item::STRING(format!("Name"))),
            State::VALUE(Item::STRING(format!("Size"))),
            State::VALUE(Item::STRING(format!("Percent"))),
            State::VALUE(Item::SORT(SortButton {
                name: format!("Sort by Name"),
                context: root.clone(),
            })),
            State::VALUE(Item::SORT(SortButton {
                name: format!("Sort by Size"),
                context: root.clone(),
            })),
        ]),
        State::LIST(vec![]),
    ]);

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

    const MAXDIR: usize = 10;
    {
        let mut content = content_with_lock.write().unwrap();
        content.insert(WinType::FOLDERWIN, folder_content);
        content.insert(WinType::FILEWIN, file_content);
        content.insert(WinType::STORAGEWIN, storage_content);
    }

    thread::spawn(move || -> ! {
        let no_threads = Arc::new(Mutex::new(0));
        let mut directories = HashMap::<Arc<String>,Directory>::new();
        let mut directory_key_order: VecDeque<Arc<String>> = VecDeque::new();
        let mut directory: Option<&mut Directory> = None;
        
        // waits and reads the messages in the channel
        loop {
            LOG!(format!("Here Out"));
            for received in &rx_frontend {
                LOG!(format!("Here In"));
                let no_threads_lc = Arc::clone(&no_threads);
                let content_with_lock_clone = Arc::clone(&content_with_lock);
                match received.mtype {
                    MessageType::READDIR => {
                        let curr_dir_name = match directory.as_deref() {
                            Some(dir) => dir.path.clone().to_string(),
                            None => root.clone().to_string(),
                        };
                        let dir = match received.content {
                            Some(val) => val,
                            None => Arc::new(format!("")),
                        };
                        let next_dir = if curr_dir_name.eq(&dir.to_string()) {
                            dir
                        } else {
                            Arc::new(format!("{}{}", curr_dir_name, dir))
                        };

                        if directory_key_order.len() == MAXDIR {
                            let to_remove = directory_key_order.pop_front().expect("Expected deque");
                            directories.remove(&to_remove);
                        }
                        if directory_key_order.len() == MAXDIR {
                            let to_remove = directory_key_order.pop_front().expect("Expected deque");
                            directories.remove(&to_remove);
                        }
                        if let Entry::Vacant(entry) = directories.entry(next_dir.clone()) {
                            match read_directory(
                                next_dir.clone(),
                                no_threads_lc,
                                content_with_lock_clone,
                                &tx_backend,
                            ) {
                                Ok(val) => {
                                    directory_key_order.push_back(next_dir.clone());
                                    directory = Some(entry.insert(val));
                                }
                                Err(e) => {
                                    println!("{e}");
                                    directory = None;
                                }
                            }
                        } else {
                            directory = directories.get_mut(&next_dir);
                            match directory.as_deref_mut() {
                                Some(dir) => {
                                    aggregate_n_send(content_with_lock.clone(), &tx_backend, dir, 0, -1);
                                },
                                None => {},
                            }
                            
                        };
                    }
                    MessageType::GOBACK => {
                        let mut same_dir: bool = false;

                        let prev_dir = match directory.as_deref() {
                            Some(dir) => {
                                let name = dir.path.clone().to_string();
                                let path = Path::new(&name);
                                match path.parent() {
                                    Some(val) => {
                                        let new_path = 
                                         val
                                            .to_str()
                                            .expect("can't convert to string")
                                            .to_string();
                                        if new_path.eq(&name) {
                                            same_dir = true;
                                        }
                                        if new_path.eq("/") {
                                            Arc::new(format!("{}",new_path))
                                        } else {
                                            Arc::new(format!("{}/",new_path))
                                        }
                                    }
                                    None => {
                                        same_dir = true;
                                        root.clone()
                                    }
                                }
                            }
                            None => root.clone(),
                        };
                        // LOG!(format!("SAME: {} {}",same_dir, is_nodelay(stdscr())));
                        if !same_dir {
                            if directory_key_order.len() == MAXDIR {
                                let to_remove = directory_key_order.pop_front().expect("Expected deque");
                                directories.remove(&to_remove);
                            }
                            if let Entry::Vacant(entry) = directories.entry(prev_dir.clone()) {
                                match read_directory(
                                    prev_dir.clone(),
                                    no_threads_lc,
                                    content_with_lock_clone,
                                    &tx_backend,
                                ) {
                                    Ok(val) => {
                                        directory_key_order.push_back(prev_dir.clone());
                                        directory = Some(entry.insert(val));
                                    }
                                    Err(e) => {
                                        println!("{e}");
                                        directory = None;
                                    }
                                }
                            } else {
                                directory = directories.get_mut(&prev_dir);
                                match directory.as_deref_mut() {
                                    Some(dir) => {
                                        aggregate_n_send(content_with_lock.clone(), &tx_backend, dir, 0, -1);
                                    },
                                    None => {},
                                }
                                
                            };
                        }
                        
                    }
                    MessageType::SORTBYNAME => {
                        let dirent = match received.content {
                            Some(val) => val.to_string(),
                            None => root.clone().to_string(),
                        };
                        // LOG!(format!("{}", dirent));
                        match &mut directory {
                            Some(dir) => {
                                if dirent.eq(&root.to_string()) {
                                    // is root
                                    // LOG!(format!("SORTNAME: {}", dirent));
                                    sort_dirents_by_name(&mut dir.dirents, dir.sorted_by_name);
                                    dir.sorted_by_name = !dir.sorted_by_name;
                                } else {
                                    // find the dirent
                                    // LOG!(format!("{}", dirent));
                                    let mut agg_dirent: Option<Arc<Dirent>> = None;
                                    for dirent_entry in &mut dir.dirents {
                                        let is_this_name = match dirent_entry.as_ref() {
                                            Dirent::AGGREGATE(mutex) => {
                                                let agg = mutex.lock().unwrap();
                                                // LOG!(format!("Agg {} {}",agg.common_name, agg.common_name.eq(&dirent)));
                                                agg.common_name.eq(&dirent)
                                            }
                                            Dirent::VALUE(_) => false,
                                        };
                                        if is_this_name {
                                            agg_dirent = Some(dirent_entry.clone());
                                            break;
                                        }
                                    }
                                    match agg_dirent {
                                        Some(agg_dirent) => {
                                            sort_dir_entry_by_name(agg_dirent);
                                        }
                                        None => {}
                                    }
                                }
                                aggregate(content_with_lock.clone(), dir, 0, -1);
                                let _ = tx_backend.send(Message {
                                    content: None,
                                    mtype: MessageType::RELOADSTORAGE,
                                });
                            }
                            None => {}
                        }
                    }
                    MessageType::SORTBYSIZE => {
                        let dirent = match received.content {
                            Some(val) => val.to_string(),
                            None => root.clone().to_string(),
                        };
                        match &mut directory {
                            Some(dir) => {
                                if dirent.eq(&root.to_string()) {
                                    // is root
                                    sort_dirents_by_size(&mut dir.dirents, dir.sorted_by_size);
                                    dir.sorted_by_size = !dir.sorted_by_size;
                                } else {
                                    // find the dirent
                                    let mut agg_dirent: Option<Arc<Dirent>> = None;
                                    for dirent_entry in &mut dir.dirents {
                                        let is_this_name = match dirent_entry.as_ref() {
                                            Dirent::AGGREGATE(mutex) => {
                                                let agg = mutex.lock().unwrap();
                                                agg.common_name.eq(&dirent)
                                            }
                                            Dirent::VALUE(_) => false,
                                        };
                                        if is_this_name {
                                            agg_dirent = Some(dirent_entry.clone());
                                            break;
                                        }
                                    }
                                    match agg_dirent {
                                        Some(agg_dirent) => {
                                            sort_dir_entry_by_size(agg_dirent);
                                        }
                                        None => {}
                                    }
                                }
                                aggregate(content_with_lock.clone(), dir, 0, -1);
                                let _ = tx_backend.send(Message {
                                    content: None,
                                    mtype: MessageType::RELOADSTORAGE,
                                });
                            }
                            None => {}
                        }
                    }
                    MessageType::RELOADSTORAGE => {
                      // LOG!("Hro");
                        let _ = tx_backend.send(Message {
                            content: None,
                            mtype: MessageType::RELOADSTORAGE,
                        });
                    }
                    MessageType::CHANGEFILEINFO => {
                        let dirent_name = match received.content {
                            Some(val) => val.to_string(),
                            None => root.clone().to_string(),
                        };
                        match &mut directory {
                            Some(dir) => {
                                let mut got = false;
                                for dirent_entry in &dir.dirents {
                                    match dirent_entry.as_ref() {
                                        Dirent::AGGREGATE(mutex) => {
                                            let agg = mutex.lock().unwrap();
                                            for agg_dirent in &agg.dirents {
                                                if agg_dirent.lock().unwrap().name.eq(&dirent_name) {
                                                    dir.selected = Some(agg_dirent.clone());
                                                    got = true;
                                                    break;
                                                }
                                            }
                                        }
                                        Dirent::VALUE(mutex) => {
                                            if mutex.lock().unwrap().name.eq(&dirent_name) {
                                                dir.selected = Some(mutex.clone());
                                                got = true;
                                                break;
                                            }
                                        },
                                    };
                                    if got {
                                        break;
                                    }
                                }
                                change_selected(content_with_lock.clone(), dir);
                                let _ = tx_backend.send(Message {
                                    content: None,
                                    mtype: MessageType::RELOADSTORAGE,
                                });
                            },
                            None => {},
                        }
                    },
                    MessageType::RELOAD => {
                        let _ = tx_backend.send(Message {
                            content: None,
                            mtype: MessageType::RELOAD,
                        });
                    }
                    _ => {}
                }
            }
        }
    });
}
