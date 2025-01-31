mod backend;
mod models;

use std::any::Any;
use std::collections::HashMap;
use std::sync::mpsc::channel;
use std::sync::{Mutex, RwLock};
use std::{ffi::NulError, sync::Arc};

use backend::run_backend;
use models::models::{DimensionType, Item, Message, MessageType, Screen, State};
use models::windows::{FileInfoWin, TextBox, Window};
use ncurses::{
    clear, curs_set, endwin, getch, initscr, keypad, noecho, refresh, stdscr, KEY_RESIZE,
};

fn init_screen(
    content_with_lock: Arc<RwLock<HashMap<Arc<String>, State>>>,
    folder_win: Arc<String>,
    file_win: Arc<String>,
    progress_win: Arc<String>,
) -> Result<Screen, NulError> {
    initscr();
    clear();
    curs_set(ncurses::CURSOR_VISIBILITY::CURSOR_INVISIBLE);
    // cbreak();
    keypad(stdscr(), true);
    noecho();
    let mut screen = Screen::new();
    refresh();
    screen.update_height();
    let width_30p = (0.3 * screen.dim.width as f32).floor() as i32;
    let height_30p = (0.3 * screen.dim.height as f32).floor() as i32;

    let mut style: HashMap<String, &dyn Any> = HashMap::new();
    style.insert("with_border".to_owned(), &true);

    screen.add_window(
        folder_win.clone(),
        Box::new(Window::new(
            format!("Folder"),
            0,
            0,
            DimensionType::DIMENS(-1),
            DimensionType::PERCEN(0.3),
            Some(&style),
        )),
    );
    screen.add_window(
        file_win.clone(),
        Box::new(Window::new(
            "File Info".to_string(),
            width_30p,
            0,
            DimensionType::PERCEN(0.3),
            DimensionType::DIMENS(-1),
            Some(&style),
        )),
    );
    screen.add_window(
        progress_win.clone(),
        Box::new(Window::new(
            "Storage".to_string(),
            width_30p,
            height_30p - 1,
            DimensionType::DIMENS(-1),
            DimensionType::DIMENS(-1),
            Some(&style),
        )),
    );

    screen.refresh_screen(true)?;

    let content: std::sync::RwLockReadGuard<'_, HashMap<Arc<String>, State>> = content_with_lock.read().unwrap();

    let folderwindow = screen.get_window(folder_win.clone());
    let folder_content = content.get(&folder_win).expect("Error");

    folderwindow.add_child(Box::new(TextBox::new(
        0,
        -1,
        DimensionType::DIMENS(4),
        DimensionType::DIMENS(-1),
    )));

    match folder_content {
        State::LIST(states) => {
            let last_ind = states.len() - 1;
            states.iter().enumerate().for_each(|(ind, _)| {
                if last_ind != ind {
                    folderwindow.add_child(Box::new(TextBox::new(
                        0,
                        -1,
                        DimensionType::DIMENS(1),
                        DimensionType::PERCEN(1.),
                    )));
                }
            });
        }
        State::VALUE(_) => {}
    }

    let filewindow = screen.get_window(file_win.clone());
    filewindow.add_child(Box::new(TextBox::new(
        0,
        -1,
        DimensionType::DIMENS(1),
        DimensionType::PERCEN(1.),
    )));
    filewindow.add_child(Box::new(FileInfoWin::new(
        0,
        -1,
        DimensionType::DIMENS(-1),
        DimensionType::PERCEN(1.),
    )));

    screen.refresh_screen(true)?;

    screen.populate(content)?;

    // // turn this "true" atlast will result in flushing of the value populated before
    screen.refresh_screen(false)?;

    Ok(screen)
}

fn total_size_to_string(total_size: u64) -> String {
    let kb = total_size as f64 / 1024.;
    let mb = kb / 1024.;
    if mb <= 1. {
        return format!("{} kb", kb);
    }
    let gb = mb / 1024.;
    if gb <= 1. {
        return format!("{} mb", mb);
    }
    let tb = gb / 1024.;
    if tb <= 1. {
        return format!("{} gb", gb);
    } else {
        return format!("{} tb", tb);
    }
}

fn main() -> Result<(), NulError> {
    let folder_win = Arc::new("FOLDER".to_string());
    let file_win = Arc::new("FILE".to_string());
    let progress_win = Arc::new("PROGRESS".to_string());

    let mut names: Vec<State> = vec![];
    let (tx_frontend, rx_frontend) = channel::<Message>();
    let (tx_backend, rx_backend) = channel::<Message>();

    // let directory = match read_directory(Arc::new("/".to_owned()), no_threads_lc) {
    //     Ok(val) => Some(val),
    //     Err(e) => {
    //         println!("{e}");
    //         None
    //     }
    // };
    // 

    let mut content = Arc::new(RwLock::new(HashMap::new()));
    // content.insert(folder_win.clone(), folder_content);
    // content.insert(file_win.clone(), file_content);

    let dir = Arc::new(format!("/"));
    
    run_backend(
        content.clone(),
        folder_win.clone(),
        file_win.clone(),
        progress_win.clone(),
        rx_frontend,
        tx_backend,
    );

    let mut screen = init_screen(
        content.clone(),
        folder_win.clone(),
        file_win.clone(),
        progress_win.clone(),
    )?;

    let _ = tx_frontend.send(Message{
        content: dir.clone(),
        mtype: MessageType::READDIR
    }).map_err(|e| format!("{e:?}"));

    loop {
        match rx_backend.try_recv() {
            Ok(message) => {
                Ok(())
            },
            Err(e) => {
                match e {
                    std::sync::mpsc::TryRecvError::Empty => {/*Do nothing */
                        Ok(())
                    },
                    std::sync::mpsc::TryRecvError::Disconnected => {
                        println!("{:?}", e);
                        // later propogate or handle the error correctly
                        Ok(())
                    },
                }
            },
        }?;

        let ch = getch();
        if ch == KEY_RESIZE {
            screen.update_height();
            init_screen(
                content.clone(),
                folder_win.clone(),
                file_win.clone(),
                progress_win.clone(),
            )?;
        }
        if ch == 'q' as i32 || ch == 'Q' as i32 {
            break;
        }
    }

    endwin();
    Ok(())
}
