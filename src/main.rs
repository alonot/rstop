mod backend;
mod models;
mod util;
// cargo build; time sudo ./target/debug/rstop

use std::any::Any;
use std::collections::HashMap;
use std::fs;
use std::io::StdinLock;
use std::process::exit;
use std::sync::mpsc::{channel, Sender};
use std::sync::{Mutex, RwLock};
use std::{ffi::NulError, sync::Arc};

use backend::run_backend;
use models::models::{DimensionType, DisplayContent, Item, Message, MessageType, Screen, State, WinType, STYLETYPE};
use models::windows::{FileInfoWin, StorageWin, TextBox, Window};
use ncurses::{
    cbreak, clear, curs_set, endwin, getch, getmouse, initscr, keypad, mmask_t, mousemask, mvprintw, mvwprintw, newwin, nodelay, noecho, printw, raw, refresh, stdscr, timeout, wattron, wborder, wgetch, wrefresh, ALL_MOUSE_EVENTS, BUTTON1_CLICKED, BUTTON1_PRESSED, BUTTON2_PRESSED, BUTTON3_PRESSED, BUTTON4_PRESSED, KEY_MOUSE, KEY_RESIZE, MEVENT, OK
};

#[macro_export]
macro_rules! LOG {
    ($val:expr) => {
        let val = fs::read_to_string("log.txt").map_or(format!(""), |f| f);
        fs::write("log.txt", format!("{val}\nLOGGED: {:?} ", $val));
    };
}

fn init_screen(
    content_with_lock: Arc<RwLock<HashMap<WinType, State>>>,
    tx_frontend: Arc<Sender<Message>>,
) -> Result<Screen, NulError> {
    initscr();
    // cbreak();
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
        WinType::FOLDERWIN,
        Box::new(Window::new(
            format!("Folder"),
            0,
            0,
            DimensionType::DIMENS(-1),
            DimensionType::PERCEN(0.3),
            Some(&style),
            vec![(STYLETYPE::FULLBORDER,0)]
        )),
    );
    screen.add_window(
        WinType::FILEWIN,
        Box::new(Window::new(
            "File Info".to_string(),
            width_30p,
            0,
            DimensionType::PERCEN(0.3),
            DimensionType::DIMENS(-1),
            Some(&style),
            vec![(STYLETYPE::FULLBORDER,0)]
        )),
    );
    let content: std::sync::RwLockReadGuard<'_, HashMap<WinType, State>> =
        content_with_lock.read().unwrap();
    let folder_content = content.get(&WinType::FOLDERWIN).expect("Expected folder");

    screen.add_window(
        WinType::STORAGEWIN,
        Box::new(Window::new(
            "Storage".to_string(),
            width_30p,
            height_30p - 1,
            DimensionType::DIMENS(-1),
            DimensionType::DIMENS(-1),
            Some(&style),
            vec![(STYLETYPE::FULLBORDER,0)]
        )),
    );

    screen.refresh_screen(true)?;

    let tx_frontend_clone = tx_frontend.clone();
    let clicked: Arc<dyn Fn(&mut State) -> Result<bool, String>> =
        Arc::new(move |t: &mut State| -> Result<bool, String> {
            match t {
                State::LIST(states) => {}
                State::VALUE(item) => {
                    match item {
                        Item::STRING(val) => {
                            // Convert &mut String to String

                            let _ = tx_frontend_clone.send(Message {
                                content: Some(Arc::new(val.to_string())),
                                mtype: MessageType::READDIR,
                            });
                        }
                        Item::DIRECTORY(dir_info) => {}
                    }
                }
            }
            Ok(true)
        });

    let folderwindow = screen.get_window(WinType::FOLDERWIN);

    let mut current_folder_win = Box::new(Window::new(
        format!(""),
        0,
        -1,
        DimensionType::DIMENS(5),
        DimensionType::DIMENS(-1),
        Some(&style),   
        vec![(STYLETYPE::BOTTOMBORDER,1)]
    ));

    current_folder_win.add_child(Box::new(TextBox::new(
        0,
        -1,
        DimensionType::DIMENS(2),
        DimensionType::DIMENS(-1),
        None,   
        vec![]
    )));

    let back_clicked: Arc<dyn Fn(&mut State) -> Result<bool, String>> =
        Arc::new( move |t: &mut State| -> Result<bool, String> {
            
            let _ = tx_frontend.send(Message {
                content: None,
                mtype: MessageType::GOBACK,
            });
            
            Ok(true)
        });

        style.clear();
        style.insert(format!("left_click"), &back_clicked);
    
    current_folder_win.add_child(Box::new(TextBox::new(
        0,
        3,
        DimensionType::DIMENS(1),
        DimensionType::DIMENS(10),
        Some(&style),   
        vec![]
    )));

    folderwindow.add_child(current_folder_win);

    style.clear();
    style.insert(format!("left_click"), &clicked);

    match folder_content {
        State::LIST(states) => {
            states.iter().enumerate().for_each(|(ind, _)| {
                folderwindow.add_child(Box::new(TextBox::new(
                    0,
                    -1,
                    DimensionType::DIMENS(1),
                    DimensionType::PERCEN(1.),
                    Some(&style),
                    vec![]
                )));
            });
        }
        State::VALUE(_) => {}
    }

    let filewindow = screen.get_window(WinType::FILEWIN);
    filewindow.add_child(Box::new(TextBox::new(
        0,
        -1,
        DimensionType::DIMENS(1),
        DimensionType::PERCEN(1.),
        None,
        vec![]
    )));
    filewindow.add_child(Box::new(FileInfoWin::new(
        0,
        -1,
        DimensionType::DIMENS(-1),
        DimensionType::PERCEN(1.),
        vec![]
    )));

    let storagewindow = screen.get_window(WinType::STORAGEWIN);
    let storage_content = content.get(&WinType::STORAGEWIN).expect("Expected storage");

    match storage_content {
        State::LIST(states) => {
            // LOG!(format!("{}",states.len()));
            states.iter().for_each(|_| {
                storagewindow.add_child(Box::new(StorageWin::new(
                    0,
                    -1,
                    DimensionType::DIMENS(1),
                    DimensionType::PERCEN(1.),
                    vec![]
                )));
            });
        },
        State::VALUE(_) => {},
    }

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

fn main() -> Result<(), NulError> {
    let _ = fs::write("log.txt", "");
    let (tx_frontend, rx_frontend) = channel::<Message>();
    let (tx_backend, rx_backend) = channel::<Message>();

    let content: Arc<RwLock<HashMap<WinType, State>>> = Arc::new(RwLock::new(HashMap::new()));

    let dir = Arc::new(format!("/"));

    run_backend(content.clone(), rx_frontend, tx_backend);
    let tx_frontend_arc = Arc::new(tx_frontend);

    let mut screen = init_screen(content.clone(), tx_frontend_arc.clone())?;

    let _ = tx_frontend_arc
        .send(Message {
            content: Some(dir.clone()),
            mtype: MessageType::READDIR,
        })
        .map_err(|e| format!("{e:?}"));

    nodelay(stdscr(), true); // make getch non-blocking
    mousemask(ALL_MOUSE_EVENTS as mmask_t, None);

    loop {
        match rx_backend.try_recv() {
            Ok(message) => {
                screen = init_screen(content.clone(), tx_frontend_arc.clone())?;
                // println!("{:?} ",message.mtype);
                Ok(())
            }
            Err(e) => {
                match e {
                    std::sync::mpsc::TryRecvError::Empty => {
                        /*Do nothing */
                        Ok(())
                    }
                    std::sync::mpsc::TryRecvError::Disconnected => {
                        println!("{:?}", e);
                        // later propogate or handle the error correctly
                        exit(1);
                        Ok(())
                    }
                }
            }
        }?;

        let ch = getch();
        if ch == KEY_RESIZE {
            screen.update_height();
            screen = init_screen(content.clone(), tx_frontend_arc.clone())?;
        } else if ch == 'q' as i32 || ch == 'Q' as i32 {
            break;
        } else if ch == KEY_MOUSE {
            // let val = fs::read_to_string("log.txt").map_or(format!(""), |f| f);
            let mut event = MEVENT {
                id: 0,
                x: 0,
                y: 0,
                z: 0,
                bstate: 0,
            };

            if getmouse(&mut event) == OK {
                // LOG!(format!("CALLED: {}", event.bstate & BUTTON1_PRESSED as u32 == 0));
                // fs::write("log.txt", format!("{val}\nREmoved {event:?}"));
                screen.checkMouseClick(content.write().unwrap(), &mut event);
            }
        }
        // checking for mouseclicks in respective windows
        // timeout(10);
        // fs::write("logs.txt", "here");
    }

    endwin();
    Ok(())
}
