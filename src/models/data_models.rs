use std::{
    collections::HashMap,
    fs::{DirEntry, FileType},
    path::PathBuf,
    sync::{Arc, Mutex},
    time::SystemTime,
};

use cncurses::{interfaces::{StateEqual, Stateful}, LOGLn};

use crate::{utils::{compare_arc, path_last_name, path_to_string}, LOGLn2};
#[derive(Debug, Clone)]
pub struct Directory {
    pub dir_info: Arc<Mutex<DirInfo>>,
    pub dirents: Arc<Mutex<Vec<Dirent>>>,
    pub fully_loaded: bool
}

impl Directory {
    pub fn new(path: &Arc<PathBuf>) -> Directory {
        Directory {
            dir_info: Arc::new(Mutex::new(DirInfo {
                name: Arc::new(path_last_name(path)),
                path: Arc::new(path.canonicalize().expect("Unable to canocalize")),
                no_folders: 0,
                no_files: 0,
                total_size: 0,
            })),
            dirents: Arc::new(Mutex::new(vec![])),
            fully_loaded: false,
        }
    }
}
#[derive(Debug, Clone)]
pub struct DirectoryOut {
    pub dir_info: Arc<DirInfo>,
    pub dirents: Arc<Mutex<Vec<Dirent>>>,
    pub sorted_by_size: Arc<Mutex<bool>>,
    pub sorted_by_name: Arc<Mutex<bool>>,
}

impl DirectoryOut {
    pub fn new(path: &Arc<PathBuf>) -> DirectoryOut {
        DirectoryOut {
            dir_info: Arc::new(DirInfo {
                name: Arc::new(path_last_name(path)),
                path: Arc::new(path.canonicalize().expect("Unable to canocalize")),
                no_folders: 0,
                no_files: 0,
                total_size: 0,
            }),
            dirents: Arc::new(Mutex::new(vec![])),
            sorted_by_name: Arc::new(Mutex::new(false)),
            sorted_by_size: Arc::new(Mutex::new(false)),
        }
    }
    pub fn from_directory(dir: Arc<Mutex<Directory>>) -> DirectoryOut {
        let (dir_info_lk, dirent_lk) = {
            let directory = dir.lock().unwrap();
            (directory.dir_info.clone(), directory.dirents.clone())
        };
        let dirents: Arc<Mutex<Vec<Dirent>>> = Mutex::new(dirent_lk.lock().unwrap().clone()).into();
        let mut dirinfo = dir_info_lk.lock().unwrap().clone();
        
        dirinfo.total_size = dirents.clone().lock().unwrap().iter().fold(0, |f, dirent| {
            match dirent {
                Dirent::AGGREGATE(agg_lk) => {
                    let agg = agg_lk.lock().unwrap();
                    f + agg.total_size
                },
                Dirent::VALUE(val_lk) => {
                    let val = val_lk.lock().unwrap();
                    f + val.size
                },
            }
        });
        
        let dir_info: Arc<DirInfo> = dirinfo.into();
        DirectoryOut { dir_info, dirents, 
            sorted_by_name: Arc::new(Mutex::new(false)),
            sorted_by_size: Arc::new(Mutex::new(false)),
        }
    }
}

impl PartialEq for DirectoryOut {
    fn eq(&self, other: &Self) -> bool {
        false
    }
}

#[derive(Debug, Clone,PartialEq)]
pub struct DirInfo {
    pub name: Arc<String>,
    pub path: Arc<PathBuf>,
    pub no_folders: u64,
    pub no_files: u64,
    pub total_size: u64,
}

#[derive(Debug, Clone)]
pub enum Dirent {
    AGGREGATE(Arc<Mutex<AGGREGATOR>>),
    VALUE(Arc<Mutex<DIRENTRY>>),
}

impl PartialEq for Dirent {
    fn eq(&self, other: &Self) -> bool {
        match self {
            Dirent::AGGREGATE(agg_a) => {
                if let Dirent::AGGREGATE(agg_b) = other {
                    compare_arc(agg_a, agg_b)
                } else {
                    false
                }
            }
            Dirent::VALUE(val_a) => {
                if let Dirent::VALUE(val_b) = other {
                    compare_arc(val_a, val_b)
                } else {
                    false
                }
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct AGGREGATOR {
    pub common_name: Arc<String>,
    pub sorted_by_name: bool,
    pub sorted_by_size: bool,
    pub total_size: u64,
    pub percent: f32,
    pub dirents: Vec<Arc<Mutex<DIRENTRY>>>,
}

impl PartialEq for AGGREGATOR {
    fn eq(&self, other: &Self) -> bool {
        let mut eq = self.common_name == other.common_name
        && self.total_size == self.total_size
        && self.sorted_by_name == other.sorted_by_name
        && self.sorted_by_size == other.sorted_by_size
        && self.percent == self.percent;
        if eq {
            eq &= self
            .dirents
                .iter()
                .zip(other.dirents.iter())
                .all(|(a, b)| compare_arc(&a, &b))
            }
            return eq;
        }
    }
    
#[derive(Debug, Clone)]
pub struct AGGHOLDER (pub Arc<Mutex<AGGREGATOR>>);

impl PartialEq for AGGHOLDER {
    fn eq(&self, other: &Self) -> bool {
        false
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct DIRENTRY {
    pub percent: f32,
    pub name: Arc<String>,
    pub path: String,
    pub size: u64,
    pub st_size: u64,
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
pub struct DIRENTHolder(pub Arc<Vec<Arc<Mutex<Dirent>>>>);

impl StateEqual for DIRENTHolder {
    fn equal(&self, other: &Self) -> bool {
        self.0
            .iter()
            .zip(other.0.iter())
            .all(|(a, b)| compare_arc(a, b))
    }
}

#[derive(Debug, Clone)]
pub struct DIRENTRYHolder(pub Arc<Mutex<DIRENTRY>>);

impl PartialEq for DIRENTRYHolder {
    fn eq(&self, other: &Self) -> bool {
        compare_arc(&self.0, &other.0)
    }
}

#[derive(Debug, Clone)]
pub enum DIRENTRYOUT {
    TODIR,
    TOAGG(Arc<String>),
}

#[derive(Clone, Debug)]
pub enum Message {
    // consumed by both
    OPENCONN,
    CLOSECONN,
    ENDCONN,

    // consumed by backend only
    READDIR(Arc<PathBuf>),
    RELOADDIR(Arc<PathBuf>),
    UPDATECOMPLETE,

    // consumed by frontend only
    /** To update the information about the directory itself */
    UPDATEDIRINFO(Arc<DirectoryOut>),
    /** Since all the data is with the backend, It just tells whether the whole directory is updated or only an aggregator.
     * if the aggregator is updated then only Storage window need to be refreshed. Else the Whole Screen
     */
    SENDREPONSE(DIRENTRYOUT),
}
