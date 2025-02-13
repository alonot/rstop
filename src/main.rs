mod backend;
mod models;
mod util;
// cargo build; time sudo ./target/debug/rstop

use core::panic;
use std::any::Any;
use std::collections::HashMap;
use std::fs;
use std::sync::mpsc::{channel, Sender};
use std::sync::RwLock;
use std::{ffi::NulError, sync::Arc};

use backend::run_backend;
use models::models::{
    DimensionType, DisplayContent, Item, Message, MessageType, Screen, Selected, State, WinType,
    STYLETYPE,
};
use models::windows::{FileInfoWin, HeaderWin, ScrollView, StorageWin, TextBox, Window};
use ncurses::{
    attr_on, bkgd, cbreak, clear, curs_set, endwin, getch, getmouse, has_colors, init_color, init_pair, initscr, is_nodelay, keypad, mmask_t, mouseinterval, mousemask, nodelay, noecho, refresh, start_color, stdscr, use_default_colors, wbkgd, ALL_MOUSE_EVENTS, BUTTON1_PRESSED, BUTTON4_PRESSED, BUTTON5_PRESSED, COLOR_BLACK, COLOR_BLUE, COLOR_CYAN, COLOR_GREEN, COLOR_MAGENTA, COLOR_PAIR, COLOR_RED, COLOR_WHITE, COLOR_YELLOW, ERR, KEY_DOWN, KEY_ENTER, KEY_LEFT, KEY_MOUSE, KEY_RESIZE, KEY_RIGHT, KEY_UP, MEVENT, OK
};
use util::*;

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
    selected: &mut Selected,
) -> Result<(), NulError> {
    // cbreak();
    clear();
    curs_set(ncurses::CURSOR_VISIBILITY::CURSOR_INVISIBLE);
    cbreak();
    keypad(stdscr(), true);
    noecho();
    start_color();
    initialize_colors();
    use_default_colors();
    refresh();
    bkgd(COLOR_PAIR(PAIR_WHITE_BLACK));
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
            vec![
                (STYLETYPE::STARTCOLOR, COLOR_PAIR(PAIR_GOLD_BLACK)),
                (STYLETYPE::FULLBORDER, 0),
            ],
            vec![(STYLETYPE::REMOVECOLOR, COLOR_PAIR(PAIR_GOLD_BLACK))],
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
            vec![
                (STYLETYPE::STARTCOLOR, COLOR_PAIR(PAIR_TURQUOISE_BLACK)),
                (STYLETYPE::FULLBORDER, 0),
            ],
            vec![(STYLETYPE::REMOVECOLOR, COLOR_PAIR(PAIR_TURQUOISE_BLACK))],
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
            vec![
                (STYLETYPE::STARTCOLOR, COLOR_PAIR(PAIR_GREEN_BLACK)),
                (STYLETYPE::FULLBORDER, 0),
            ],
            vec![(STYLETYPE::REMOVECOLOR, COLOR_PAIR(PAIR_GREEN_BLACK))],
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
                        _ => {}
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
        vec![
            (STYLETYPE::STARTCOLOR, COLOR_PAIR(PAIR_RED_BLACK)),
            (STYLETYPE::BOTTOMBORDER, 1),
        ],
        vec![(STYLETYPE::REMOVECOLOR, COLOR_PAIR(PAIR_RED_BLACK))],
    ));

    current_folder_win.add_child(Box::new(TextBox::new(
        0,
        -1,
        DimensionType::DIMENS(2),
        DimensionType::DIMENS(-1),
        None,
        vec![(STYLETYPE::STARTCOLOR, COLOR_PAIR(PAIR_BLUE_BLACK))],
        vec![(STYLETYPE::REMOVECOLOR, COLOR_PAIR(PAIR_BLUE_BLACK))],
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
        vec![(STYLETYPE::STARTCOLOR, COLOR_PAIR(PAIR_RED_BLACK))],
        vec![(STYLETYPE::REMOVECOLOR, COLOR_PAIR(PAIR_RED_BLACK))],
    )));

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
        vec![(STYLETYPE::STARTCOLOR, COLOR_PAIR(PAIR_CYAN_BLACK))],
        vec![(STYLETYPE::REMOVECOLOR, COLOR_PAIR(PAIR_CYAN_BLACK))],
    )));
    filewindow.add_child(Box::new(FileInfoWin::new(
        0,
        -1,
        DimensionType::DIMENS(-1),
        DimensionType::PERCEN(1.),
        vec![],
        vec![],
    )));

    let storagewindow = screen.get_window(WinType::STORAGEWIN);
    let storage_content = content.get(&WinType::STORAGEWIN).expect("Expected storage");

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

    storagewindow.add_child(Box::new(HeaderWin::new(
        0,
        -1,
        DimensionType::DIMENS(1),
        DimensionType::PERCEN(1.),
        sort_by_name,
        sort_by_size,
        vec![],
        vec![],
    )));

    let mut storage_inner_window = Box::new(ScrollView::new(
        format!("Storage"),
        0,
        -1,
        DimensionType::DIMENS(-1),
        DimensionType::DIMENS(-1),
        None,
        vec![(STYLETYPE::BOTTOMBORDER, 1)],
        vec![],
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

    screen.change_bg_selected(selected, PAIR_BLACK_YELLOW, false);
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

fn initialize_colors() {
    // Define custom colors (R, G, B values range from 0-1000)
    // init_color(CCOLOR_GOLD, 1000, 843, 0); // RGB(255, 215, 0)
    // init_color(CCOLOR_ORANGE, 1000, 549, 0); // RGB(255, 140, 0)

    // init_color(CCOLOR_LIGHT_RED, 1000, 400, 400); // Light red
    // init_color(CCOLOR_PINK, 1000, 600, 800);      // Pink
    // init_color(CCOLOR_MAGENTA, 800, 200, 800);    // Magenta
    // init_color(CCOLOR_DARK_PURPLE, 600, 0, 600);  // Dark purple
    // init_color(CCOLOR_PURPLE, 500, 0, 500);       // Purple
    // init_color(CCOLOR_TURQUOISE, 200, 800, 800);  // Turquoise

    // init_color(CCOLOR_GRADIENT_1, 1000, 27, 1000);
    // init_color(CCOLOR_GRADIENT_2, 227, 43, 443);
    // init_color(CCOLOR_GRADIENT_3, 329, 71, 576);
    // init_color(CCOLOR_GRADIENT_4, 447, 106, 678);
    // init_color(CCOLOR_GRADIENT_5, 576, 141, 753);
    // init_color(CCOLOR_GRADIENT_6, 706, 176, 808);
    // init_color(CCOLOR_GRADIENT_7, 800, 180, 580);
    // init_color(CCOLOR_GRADIENT_8, 800, 325, 200);

    // Keep specified color pairs
    init_pair(PAIR_YELLOW_PURPLE, COLOR_YELLOW, CCOLOR_DARK_PURPLE);
    init_pair(PAIR_LIGHTRED_PURPLE, CCOLOR_LIGHT_RED, CCOLOR_PURPLE);
    init_pair(PAIR_PINK_YELLOW, CCOLOR_PINK, COLOR_YELLOW);
    init_pair(PAIR_CYAN_MAGENTA, COLOR_CYAN, CCOLOR_MAGENTA);
    init_pair(PAIR_BLACK_GOLD, COLOR_BLACK, CCOLOR_GOLD);
    init_pair(PAIR_BLACK_TURQUOISE, COLOR_BLACK, CCOLOR_TURQUOISE);

    // Black background pairs for each color
    init_pair(PAIR_WHITE_BLACK, COLOR_WHITE, COLOR_BLACK);
    init_pair(PAIR_RED_BLACK, COLOR_RED, COLOR_BLACK);
    init_pair(PAIR_GREEN_BLACK, COLOR_GREEN, COLOR_BLACK);
    init_pair(PAIR_BLUE_BLACK, COLOR_BLUE, COLOR_BLACK);
    init_pair(PAIR_CYAN_BLACK, COLOR_CYAN, COLOR_BLACK);
    init_pair(PAIR_MAGENTA_BLACK, COLOR_MAGENTA, COLOR_BLACK);
    init_pair(PAIR_YELLOW_BLACK, COLOR_YELLOW, COLOR_BLACK);

    init_pair(PAIR_BLACK_WHITE, COLOR_BLACK, COLOR_WHITE);
    init_pair(PAIR_BLACK_RED, COLOR_BLACK, COLOR_RED);
    init_pair(PAIR_BLACK_GREEN, COLOR_BLACK, COLOR_GREEN);
    init_pair(PAIR_BLACK_BLUE, COLOR_BLACK, COLOR_BLUE);
    init_pair(PAIR_BLACK_CYAN, COLOR_BLACK, COLOR_CYAN);
    init_pair(PAIR_BLACK_MAGENTA, COLOR_BLACK, COLOR_MAGENTA);
    init_pair(PAIR_BLACK_YELLOW, COLOR_BLACK, COLOR_YELLOW);

    // Black background for custom colors
    init_pair(PAIR_GOLD_BLACK, CCOLOR_GOLD, COLOR_BLACK);
    init_pair(PAIR_ORANGE_BLACK, CCOLOR_ORANGE, COLOR_BLACK);
    init_pair(PAIR_PURPLE_BLACK, CCOLOR_PURPLE, COLOR_BLACK);
    init_pair(PAIR_DARKPURPLE_BLACK, CCOLOR_DARK_PURPLE, COLOR_BLACK);
    init_pair(PAIR_TURQUOISE_BLACK, CCOLOR_TURQUOISE, COLOR_BLACK);

    // Define color pairs (White foreground, gradient background)
    // init_pair(PAIR_WHITE_GRADIENT_1, COLOR_WHITE, CCOLOR_GRADIENT_1);
    // init_pair(PAIR_WHITE_GRADIENT_2, COLOR_WHITE, CCOLOR_GRADIENT_2);
    // init_pair(PAIR_WHITE_GRADIENT_3, COLOR_WHITE, CCOLOR_GRADIENT_3);
    // init_pair(PAIR_WHITE_GRADIENT_4, COLOR_WHITE, CCOLOR_GRADIENT_4);
    // init_pair(PAIR_WHITE_GRADIENT_5, COLOR_WHITE, CCOLOR_GRADIENT_5);
    // init_pair(PAIR_WHITE_GRADIENT_6, COLOR_WHITE, CCOLOR_GRADIENT_6);
    // init_pair(PAIR_WHITE_GRADIENT_7, COLOR_WHITE, CCOLOR_GRADIENT_7);
    // init_pair(PAIR_WHITE_GRADIENT_8, COLOR_WHITE, CCOLOR_GRADIENT_8);
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

    let mut selected = Selected {
        win_type: WinType::FOLDERWIN,
        index: vec![0, 0],
    };

    initscr();
    if !has_colors() {
        panic!("No colors")
    }
    init_screen(
        &mut screen,
        content.clone(),
        tx_frontend_arc.clone(),
        &mut selected,
    )?;

    let _ = tx_frontend_arc
        .send(Message {
            content: Some(dir.clone()),
            mtype: MessageType::READDIR,
        })
        .map_err(|e| format!("{e:?}"));

    mousemask(ALL_MOUSE_EVENTS as mmask_t, None);
    mouseinterval(0);

    nodelay(stdscr(), true); // make getch non-blocking

    loop {
        // LOG!("Hello");
        rx_backend.try_iter().try_for_each(|message| {
            // LOG!(format!("Recieved {:?}", message.mtype));
            match message.mtype {
                MessageType::RELOADSTORAGE => {
                    // LOG!("RELOADSTORAGE");
                    screen.refresh_screen_of(WinType::STORAGEWIN, true)?;
                    let content_without_lock = (*content).read().unwrap();
                    screen.change_bg_selected(&mut selected, PAIR_BLACK_YELLOW, false);
                    screen.populate_of(WinType::STORAGEWIN, content_without_lock)?;
                    // // turn this "true" atlast will result in flushing of the value populated before
                    screen.refresh_screen_of(WinType::STORAGEWIN, false)?;
                }
                MessageType::RELOADFOLDER => {
                    // LOG!("RELOADSTORAGE");
                    screen.refresh_screen_of(WinType::FOLDERWIN, true)?;
                    let content_without_lock = (*content).read().unwrap();
                    screen.change_bg_selected(&mut selected, PAIR_BLACK_YELLOW, false);
                    screen.populate_of(WinType::FOLDERWIN, content_without_lock)?;
                    // // turn this "true" atlast will result in flushing of the value populated before
                    screen.refresh_screen_of(WinType::FOLDERWIN, false)?;
                }
                MessageType::RELOADFILE => {
                    // LOG!("RELOADSTORAGE");
                    screen.refresh_screen_of(WinType::FILEWIN, true)?;
                    let content_without_lock = (*content).read().unwrap();
                    screen.change_bg_selected(&mut selected, PAIR_BLACK_YELLOW, false);
                    screen.populate_of(WinType::FILEWIN, content_without_lock)?;
                    // // turn this "true" atlast will result in flushing of the value populated before
                    screen.refresh_screen_of(WinType::FILEWIN, false)?;
                }
                MessageType::RELOAD => {
                    clear();
                    nodelay(stdscr(), true); // make getch non-blocking
                    screen.clear_screen()?;
                    let content_without_lock = (*content).read().unwrap();
                    screen.populate(content_without_lock)?;
                    // // turn this "true" atlast will result in flushing of the value populated before
                    screen.refresh_screen(false)?;
                }
                MessageType::READDIR => {
                    // LOG!("READDIR START");
                    selected = Selected {
                        win_type: WinType::FOLDERWIN,
                        index: vec![0, 0],
                    };
                    init_screen(
                        &mut screen,
                        content.clone(),
                        tx_frontend_arc.clone(),
                        &mut selected,
                    )?;
                    // LOG!("READDIR END");
                }
                _ => {}
            }
            Ok(())
        })?;
        let ch = getch();
        // if (ch != ERR) {
        // LOG!(format!("{}", ch));
        // }

        if ch == KEY_RESIZE {
            screen.update_height();
            init_screen(
                &mut screen,
                content.clone(),
                tx_frontend_arc.clone(),
                &mut selected,
            )?;
        } else if ch == 'q' as i32 || ch == 'Q' as i32 {
            break;
        } else if ch == KEY_MOUSE {
            let mut event = MEVENT {
                id: 0,
                x: 0,
                y: 0,
                z: 0,
                bstate: 0,
            };

            if getmouse(&mut event) == OK {
                let _ = screen.checkMouseClick(
                    content.write().unwrap(),
                    &mut event,
                    tx_frontend_arc.clone(),
                );
                // LOG!("Finished");
            }
        } else if ch == 10 {
            let (y, x) = screen.get_xy(&mut selected);
            let mut event = MEVENT {
                id: 0,
                x,
                y,
                z: 0,
                bstate: 2,
            };
            // LOG!("sending");
            let _ = screen.checkMouseClick(
                content.write().unwrap(),
                &mut event,
                tx_frontend_arc.clone(),
            );
        } else if ch == KEY_DOWN {
            screen.change_bg_selected(&mut selected, PAIR_WHITE_BLACK, true);
            let (win_type, visited) = screen.select_down(&mut selected);
            if !visited {
                let window = screen.get_window(win_type);
                let dim = window.get_dim_unmut();
                let mut event = MEVENT {
                    id: 0,
                    x: dim.startx + 2,
                    y: dim.starty + 2,
                    z: 0,
                    bstate: 2097152, // scroll down event
                };
                // LOG!("sending");
                let _ = screen.checkMouseClick(
                    content.write().unwrap(),
                    &mut event,
                    tx_frontend_arc.clone(),
                );
            }
            screen.refresh_screen_of(win_type, true);
            screen.change_bg_selected(&mut selected, PAIR_BLACK_YELLOW, false);
            screen.populate_of(win_type, content.clone().read().unwrap());
            screen.refresh_screen_of(win_type, false);
        } else if ch == KEY_UP {
            screen.change_bg_selected(&mut selected, PAIR_WHITE_BLACK, true);
            let (win_type, visited) = screen.select_up(&mut selected);
            LOG!(format!("??{}", visited));
            if !visited {
                let (y, x) = screen.get_xy(&mut selected);
                let mut event = MEVENT {
                    id: 0,
                    x,
                    y,
                    z: 0,
                    bstate: 65536, // scroll up event
                };
                // LOG!("sending");
                let _ = screen.checkMouseClick(
                    content.write().unwrap(),
                    &mut event,
                    tx_frontend_arc.clone(),
                );
            }
            screen.refresh_screen_of(win_type, true);
            screen.change_bg_selected(&mut selected, PAIR_BLACK_YELLOW, false);
            screen.populate_of(win_type, content.clone().read().unwrap());
            screen.refresh_screen_of(win_type, false);
        } else if ch == KEY_RIGHT {
            screen.change_bg_selected(&mut selected, PAIR_WHITE_BLACK, true);
            screen.select_right(&mut selected);
            screen.refresh_screen(true);
            screen.change_bg_selected(&mut selected, PAIR_BLACK_YELLOW, false);
            screen.populate(content.clone().read().unwrap());
            screen.refresh_screen(false);
        } else if ch == KEY_LEFT {
            screen.change_bg_selected(&mut selected, PAIR_WHITE_BLACK, true);
            screen.select_left(&mut selected);
            screen.refresh_screen(true);
            screen.change_bg_selected(&mut selected, PAIR_BLACK_YELLOW, false);
            screen.populate(content.clone().read().unwrap());
            screen.refresh_screen(false);
        }
    }

    endwin();
    Ok(())
}
