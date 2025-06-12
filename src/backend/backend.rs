use std::collections::hash_map::Entry;
use std::collections::{vec_deque, HashMap, VecDeque};
use std::fs;
use std::fs::{exists, read_dir, FileType, Metadata};
use std::io::Error;
use std::os::linux::fs::MetadataExt;
use std::os::unix::fs::{FileTypeExt, MetadataExt as UMetaExt, PermissionsExt};
use std::path::{Path, PathBuf};
use std::sync::atomic::AtomicBool;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, RwLock};
use std::sync::{BarrierWaitResult, Mutex};
use std::thread::{sleep, JoinHandle};
use std::time::{Duration, SystemTime};
use std::{io, thread, vec};

use chrono::{DateTime, Utc};
use cncurses::LOGLn;
use models::data_models::{Directory, Dirent, AGGREGATOR};

use crate::models::data_models::{DirInfo, DirectoryOut, Message, DIRENTRY};
use crate::utils::{file_type_to_string, path_last_name, path_to_string};
use crate::{models, total_size_to_string};

fn mode_to_octal(mut mode: u32) -> String {
    let mut octal = format!("");
    while mode != 0 {
        octal += &(mode % 8).to_string();
        mode = mode / 8;
    }
    octal = octal.chars().rev().collect();
    octal
}

fn is_worker_free() -> bool {
    {
        let free_threads = free_workers.read().unwrap();
        if *free_threads == 0 {
            return false;
        }
    }
    let mut free_threads = free_workers.write().unwrap();
    if *free_threads > 0 {
        *free_threads -= 1;
        true
    } else {
        false
    }
}

fn calculate_size(
    dir_path: PathBuf,
    dir_entry: Arc<Mutex<DIRENTRY>>,
    cancel: Arc<AtomicBool>,
) -> io::Result<()> {
    let mut threads: Vec<JoinHandle<()>> = vec![];
    let mut event_cancelled = false;
    let size_res: io::Result<u64> = read_dir(&dir_path)?.try_fold(0, |acc, file| {
        if event_cancelled {
            return Ok(acc);
        }
        let file = file?;
        let dir_entry_c = dir_entry.clone();
        let cancel_c = cancel.clone();
        let size = match file.metadata()? {
            data if data.is_dir() => {
                let is_thread_free = is_worker_free();
                if is_thread_free {
                    threads.push(thread::spawn(move || {
                        let _ = calculate_size(file.path(), dir_entry_c, cancel_c);
                        *free_workers.write().unwrap() += 1;
                    }));
                } else {
                    let _ = calculate_size(file.path(), dir_entry_c, cancel_c);
                }
                // data.size()
                0
            }, 
            data if data.blocks() == 0 => {
                0
            },
            data => {
                data.st_size()
            }
        };
        if cancel.load(std::sync::atomic::Ordering::Relaxed) {
            event_cancelled = true;
        }
        Ok(acc + size)
    });

    
    let size = match size_res {
        Ok(s) => s,
        Err(_) => 0,
    };

    dir_entry.lock().unwrap().size += size;

    for t in threads {
        let _ = t.join();
    }

    Ok(())
}

fn dir_entry_data(metadata: &Metadata, path: &PathBuf) -> Result<DIRENTRY, Error> {
    let dir_entry = DIRENTRY {
        path: path.to_string_lossy().to_string(),
        name: Arc::new(match path.file_name() {
            Some(val) => val.to_string_lossy().to_string(),
            None => format!(""),
        }),
        st_size: metadata.st_blocks() * 512,
        size: metadata.st_blocks() * 512,
        percent: 0.,
        gid: metadata.gid(),
        dev: metadata.dev(),
        nlink: metadata.nlink(),
        ino: metadata.ino(),
        mode: mode_to_octal(metadata.permissions().mode()),
        file_type: file_type_to_string(metadata.file_type()),
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
    cancel: Arc<AtomicBool>,
    directory: Arc<Mutex<Directory>>,
    tx_internal: Arc<Sender<Message>>,
) -> Result<(), String> {
    let mut aggregators: HashMap<Arc<String>, Arc<Mutex<AGGREGATOR>>> = HashMap::new();

    let path = {
        directory
            .lock()
            .unwrap()
            .dir_info
            .lock()
            .unwrap()
            .path
            .clone()
    };

    let dir_path = Path::new(&*path);
    let (tx_readdir, rx_readdir) = {
        let (tx, rx) = channel::<Message>();
        (Arc::new(tx), rx)
    };
    let mut len = 0;

    if dir_path.exists() && dir_path.is_dir() {
        for entry in read_dir(dir_path).map_err(|e| format!("READ DIR ERROR: {}", e))? {
            if cancel.load(std::sync::atomic::Ordering::Relaxed) {
                return Ok(());
            }
            if let Ok(entry) = entry {
                let path = entry.path();
                if let Ok(metadata) = entry.metadata() {
                    // if (metadata.dev() >> 8) & 0xff == 0 {
                    //         continue;
                    //     } // Skip pseudo-files

                    let dir_entry_lk =
                        dir_entry_data(&metadata, &path).map_err(|e| e.to_string())?;
                    let size = dir_entry_lk.size;
                    let dir_entry = Arc::new(Mutex::new(dir_entry_lk));

                    if metadata.is_dir() {
                        let dirent = {
                            let dir = directory.lock().unwrap();
                            let mut dir_info = dir.dir_info.lock().unwrap();
                            dir_info.no_folders += 1;
                            let dirent = Dirent::VALUE(dir_entry.clone());
                            dir.dirents.lock().unwrap().push(dirent);
                            dir_entry
                        };
                        let mut wq = work_queue.lock().unwrap();
                        wq.push_back(Work::CALCULATESIZE(
                            dirent,
                            tx_readdir.clone(),
                            cancel.clone(),
                        ));
                        len += 1;
                    } else {
                        let agg_lk = {
                            let dir = directory.lock().unwrap();
                            let mut dir_info = dir.dir_info.lock().unwrap();
                            dir_info.no_files += 1;
                            let ext = Arc::new(format!(
                                ".{}",
                                path.extension().and_then(|val| val.to_str()).unwrap_or("")
                            ));

                            aggregators.entry(ext.clone()).or_insert_with(|| {
                                let agg = Arc::new(Mutex::new(AGGREGATOR {
                                    common_name: ext.clone(),
                                    percent: 0.,
                                    total_size: 0,
                                    dirents: vec![],
                                    sorted_by_name: false,
                                    sorted_by_size: false,
                                }));
                                dir.dirents
                                    .lock()
                                    .unwrap()
                                    .push(Dirent::AGGREGATE(Arc::clone(&agg)));
                                agg
                            })
                        };
                        let mut agg = agg_lk.lock().unwrap();
                        agg.dirents.push(dir_entry.clone());
                        {
                            let mut direntry = dir_entry.lock().unwrap();
                            agg.total_size += size;
                            direntry.percent =
                                ((size as f64 / agg.total_size as f64) * 100.).round() as f32;
                        }
                    }
                }
            }
        }
    }

    while len > 0 {
        loop {
            if cancel.load(std::sync::atomic::Ordering::Relaxed) {
                return Ok(());
            }

            match rx_readdir.try_recv() {
                Ok(v) => match v {
                    Message::CLOSECONN => {
                        len -= 1;
                        break;
                    }
                    _ => {}
                },
                Err(e) => {
                    match e {
                        std::sync::mpsc::TryRecvError::Empty => {
                            // do nothing
                        }
                        std::sync::mpsc::TryRecvError::Disconnected => {
                            len -= 1;
                            break;
                        }
                    }
                }
            }
        }
    }
    {
        directory.lock().unwrap().fully_loaded = true;
    }
    // LOGLn!(
    //     "HERE, SEnding {}",
    //     directory.lock().unwrap().dir_info.lock().unwrap().no_files
    // );
    let _ = tx_internal.send(Message::CLOSECONN);
    Ok(())
}

#[macro_export]
macro_rules! LOGLn2 {
    ($val:expr $(,$other: expr)* ) => {
        let var = std::fs::read_to_string("debug2.txt").map_or(format!(""), |f| f);
        let _ = std::fs::write("debug2.txt", format!(concat!("{}", $val, "\n"), var,$($other, )*));
    };
}

enum Work {
    READDIR(Arc<Mutex<Directory>>, Arc<Sender<Message>>, Arc<AtomicBool>),
    SENDRESPONSE(
        Arc<Mutex<Directory>>,
        Arc<Mutex<Sender<Message>>>,
        Arc<Mutex<Receiver<Message>>>,
        Arc<AtomicBool>,
    ),
    CALCULATESIZE(Arc<Mutex<DIRENTRY>>, Arc<Sender<Message>>, Arc<AtomicBool>),
}

fn send_response(
    dir: Arc<Mutex<Directory>>,
    tx_backend: Arc<Mutex<Sender<Message>>>,
    rx_internal: Arc<Mutex<Receiver<Message>>>,
    cancel: Arc<AtomicBool>,
) {
    let mut time = SystemTime::now();
    let mut close_con = false;
    loop {
        sleep(Duration::from_millis(10));
        let Ok(curr) = SystemTime::now()
            .duration_since(time)
            .map_err(|e| e.to_string())
        else {
            return;
        };
        if curr >= Duration::from_secs(1) {
            let _ = tx_backend.lock().unwrap().send(Message::UPDATEDIRINFO(
                DirectoryOut::from_directory(dir.clone()).into(),
            ));
            loop {
                if cancel.load(std::sync::atomic::Ordering::Relaxed) {
                    return;
                }

                match rx_internal.lock().unwrap().try_recv() {
                    Ok(v) => match v {
                        Message::UPDATECOMPLETE => {
                            time = SystemTime::now();
                            break;
                        }
                        Message::CLOSECONN => {
                            close_con = true;
                            break;
                        }
                        _ => {}
                    },
                    Err(e) => {
                        match e {
                            std::sync::mpsc::TryRecvError::Empty => {
                                // do nothing
                            }
                            std::sync::mpsc::TryRecvError::Disconnected => {
                                return;
                            }
                        }
                    }
                }
            }
            if close_con {
                break;
            }
        }
        match rx_internal.lock().unwrap().try_recv() {
            Ok(v) => match v {
                Message::CLOSECONN => {
                    break;
                }
                _ => {}
            },
            Err(e) => {
                match e {
                    std::sync::mpsc::TryRecvError::Empty => {
                        // do nothing
                    }
                    std::sync::mpsc::TryRecvError::Disconnected => {
                        break;
                    }
                }
            }
        }
    }
    let _ = tx_backend.lock().unwrap().send(Message::UPDATEDIRINFO(
        DirectoryOut::from_directory(dir.clone()).into(),
    ));
}

fn worker(cancel: Arc<AtomicBool>) {
    loop {
        if cancel.load(std::sync::atomic::Ordering::Relaxed) {
            break;
        }
        let next_work_opt = {
            let mut wq = work_queue.lock().unwrap();
            wq.pop_front()
        };
        let work = match next_work_opt {
            Some(w) => w,
            None => {
                thread::sleep(Duration::from_millis(100));
                continue;
            }
        };
        match work {
            Work::READDIR(dir, tx_internal, cancel) => {
                let res = read_directory(cancel, dir.clone(), tx_internal);
            }
            Work::SENDRESPONSE(dir, tx_backend, rx_internal, cancel) => {
                send_response(dir, tx_backend, rx_internal, cancel);
            }
            Work::CALCULATESIZE(direntry, tx_read_dir, cancel) => {
                let dir_path = PathBuf::from(direntry.lock().unwrap().path.clone());
                if dir_path.exists() {
                    let res = calculate_size(dir_path, direntry, cancel);
                }
                let _ = tx_read_dir.send(Message::CLOSECONN);
            }
        }
    }
}

static work_queue: Mutex<VecDeque<Work>> = Mutex::new(VecDeque::new());
static free_workers: RwLock<i32> = RwLock::new(5);

fn start_workers(cancel: Arc<AtomicBool>) {
    let mut started = 0;
    while started < 5 {
        if is_worker_free() {
            let cancel_c = cancel.clone();
            thread::spawn(|| {
                worker(cancel_c);
                *free_workers.write().unwrap() += 1;
            });
            started += 1;
        }
    }
}

/**
 * Runs backend in another thread
 */
pub fn run_backend(rx_frontend: Receiver<Message>, tx_backend: Sender<Message>) {
    thread::spawn(move || -> ! {
        let mut directories = HashMap::<Arc<PathBuf>, Arc<Mutex<Directory>>>::new();
        let mut directory_key_order: VecDeque<Arc<PathBuf>> = VecDeque::new();

        let tx_backend_arc = Arc::new(Mutex::new(tx_backend));

        let (tx_internal, rx_internal) = {
            let (tx, rx) = channel::<Message>();
            (Arc::new(tx), Arc::new(Mutex::new(rx)))
        };

        let mut cur_workers_flag: Option<Arc<AtomicBool>> = None;
        // waits and reads the messages in the channel
        loop {
            for received in &rx_frontend {
                match received {
                    Message::READDIR(dir_name) => {
                        // check for this dir in cache, if not present
                        // then send threads to load it
                        // threads keep updating the dir_info while they calculate the size and number of folders.
                        // as a dirent is completely evaluated(all info about it if completely fetched), the dirent is sent to the frontend
                        let mut present = true;
                        let dir: Arc<Mutex<Directory>> = directories
                            .entry(dir_name.clone())
                            .or_insert_with(|| {
                                present = false;
                                let dir = Arc::new(Mutex::new(Directory::new(&dir_name)));
                                directory_key_order.push_back(dir_name.clone());
                                dir
                            })
                            .clone();
                        {
                            if let Some(flag) = cur_workers_flag {
                                flag.store(true, std::sync::atomic::Ordering::Relaxed);
                            }
                            let new_worker_flag = Arc::new(AtomicBool::new(false));
                            let mut wq = work_queue.lock().unwrap();
                            if !present || !dir.lock().unwrap().fully_loaded {
                                wq.push_back(Work::READDIR(
                                    dir.clone(),
                                    tx_internal.clone(),
                                    new_worker_flag.clone(),
                                ));
                            }
                            wq.push_back(Work::SENDRESPONSE(
                                dir.clone(),
                                tx_backend_arc.clone(),
                                rx_internal.clone(),
                                new_worker_flag.clone(),
                            ));
                            cur_workers_flag = Some(new_worker_flag.clone());
                            start_workers(new_worker_flag);
                        }
                        let res =
                            tx_backend_arc
                                .lock()
                                .unwrap()
                                .send(Message::UPDATEDIRINFO(Arc::new(
                                    DirectoryOut::from_directory(dir),
                                )));
                    }
                    Message::RELOADDIR(dir_name) => {
                        // reload the dir, even if it is present in the cache
                        let dir = Arc::new(Mutex::new(Directory::new(&dir_name)));
                        directories.insert(dir_name.clone(), dir.clone());
                        directory_key_order.push_back(dir_name.clone());
                        {
                            if let Some(flag) = cur_workers_flag {
                                flag.store(true, std::sync::atomic::Ordering::Relaxed);
                            }
                            let new_worker_flag = Arc::new(AtomicBool::new(false));
                            let mut wq = work_queue.lock().unwrap();
                            wq.push_back(Work::READDIR(
                                dir.clone(),
                                tx_internal.clone(),
                                new_worker_flag.clone(),
                            ));
                            wq.push_back(Work::SENDRESPONSE(
                                dir.clone(),
                                tx_backend_arc.clone(),
                                rx_internal.clone(),
                                new_worker_flag.clone(),
                            ));
                            start_workers(new_worker_flag.clone());
                            cur_workers_flag = Some(new_worker_flag);
                        }
                    }
                    Message::UPDATECOMPLETE => {
                        let _ = tx_internal.send(Message::UPDATECOMPLETE);
                    }
                    _ => {}
                }
            }
        }
    });
}
