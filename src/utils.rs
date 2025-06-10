use std::{fs::FileType, os::unix::fs::FileTypeExt, sync::Arc, time::SystemTime};

use chrono::{DateTime, Utc};

use crate::models::data_models::Dirent;

pub fn total_size_to_string(total_size: u64) -> String {
    let kb = total_size as f64 / 1024.;
    if kb <= 1. {
        return format!("{:05.2} B", total_size);
    }
    let mb = kb / 1024.;
    if mb <= 1. {
        return format!("{:05.2} kb", kb);
    }
    let gb = mb / 1024.;
    if gb <= 1. {
        return format!("{:05.2} mb", mb);
    }
    let tb = gb / 1024.;
    if tb <= 1. {
        return format!("{:05.2} gb", gb);
    } else {
        return format!("{:05.2} tb", tb);
    }
}

pub fn file_type_to_string(ftype: FileType) -> String {
    {
        if ftype.is_dir() {
            "DIR".to_string()
        } else if ftype.is_file() {
            "FILE".to_string()
        } else if ftype.is_symlink() {
            "SYMLINK".to_string()
        } else if ftype.is_block_device() {
            "BLOCK DEVICE".to_string()
        } else if ftype.is_char_device() {
            "CHAR DEVICE".to_string()
        } else if ftype.is_socket() {
            "SOCKET".to_string()
        } else if ftype.is_fifo() {
            "FIFO".to_string()
        } else {
            "".to_string()
        }
    }
}

pub fn system_time_to_string(d: SystemTime) -> String {
    let datetime: DateTime<Utc> = d.into();
    format!("{}", datetime.format("%d/%m/%Y %T"))
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
            Dirent::VALUE(mutex) => mutex.lock().unwrap().name.clone().into(),
        };
        let name2 = match &**d2 {
            Dirent::AGGREGATE(mutex) => mutex.lock().unwrap().common_name.clone(),
            Dirent::VALUE(mutex) => mutex.lock().unwrap().name.clone().into(),
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
                let size1 = &(&**d1).lock().unwrap().size;
                let size2 = &(&**d2).lock().unwrap().size;
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

