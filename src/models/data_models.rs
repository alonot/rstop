use std::{
    collections::HashMap, fs::FileType, sync::{Arc, Mutex}, time::SystemTime
};

use cncurses::interfaces::{StateEqual, Stateful};

#[derive(Debug)]
pub struct Directory {
    pub dir_info: Arc<DirInfo>,
    pub dirents: Vec<Arc<Dirent>>,
    pub aggregators: HashMap<Arc<String>, Arc<Mutex<AGGREGATOR>>>
}

#[derive(Debug)]
pub struct DirInfo {
    pub name: Arc<String>,
    pub path: Arc<String>,
    pub no_folders: u64,
    pub no_files: u64,  
    pub total_size: u64,
}

#[derive(Debug,Clone)]
pub enum Dirent {
    AGGREGATE(Arc<Mutex<AGGREGATOR>>),
    VALUE(Arc<Mutex<DIRENTRY>>),
}

#[derive(Debug)]
pub struct AGGREGATOR {
    pub common_name: Arc<String>,
    pub sorted_by_name: bool,
    pub sorted_by_size: bool,
    pub total_size: u64,
    pub percent: f32,
    pub expanded: bool,
    pub dirents: Vec<Arc<Mutex<DIRENTRY>>>,
}


#[derive(Clone, Debug, PartialEq)]
pub struct DIRENTRY {
    pub percent: f32,
    pub name: String,
    pub path: String,
    pub size: u64,
    pub blksize: u64,
    pub gid: u32,
    pub dev: u64,
    pub nlink: u64,
    pub ino: u64,
    pub file_type: String,
    pub modified: SystemTime,
    pub created: SystemTime,
    pub accessed: SystemTime,
    pub mode: String,
}

#[derive(Debug, Clone)]
pub struct DIRENTRYHolder(pub Arc<Mutex<DIRENTRY>>);

impl StateEqual for DIRENTRYHolder {
    fn equal(&self, other: &Self) -> bool {
        Arc::as_ptr(&self.0).eq(&Arc::as_ptr(&other.0))
    }
}

#[derive(Debug, Clone)]
pub enum DIRENTRYOUT {
    TODIR,
    TOAGG(Arc<String>)
}

#[derive(Clone, Debug)]
pub enum Message {
    // consumed by both
    OPENCONN,
    CLOSECONN,
    
    // consumed by backend only
    READDIR(Arc<String>),
    RELOADDIR(Arc<String>),
    UPDATECOMPLETE,

    // consumed by frontend only
    /** To update the information about the directory itself */
    UPDATEDIRINFO(Arc<Directory>),
    /** Since all the data is with the backend, It just tells whether the whole directory is updated or only an aggregator. 
     * if the aggregator is updated then only Storage window need to be refreshed. Else the Whole Screen
    */
    SENDREPONSE(DIRENTRYOUT),
}
