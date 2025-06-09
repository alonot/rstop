use std::{fs::FileType, os::unix::fs::FileTypeExt, time::SystemTime};

use chrono::{DateTime, Utc};

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
