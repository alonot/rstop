mod backend;
mod models;
mod util;
// cargo build; time sudo ./target/debug/rstop

use std::any::Any;
use std::collections::HashMap;
use std::fs;
use std::sync::mpsc::{channel, Sender};
use std::sync::RwLock;
use std::thread::sleep;
use std::time::Duration;
use std::{ffi::NulError, sync::Arc};

use backend::run_backend;
use models::models::{
    DimensionType, DisplayContent, Item, Message, MessageType, Screen, State, WinType, STYLETYPE,
};
use models::windows::{FileInfoWin, HeaderWin, ScrollView, StorageWin, TextBox, Window};
use ncurses::{
    cbreak, clear, curs_set, doupdate, endwin, flushinp, getch, getmouse, initscr, is_nodelay, keypad, mmask_t, mouseinterval, mousemask, nodelay, noecho, refresh, stdscr, timeout, ALL_MOUSE_EVENTS, BUTTON1_PRESSED, BUTTON3_PRESSED, BUTTON4_PRESSED, BUTTON5_PRESSED, ERR, KEY_MOUSE, KEY_RESIZE, MEVENT, OK
};

#[macro_export]
macro_rules! LOG {
    ($val:expr) => {
        let val = fs::read_to_string("log.txt").map_or(format!(""), |f| f);
        let _ = fs::write("log.txt", format!("{val}\nLOGGED: {:?} ", $val));
    };
}

fn init_screen(
    screen: &mut Screen,
    content_with_lock: Arc<RwLock<HashMap<WinType, State>>>,
    _: Arc<Sender<Message>>,
) -> Result<(), NulError> {
    // cbreak();
    clear();
    curs_set(ncurses::CURSOR_VISIBILITY::CURSOR_INVISIBLE);
    cbreak();
    keypad(stdscr(), true);
    noecho();
    refresh();
    screen.clear_n_new();
    nodelay(stdscr(), true); // make getch non-blocking
    screen.update_height();
    let width_30p = (0.3 * screen.dim.width as f32).floor() as i32;
    let height_30p = (0.3 * screen.dim.height as f32).floor() as i32;

    let mut style: HashMap<String, &dyn Any> = HashMap::new();
    style.insert("with_border".to_owned(), &true);

    screen.add_window(
        WinType::FOLDERWIN,
        Box::new(ScrollView::new(
            format!("Folder"),
            0,
            0,
            DimensionType::DIMENS(-1),
            DimensionType::PERCEN(0.3),
            Some(&style),
            vec![(STYLETYPE::FULLBORDER, 0)],
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
            vec![(STYLETYPE::FULLBORDER, 0)],
        )),
    );

    let content: std::sync::RwLockReadGuard<'_, HashMap<WinType, State>> =
        content_with_lock.read().unwrap();
    let folder_content = content.get(&WinType::FOLDERWIN).expect("Expected folder");

    screen.add_window(
        WinType::STORAGEWIN,
        Box::new(ScrollView::new(
            "Storage".to_string(),
            width_30p,
            height_30p - 1,
            DimensionType::DIMENS(-1),
            DimensionType::DIMENS(-1),
            Some(&style),
            vec![(STYLETYPE::FULLBORDER, 0)],
        )),
    );

    screen.refresh_screen(true)?;

    let clicked: Arc<dyn Fn(&mut State, Arc<Sender<Message>>) -> Result<bool, String>> = Arc::new(
        |t: &mut State, tx_frontend: Arc<Sender<Message>>| -> Result<bool, String> {
            match t {
                State::LIST(_) => {}
                State::VALUE(item) => {
                    match item {
                        Item::STRING(val) => {
                            // Convert &mut String to String

                            let _ = tx_frontend.send(Message {
                                content: Some(Arc::new(format!("{}/", val.to_string()))),
                                mtype: MessageType::READDIR,
                            });
                        }
                        Item::DIRECTORY(_) => {}
                        Item::SORT(_) => {}
                    }
                }
            }
            Ok(true)
        },
    );

    let folderwindow = screen.get_window(WinType::FOLDERWIN);

    let mut current_folder_win = Box::new(Window::new(
        format!(""),
        0,
        -1,
        DimensionType::DIMENS(4),
        DimensionType::DIMENS(-1),
        Some(&style),
        vec![(STYLETYPE::BOTTOMBORDER, 1)],
    ));

    current_folder_win.add_child(Box::new(TextBox::new(
        0,
        -1,
        DimensionType::DIMENS(2),
        DimensionType::DIMENS(-1),
        None,
        vec![],
    )));

    let back_clicked: Arc<dyn Fn(&mut State, Arc<Sender<Message>>) -> Result<bool, String>> =
        Arc::new(
            |_: &mut State, tx_frontend: Arc<Sender<Message>>| -> Result<bool, String> {
                let _ = tx_frontend.send(Message {
                    content: None,
                    mtype: MessageType::GOBACK,
                });

                Ok(true)
            },
        );

    style.clear();
    style.insert(format!("left_click"), &back_clicked);

    current_folder_win.add_child(Box::new(TextBox::new(
        0,
        2,
        DimensionType::DIMENS(1),
        DimensionType::DIMENS(10),
        Some(&style),
        vec![],
    )));

    let sort_by_name: Arc<dyn Fn(&mut State, Arc<Sender<Message>>) -> Result<bool, String>> =
        Arc::new(
            |t: &mut State, tx_frontend: Arc<Sender<Message>>| -> Result<bool, String> {
                match t {
                    State::LIST(_) => {}
                    State::VALUE(item) => {
                        if let Item::SORT(val) = item {
                            let _ = tx_frontend.send(Message {
                                content: Some(val.context.clone()),
                                mtype: MessageType::SORTBYNAME,
                            });
                        }
                    }
                }

                Ok(true)
            },
        );

    let sort_by_size: Arc<dyn Fn(&mut State, Arc<Sender<Message>>) -> Result<bool, String>> =
        Arc::new(
            |t: &mut State, tx_frontend: Arc<Sender<Message>>| -> Result<bool, String> {
                match t {
                    State::LIST(_) => {}
                    State::VALUE(item) => {
                        if let Item::SORT(val) = item {
                            let _ = tx_frontend.send(Message {
                                content: Some(val.context.clone()),
                                mtype: MessageType::SORTBYSIZE,
                            });
                        }
                    }
                }

                Ok(true)
            },
        );

    folderwindow.add_child(current_folder_win);

    style.clear();
    style.insert(format!("left_click"), &clicked);

    match folder_content {
        State::LIST(states) => {
            states.iter().for_each(|_| {
                folderwindow.add_child(Box::new(TextBox::new(
                    0,
                    -1,
                    DimensionType::DIMENS(1),
                    DimensionType::PERCEN(1.),
                    Some(&style),
                    vec![],
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
        vec![],
    )));
    filewindow.add_child(Box::new(FileInfoWin::new(
        0,
        -1,
        DimensionType::DIMENS(-1),
        DimensionType::PERCEN(1.),
        vec![],
    )));

    let storagewindow = screen.get_window(WinType::STORAGEWIN);
    let storage_content = content.get(&WinType::STORAGEWIN).expect("Expected storage");

    storagewindow.add_child(Box::new(HeaderWin::new(
        0,
        -1,
        DimensionType::DIMENS(1),
        DimensionType::PERCEN(1.),
        sort_by_name,
        sort_by_size,
        vec![],
    )));

    let mut storage_inner_window = Box::new(ScrollView::new(
        format!(""),
        0,
        -1,
        DimensionType::DIMENS(-1),
        DimensionType::DIMENS(-1),
        None,
        vec![(STYLETYPE::BOTTOMBORDER, 1)],
    ));

    match storage_content {
        State::LIST(states) => {
            match &states[1] {
                // LOG!(format!("{}",states.len()));
                State::LIST(states) => {
                    states.iter().for_each(|_| {
                        storage_inner_window.add_child(Box::new(StorageWin::new(
                            0,
                            -1,
                            DimensionType::DIMENS(1),
                            DimensionType::PERCEN(1.),
                            vec![],
                        )));
                    });
                }
                State::VALUE(_) => {}
            }
        }
        State::VALUE(_) => {}
    }
    storagewindow.add_child(storage_inner_window);

    screen.refresh_screen(true)?;

    screen.populate(content)?;

    // // turn this "true" atlast will result in flushing of the value populated before
    screen.refresh_screen(false)?;

    Ok(())
}

fn total_size_to_string(total_size: u64) -> String {
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

fn main() -> Result<(), NulError> {
    let _ = fs::write("log.txt", "");
    let (tx_frontend, rx_frontend) = channel::<Message>();
    let (tx_backend, rx_backend) = channel::<Message>();

    let content: Arc<RwLock<HashMap<WinType, State>>> = Arc::new(RwLock::new(HashMap::new()));

    let dir = Arc::new(format!("/"));

    run_backend(content.clone(), rx_frontend, tx_backend);
    let tx_frontend_arc = Arc::new(tx_frontend);

    let mut screen = Screen::new();

    initscr();

    init_screen(&mut screen, content.clone(), tx_frontend_arc.clone())?;

    let _ = tx_frontend_arc
        .send(Message {
            content: Some(dir.clone()),
            mtype: MessageType::READDIR,
        })
        .map_err(|e| format!("{e:?}"));

    mousemask(
        ALL_MOUSE_EVENTS as mmask_t,
        None,
    );
    mouseinterval(0);

    nodelay(stdscr(), true); // make getch non-blocking
    LOG!(format!("is_delay value {}", is_nodelay(stdscr())));

    loop {
        // LOG!("Hello");
        rx_backend.try_iter().try_for_each(|message| {
            LOG!(format!("Recieved {:?}", message.mtype));
            match message.mtype {
                MessageType::RELOADSTORAGE => {
                    // LOG!("RELOADSTORAGE");
                    screen.refresh_screen(true)?;
                    let content_without_lock = (*content).read().unwrap();
                    screen.populate(content_without_lock)?;
                    // // turn this "true" atlast will result in flushing of the value populated before
                    screen.refresh_screen(false)?;
                    nodelay(stdscr(), true); // make getch non-blocking
                }
                MessageType::RELOAD => {
                    clear();
                    nodelay(stdscr(), true); // make getch non-blocking
                    // refresh();
                    screen.clear_screen()?;
                    let content_without_lock = (*content).read().unwrap();
                    screen.populate(content_without_lock)?;
                    // // turn this "true" atlast will result in flushing of the value populated before
                    screen.refresh_screen(false)?;
                }
                MessageType::READDIR => {
                    LOG!("READDIR START");
                    init_screen(&mut screen, content.clone(), tx_frontend_arc.clone())?;
                    LOG!("READDIR END");
                }
                _ => {}
            }
            // println!("{:?} ",message.mtype);
            Ok(())
        })?;
        // flushinp();
        // timeout(0);
        // LOG!(format!("GOING {}", is_nodelay(stdscr())));
        let ch = getch();
        if ch == ERR {
            // LOG!("errr");
            // flushinp();
        } else {
            // LOG!(format!("getch value {}", ch));
            // LOG!(format!("is_delay value {}", is_nodelay(stdscr())));
        }

        if ch == KEY_RESIZE {
            screen.update_height();
            init_screen(&mut screen, content.clone(), tx_frontend_arc.clone())?;
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
                // LOG!(format!(
                //     "5:{} 4:{} 1:{} 2:{} {}",
                //     event.bstate & BUTTON5_PRESSED as u32,
                //     event.bstate & BUTTON4_PRESSED as u32,
                //     event.bstate & BUTTON1_PRESSED as u32,
                //     event.bstate & BUTTON3_PRESSED as u32,
                //     event.id
                // ));
                let _ = screen.checkMouseClick(
                    content.write().unwrap(),
                    &mut event,
                    tx_frontend_arc.clone(),
                );
                LOG!("Finished");
            }
        }
    }

    endwin();
    Ok(())
}
