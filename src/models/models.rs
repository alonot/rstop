use std::{
    collections::HashMap,
    ffi::NulError,
    fs,
    hash::Hash,
    sync::{mpsc::Sender, Arc, Mutex},
};

use ncurses::{
    attr_t, attroff, attron, box_, clear, doupdate, getmaxx, getmaxyx, getyx, keypad, ll::WINDOW,
    mvwhline, mvwprintw, mvwvline, newpad, newwin, nodelay, prefresh, refresh, stdscr, waddch,
    wattroff, wattrset, wbkgd, wclear, wmove, wprintw, wrefresh, ACS_DARROW, ACS_HLINE, ACS_VLINE,
    BUTTON1_PRESSED, BUTTON2_PRESSED, BUTTON3_PRESSED, BUTTON4_PRESSED, BUTTON5_PRESSED,
    COLOR_PAIR, LINES, MEVENT,
};

use crate::{
    total_size_to_string,
    util::{PAIR_BLACK_YELLOW, PAIR_WHITE_BLACK, PAIR_YELLOW_BLACK},
    LOG,
};

use super::data_models::Dirent;

pub struct Screen {
    windows: HashMap<WinType, Box<dyn DisplayContent>>,
    current_dir: Arc<String>,
    pub dim: Dimension,
}

impl Screen {
    pub fn clear_n_new(&mut self) {
        self.windows.clear();
        self.current_dir = Arc::new("/".to_string());
        self.dim = Dimension {
            height: 0,
            width: 0,
            startx: 0,
            starty: 0,
            scrollx: 0,
            scrolly: 0,
            lines: 0,
            initial_startx: -1,
            initial_starty: -1,
            display_height: DimensionType::DIMENS(-1),
            display_width: DimensionType::DIMENS(-1),
        };
    }

    pub fn new() -> Screen {
        Screen {
            windows: HashMap::new(),
            current_dir: Arc::new("/".to_string()),
            dim: Dimension {
                height: 0,
                width: 0,
                startx: 0,
                starty: 0,
                scrollx: 0,
                scrolly: 0,
                lines: 0,
                initial_startx: -1,
                initial_starty: -1,
                display_height: DimensionType::DIMENS(-1),
                display_width: DimensionType::DIMENS(-1),
            },
        }
    }
    pub fn add_window(&mut self, name: WinType, win: Box<dyn DisplayContent>) {
        self.windows.insert(name, win);
    }

    pub fn update_height(&mut self) {
        getmaxyx(stdscr(), &mut self.dim.height, &mut self.dim.width);
        // self.dim.height -= 1;
        // self.dim.width -= 1;
        // self.dim.height = LINES();
    }

    pub fn get_window(&mut self, name: WinType) -> &mut Box<dyn DisplayContent> {
        self.windows
            .get_mut(&name)
            .expect(&format!("Key {name:?} not found"))
    }

    pub fn refresh_screen(&mut self, re_size: bool) -> Result<(), NulError> {
        clear();
        refresh();
        let mut pdimension = None;
        if re_size {
            pdimension = Some(self.dim);
        }
        let mut cumx = 0;
        let mut cumy = 0;
        self.windows.iter_mut().try_for_each(|(_, win)| {
            (cumx, cumy) = win.display_content(pdimension, cumx, cumy, 0)?;
            Ok(())
        })?;
        doupdate();
        Ok(())
    }

    pub fn refresh_screen_of(&mut self, wintype: WinType, re_size: bool) -> Result<(), NulError> {
        refresh();
        let mut pdimension = None;
        if re_size {
            pdimension = Some(self.dim);
        }
        match self.windows.get_mut(&wintype) {
            Some(win) => {
                win.display_content(pdimension, 0, 0, 0)?;
            }
            None => {}
        }
        Ok(())
    }

    pub fn clear_screen(&mut self) -> Result<(), NulError> {
        clear();
        refresh();
        wmove(stdscr(), 0, 0);
        self.windows.iter_mut().try_for_each(|(_, win)| {
            win.clear_win()?;
            Ok(())
        })?;
        Ok(())
    }

    pub fn populate(
        &mut self,
        content: std::sync::RwLockReadGuard<'_, HashMap<WinType, State>>,
    ) -> Result<(), NulError> {
        self.windows.iter_mut().try_for_each(|(name, win)| {
            let win_content = content.get(name);
            match win_content {
                Some(val) => win.populate(val),
                None => Ok(()),
            }
        })?;
        Ok(())
    }

    // pub fn display_selected(&mut self, color: i16,set_val : bool) {
    //     let selected = &self.selected;
    //     match &selected.win {
    //         Some(win) => {
    //             let mut win = (*win).lock().unwrap();
    //             win.set_style_on(set_val);
    //             match win.get_win() {
    //                 Some(win) => {
    //                     wbkgd(win, COLOR_PAIR(color));
    //                 }
    //                 None => {}
    //             }
    //             match win.get_pad() {
    //                 Some(win) => {
    //                     wbkgd(win, COLOR_PAIR(color));
    //                 }
    //                 None => {}
    //             }
    //         },
    //         None => {

    //         },
    //     }
    // }

    // pub fn select_right(&mut self) {
    //     self.display_selected(PAIR_WHITE_BLACK, true);
    //     let selected = &mut self.selected;
    //     match selected.select_type {
    //         WinType::FOLDERWIN => {
    //             // go to folder win
    //             selected.select_type = WinType::STORAGEWIN;
    //             selected.idx1 = 0;
    //             selected.idx2 = 0;
    //             let folderwin = self.windows.get_mut(&WinType::STORAGEWIN).expect("Expected WINTYPE");
    //             let children = folderwin.get_children();

    //         },
    //         WinType::FILEWIN => todo!(),
    //         WinType::STORAGEWIN => {

    //         },
    //     }
    // }

    pub fn change_bg_selected(&mut self, selected: &mut Selected, color: i16, set_val: bool) {
        let mut child = self
            .windows
            .get_mut(&selected.win_type)
            .expect("Expected WINTYPE");
        let mut children = child.get_children();
        let mut len = children.len() as i32;
        for index in selected.index.iter().take(selected.index.len() - 1) {
            // LOG!(format!("{} {}", index, len));
            if *index < len {
                child = &mut children[*index as usize];
            } else {
                break;
            }
            children = child.get_children();
            len = children.len() as i32;
        }
        match selected.index.last() {
            Some(ind) => {
                child = &mut children[*ind as usize];
                match child.get_win() {
                    Some(win) => {
                        wbkgd(win, COLOR_PAIR(color));
                    }
                    None => {}
                }
                match child.get_pad() {
                    Some(win) => {
                        wbkgd(win, COLOR_PAIR(color));
                    }
                    None => {}
                }
                child.set_style_on(set_val);
            }
            None => todo!(),
        }
    }

    pub fn get_xy(&mut self, selected: &mut Selected) -> (i32, i32) {
        let mut child = self
            .windows
            .get_mut(&selected.win_type)
            .expect("Expected WINTYPE");
        let mut children = child.get_children();
        let mut len = children.len() as i32;
        for index in selected.index.iter().take(selected.index.len() - 1) {
            // LOG!(format!("{} {}", index, len));
            if *index < len {
                child = &mut children[*index as usize];
            } else {
                break;
            }
            children = child.get_children();
            len = children.len() as i32;
        }
        match selected.index.last() {
            Some(ind) => {
                child = &mut children[*ind as usize];
                let dim = child.get_dim_unmut();
                (dim.starty + (dim.height), dim.startx + (dim.width))
            }
            None => (-1, -1),
        }
    }

    pub fn select_up(&mut self, selected: &mut Selected) -> (WinType, bool) {
        // calculating next possible index
        let mut child = self
            .windows
            .get_mut(&selected.win_type)
            .expect("Expected WINTYPE");
        let mut children = child.get_children();
        let mut len = children.len() as i32;
        let mut refresh_folder_win = true;
        let mut visited = true;
        let mut width = 0;
        let mut height = 0;
        getmaxyx(stdscr(), &mut height, &mut width);
        match selected.win_type {
            WinType::FOLDERWIN => {
                let zero_ind = selected.index[0];
                selected.index.clear();
                if zero_ind == 1 {
                    selected.index = vec![(zero_ind - 1) % len as i32, 1];
                } else if zero_ind == 0 {
                    selected.index = vec![0, 0];
                } else {
                    selected.index = vec![(zero_ind - 1) % len as i32];
                    if selected.index[0] == 0 {
                        selected.index.push(0);
                    }
                }
                visited = children[selected.index[0] as usize].get_visited();
            }
            WinType::FILEWIN => todo!(),
            WinType::STORAGEWIN => {
                refresh_folder_win = false;
                let zero_ind = selected.index[0];
                let one_ind = selected.index[1];
                child = &mut children[1];
                children = child.get_children();
                len = children.len() as i32;
                if one_ind == 0 {
                    selected.index = vec![0, 4];
                } else if zero_ind == 0 {
                    selected.index = vec![0, 3];
                } else {
                    let prev_ind = one_ind - 1;
                    let mut at_end = true;
                    if selected.index.len() >= 4 {
                        // expanded
                        let next_child = &mut children[one_ind as usize];
                        let next_children = next_child.get_children();
                        let third_ind = selected.index[3];
                        let two_ind = selected.index[2];
                        at_end = false;
                        if two_ind == 1 {
                            if third_ind == 4 {
                                selected.index = vec![1, one_ind, 1, 3];
                            } else {
                                selected.index = vec![1, one_ind, 0];
                            }
                            visited = children[selected.index[0] as usize].get_visited();
                        } else if two_ind == 2 {
                            if third_ind != 0 {
                                let all_children = next_children[2].get_children();
                                selected.index = vec![
                                    1,
                                    one_ind,
                                    2,
                                    (third_ind - 1) % all_children.len() as i32,
                                    0,
                                ];
                                visited = all_children[selected.index[3] as usize].get_visited();
                            } else {
                                selected.index = vec![1, one_ind, 1, 4];
                                visited = children[selected.index[0] as usize].get_visited();
                            }
                        } else {
                            at_end = true;
                        }
                    }
                    if at_end {
                        let prev_child = &mut children[prev_ind as usize];
                        let prev_children = prev_child.get_children();
                        if prev_children.len() == 3 {
                            // expanded
                            let all_children = prev_children[2].get_children();
                            selected.index = vec![1, prev_ind, 2, all_children.len() as i32 - 1, 0];
                            visited = all_children[selected.index[3] as usize].get_visited();
                        } else {
                            selected.index = vec![1, prev_ind % len as i32, 0];
                            visited = children[selected.index[0] as usize].get_visited();
                        }
                    }
                }
            }
        }
        if refresh_folder_win {
            (WinType::FOLDERWIN, visited)
        } else {
            (WinType::STORAGEWIN, visited)
        }
    }

    pub fn select_left(&mut self, selected: &mut Selected) {
        match selected.win_type {
            WinType::FOLDERWIN => {
                selected.index.clear();
                selected.win_type = WinType::STORAGEWIN;
                selected.index = vec![0, 3];
            }
            WinType::FILEWIN => todo!(),
            WinType::STORAGEWIN => {
                selected.win_type = WinType::FOLDERWIN;
                selected.index = vec![0, 0];
            }
        }
    }

    pub fn select_down(&mut self, selected: &mut Selected) -> (WinType, bool) {
        let mut refresh_folder_win = true;

        let mut child = self
            .windows
            .get_mut(&selected.win_type)
            .expect("Expected WINTYPE");
        let mut children = child.get_children();
        let mut len = children.len() as i32;
        let mut visited = true;
        let mut width = 0;
        let mut height = 0;
        getmaxyx(stdscr(), &mut height, &mut width);
        match selected.win_type {
            WinType::FOLDERWIN => {
                let zero_ind = selected.index[0];
                let one_ind = selected.index.get(1).map_or(1, |f| *f);
                selected.index.clear();
                if zero_ind == 0 && one_ind == 0 {
                    selected.index = vec![(zero_ind) % len as i32, 1];
                } else {
                    selected.index = vec![(zero_ind + 1) % len as i32];
                    if selected.index[0] == 0 {
                        selected.index.push(0);
                    }
                }
                let dim = children[selected.index[0] as usize].get_dim_unmut();
                if dim.startx >= width || dim.starty >= height {
                    visited = false;
                }
            }
            WinType::FILEWIN => todo!(),
            WinType::STORAGEWIN => {
                refresh_folder_win = false;
                let zero_ind = selected.index[0];
                let one_ind = selected.index[1];
                child = &mut children[1];
                children = child.get_children();
                len = children.len() as i32;
                if zero_ind == 0 {
                    if one_ind == 3 {
                        selected.index = vec![0, (one_ind + 1)];
                    } else {
                        selected.index = vec![1, 0, 0];
                    }
                } else {
                    let next_child = &mut children[one_ind as usize];
                    let next_children = next_child.get_children_unmut();
                    if next_children.len() == 3 {
                        let two_ind = selected.index[2];
                        // expanded
                        if two_ind == 0 {
                            selected.index = vec![1, one_ind, 1, 3];
                        } else if two_ind == 1 {
                            let third_ind = selected.index[3];
                            if third_ind == 3 {
                                selected.index = vec![1, one_ind, 1, 4];
                            } else {
                                selected.index = vec![1, one_ind, 2, 0, 0];
                            }
                        } else {
                            let all_children = next_children[2].get_children_unmut();
                            let third_ind = selected.index[3];
                            if third_ind < all_children.len() as i32 - 1 {
                                selected.index = vec![
                                    1,
                                    one_ind,
                                    2,
                                    (third_ind + 1) % all_children.len() as i32,
                                    0,
                                ];
                                let dim = all_children[selected.index[3] as usize].get_dim_unmut();
                                if dim.startx >= width || dim.starty >= height {
                                    visited = false;
                                }
                            } else if one_ind < children.len() as i32 - 1 {
                                selected.index = vec![1, (one_ind + 1) % len as i32, 0];
                            } else {
                                selected.index = vec![1, one_ind, 0];
                            }
                        }
                    } else {
                        // collapsed
                        selected.index = vec![1, (one_ind + 1).min(len - 1), 0];
                        let dim = children[selected.index[1] as usize].get_dim_unmut();
                        if dim.startx + dim.width > width || dim.starty + dim.height > height {
                            visited = false;
                        }
                    }
                }
            }
        }

        if refresh_folder_win {
            (WinType::FOLDERWIN, visited)
        } else {
            (WinType::STORAGEWIN, visited)
        }
    }

    pub fn select_right(&mut self, selected: &mut Selected) {
        match selected.win_type {
            WinType::FOLDERWIN => {
                selected.index.clear();
                selected.win_type = WinType::STORAGEWIN;
                selected.index = vec![0, 3];
            }
            WinType::FILEWIN => todo!(),
            WinType::STORAGEWIN => {
                selected.win_type = WinType::FOLDERWIN;
                selected.index = vec![0, 0];
            }
        }
    }

    pub fn populate_of(
        &mut self,
        wintype: WinType,
        content: std::sync::RwLockReadGuard<'_, HashMap<WinType, State>>,
    ) -> Result<(), NulError> {
        match self.windows.get_mut(&wintype) {
            Some(win) => {
                let win_content = content.get(&wintype);
                match win_content {
                    Some(val) => win.populate(val),
                    None => Ok(()),
                }?;
            }
            None => {}
        }
        Ok(())
    }

    pub fn checkMouseClick(
        &mut self,
        mut content: std::sync::RwLockWriteGuard<'_, HashMap<WinType, State>>,
        event: &mut MEVENT,
        tx_frontend: Arc<Sender<Message>>,
    ) -> Result<(), String> {
        self.windows.iter_mut().try_for_each(|(name, win)| {
            let win_content = content.get_mut(name);
            match win_content {
                Some(val) => win.check_mouse_event(val, event, tx_frontend.clone()),
                None => Ok(()),
            }
        })?;
        Ok(())
    }
}

#[derive(Clone, Copy)]
pub struct Dimension {
    pub height: i32,
    pub width: i32,
    pub startx: i32,
    pub starty: i32,
    pub scrollx: i32,
    pub scrolly: i32,
    pub lines: u32,
    pub initial_startx: i32,
    pub initial_starty: i32,
    pub display_height: DimensionType,
    pub display_width: DimensionType,
}

/**
 * Applies the styles to the win in given order
 */
pub fn apply_stylying(win: WINDOW, dim: &Dimension, styles: &Vec<(STYLETYPE, attr_t)>) {
    let mut y: i32 = 0;
    let mut x: i32 = 0;
    getyx(win, &mut y, &mut x);
    for (style_type, attr) in styles {
        match style_type {
            STYLETYPE::LEFTBORDER => {
                mvwvline(win, 0, 0 - *attr as i32, ACS_VLINE(), dim.height);
            }
            STYLETYPE::TOPBORDER => {
                mvwhline(win, 0 - *attr as i32, 0, ACS_HLINE(), dim.width);
                // LOG!(format!("{} {}", y, x));
            }
            STYLETYPE::BOTTOMBORDER => {
                mvwhline(win, dim.height - *attr as i32, 0, ACS_HLINE(), dim.width);
            }
            STYLETYPE::RIGHTBORDER => {
                mvwvline(win, 0, dim.width, ACS_VLINE(), dim.height);
            }
            STYLETYPE::STARTCOLOR => {
                wattrset(win, *attr);
            }
            STYLETYPE::REMOVECOLOR => {
                wattroff(win, *attr);
            }
            STYLETYPE::FULLBORDER => {
                box_(win, 0, 0);
            }
            STYLETYPE::SPECIALCHARS => {
                waddch(win, *attr);
                getyx(win, &mut y, &mut x);
            }
        }
    }
    wmove(win, y, x); // to again move the print cursor where it was
}

/**
 * The Top level modular box, All UI Windows implement this
 */
pub trait DisplayContent {
    /**
     * returns ncurses::WINDOW
     */
    fn get_win(&self) -> Option<WINDOW>;
    /**
     * return ncurses::WINDOW (declared as pad)
     */
    fn get_pad(&self) -> Option<WINDOW>;
    /**
     * sets ncurses::WINDOW
     */
    fn set_win(&mut self, window: WINDOW);
    fn set_visited(&mut self, _: bool);
    fn get_visited(&mut self) -> bool;
    fn set_style_on(&mut self, _: bool);
    fn get_style_on(&mut self) -> bool;
    /**
     * sets ncurses::WINDOW (declared as pad)
     */
    fn set_pad(&mut self, window: WINDOW);
    fn clear_for_resize(&mut self);
    fn get_title(&self) -> Option<String>;
    fn create_win(&self) -> bool;
    fn get_children(&mut self) -> &mut Vec<Box<dyn DisplayContent>>;
    fn get_children_unmut(&self) -> &Vec<Box<dyn DisplayContent>>;
    fn add_child(&mut self, win: Box<dyn DisplayContent>);
    /**
     * returns styling to be applied in a specific order to be applied before populating content
     */
    fn get_style_before_populate(&self) -> &Vec<(STYLETYPE, attr_t)>;

    /**
     * returns styling to be applied in a specific order to be applied after populating content
     */
    fn get_style_after_populate(&self) -> &Vec<(STYLETYPE, attr_t)>;

    /**
     * Returns Dimension
     */
    fn get_dim(&mut self) -> &mut Dimension;
    fn get_dim_unmut(&self) -> &Dimension;

    /**
     * Return NulError if the object does not expect to print anything
     */
    fn display_state(&mut self, state: &Item) -> Result<(), NulError>;

    fn left_click(&mut self, _: &mut State, _: Arc<Sender<Message>>) -> Result<bool, String> {
        Ok(false)
    }
    fn right_click(&mut self, _: &mut State, _: Arc<Sender<Message>>) -> Result<bool, String> {
        Ok(false)
    }
    fn scroll_up(&mut self, _: &mut State, _: Arc<Sender<Message>>) -> Result<bool, String> {
        Ok(false)
    }
    fn scroll_down(&mut self, _: &mut State, _: Arc<Sender<Message>>) -> Result<bool, String> {
        Ok(false)
    }

    ////////////////////////////     DEFAULT        //////////////////////////////////////////
    /**
     * sets all visited value for all the child of this.
     * Eg. if parent's visited is false(i.e. it is not visible), then all its children must also be set to false(they are also not visible)
     */
    fn set_visited_all(&mut self, val: bool) {
        self.set_visited(val);
        self.get_children().iter_mut().for_each(|child| {
            child.set_visited_all(val);
        });
    }

    /**
     * Calculates the states of a dirent
     * If dirent is aggregator then returns different set when expanded,
     * else different set of states.
     */
    fn calculate_next_states(&mut self, dir_info: &Arc<Dirent>) -> Vec<State> {
        let dirent = &**dir_info;
        let (name, size, percent) = match dirent {
            crate::models::data_models::Dirent::AGGREGATE(mutex) => {
                let agg = mutex.lock().unwrap();
                (
                    format!("*.{}", agg.common_name),
                    agg.total_size,
                    agg.percent,
                )
            }
            crate::models::data_models::Dirent::VALUE(mutex) => {
                let dirent = mutex.lock().unwrap();
                (format!("{}", dirent.name), dirent.size, dirent.percent)
            }
        };
        let mut next_states;
        match dirent {
            Dirent::AGGREGATE(mutex) => {
                let agg = mutex.lock().unwrap();
                if agg.expanded {
                    let name_clone = Arc::new(agg.common_name.clone());
                    next_states = vec![
                        State::VALUE(Item::STRING(format!("Close X"))),
                        State::LIST(vec![
                            State::VALUE(Item::STRING(name)),
                            State::VALUE(Item::STRING(format!("{}", total_size_to_string(size)))),
                            State::VALUE(Item::STRING(format!("{}%", percent))),
                            State::VALUE(Item::SORT(SortButton {
                                name: format!("Sort Name"),
                                context: name_clone.clone(),
                            })),
                            State::VALUE(Item::SORT(SortButton {
                                name: format!("Sort Size"),
                                context: name_clone,
                            })),
                        ]),
                    ];
                    let mut storage_val = vec![];
                    // adding the children of the aggregator to the list
                    for dir_entry in &agg.dirents {
                        storage_val.push(State::VALUE(Item::DIRECTORY(Arc::new(Dirent::VALUE(
                            dir_entry.clone(),
                        )))));
                    }
                    next_states.push(State::LIST(storage_val));
                    next_states.push(State::VALUE(Item::STRING(format!(""))));
                } else {
                    // this state will go to a StorageWin
                    next_states = vec![
                        State::VALUE(Item::STRING(name.clone())),
                        State::VALUE(Item::STRING(format!("{}", total_size_to_string(size)))),
                        State::VALUE(Item::STRING(format!("{}%", percent))),
                        State::VALUE(Item::NUM(percent / 100.)),
                    ];
                }
            }
            Dirent::VALUE(_) => {
                // this state will go to a StorageWin
                next_states = vec![
                    State::VALUE(Item::STRING(name.clone())),
                    State::VALUE(Item::STRING(format!("{}", total_size_to_string(size)))),
                    State::VALUE(Item::STRING(format!("{}%", percent))),
                    State::VALUE(Item::NUM(percent / 100.)),
                ];
            }
        }
        next_states
    }

    /**
     * Checks which Window was clicked and calls its respective event handler
     * If that event handler returns true that means this event has been consumed, hence mark the id of event to -1
     */
    fn check_mouse_event(
        &mut self,
        t: &mut State,
        mut event: &mut MEVENT,
        tx_frontend: Arc<Sender<Message>>,
    ) -> Result<(), String> {
        if event.id < 0 {
            return Ok(());
        }
        match t {
            State::LIST(list) => (*self.get_children())
                .iter_mut()
                .zip(list.iter_mut())
                .try_for_each(|(win, win_state)| {
                    win.check_mouse_event(win_state, &mut event, tx_frontend.clone())
                })?,
            State::VALUE(item) => match item {
                Item::DIRECTORY(dir_info) => {
                    let mut next_states = self.calculate_next_states(dir_info);
                    self.get_children()
                        .iter_mut()
                        .zip(next_states.iter_mut())
                        .try_for_each(|(win, state)| {
                            win.check_mouse_event(state, event, tx_frontend.clone())
                        })
                        .map_err(|e| e.to_string())?;
                    // LOG!(format!("dirent {}", event.id));
                }
                _ => {
                    // LOG!(format!("Sort {} {}", sort_button.context, event.id));
                }
            },
        }
        if event.id < 0 {
            return Ok(());
        }
        let dimension = self.get_dim();
        let startx = dimension.startx;
        let starty = dimension.starty;
        let height = dimension.height + starty;
        let width = dimension.width + startx;
        if self.get_visited()
            && event.x >= startx
            && event.y >= starty
            && event.x <= width
            && event.y <= height
        {
            let mut res: bool = false;
            // numbers decided by multiple loggings
            if event.bstate == BUTTON1_PRESSED as u32 {
                res = self.left_click(t, tx_frontend)?
                // left mouse clicked
            } else if event.bstate == BUTTON3_PRESSED as u32 {
                // right click
                res = self.right_click(t, tx_frontend)?
            } else if event.bstate == BUTTON4_PRESSED as u32 {
                // scroll up
                res = self.scroll_up(t, tx_frontend)?
            } else if event.bstate == BUTTON5_PRESSED as u32 {
                // scroll down
                res = self.scroll_down(t, tx_frontend)?
            }
            if res {
                event.id = -1;
            }
        }
        Ok(())
    }

    /**
    apply the styles_before_populate
     Populate the windows recursively using the state.
     apply the styles_after_populate
     state is assumed to be of same depth as of children
    */
    fn populate(&mut self, state: &State) -> Result<(), NulError> {
        // applying the styles
        if self.get_style_on() {
            let window = self.get_win();

            let pad = self.get_pad();
            let styles = self.get_style_before_populate();
            match window {
                Some(window) => {
                    apply_stylying(window, self.get_dim_unmut(), styles);
                }
                None => {}
            }
            match pad {
                Some(val) => {
                    apply_stylying(val, self.get_dim_unmut(), styles);
                }
                None => {}
            }
        }
        // populating the content to the windows
        match state {
            State::LIST(list) => (*self.get_children())
                .iter_mut()
                .zip(list.iter())
                .try_for_each(|(win, win_state)| win.populate(win_state))?,
            State::VALUE(val) => {
                self.display_state(val)?;
            }
        };

        // applying the styles
        if self.get_style_on() {
            let window = self.get_win();
            let pad = self.get_pad();
            let styles = self.get_style_after_populate();
            let title = self.get_title();
            // let dimension = self.get_dim_unmut();
            match window {
                Some(window) => {
                    match title {
                        Some(val) => {
                            if val.len() != 0 {
                                mvwprintw(window, 0, 1, &format!("{}", val))?;
                            }
                        }
                        None => {}
                    }
                    apply_stylying(window, self.get_dim_unmut(), styles);
                }
                None => {}
            }
            match pad {
                Some(val) => {
                    apply_stylying(val, self.get_dim_unmut(), styles);
                }
                None => {}
            }
        }
        Ok(())
    }

    /**
        This calculates the height and width and re/initializes ncurse: WINDOWs and PADs as required
    */
    fn re_initialize_win(
        &mut self,
        parent: Dimension,
        mut cumulative_startx: i32,
        mut cumulative_starty: i32,
    ) -> (i32, i32) {
        (cumulative_startx, cumulative_starty) =
            self.extract_dimensions(parent, cumulative_startx, cumulative_starty);
        let dimension = self.get_dim();
        let height = dimension.height;
        let width = dimension.width;
        let startx = dimension.startx;
        let starty = dimension.starty;
        self.clear_for_resize();

        // Now we can safely mutate `self` after we've finished getting the dimension

        if self.create_win() {
            let win = newwin(height, width, starty, startx);
            nodelay(win, true);
            keypad(win, true);
            wbkgd(win, COLOR_PAIR(PAIR_WHITE_BLACK));
            self.set_win(win);
        }
        // LOG!("Here");
        if self.get_children().is_empty() {
            let pad = newpad(height + 1, width + 1);
            keypad(pad, true);
            nodelay(pad, true);
            wbkgd(pad, COLOR_PAIR(PAIR_WHITE_BLACK));
            self.set_pad(pad);
        }
        (cumulative_startx, cumulative_starty)
    }

    /**
        goes to each children recursively and refresh them to their respective screen position
        also taking care of the elements which overflows
    */
    fn display_content(
        &mut self,
        parent: Option<Dimension>,
        mut cumulative_startx: i32,
        mut cumulative_starty: i32,
        parent_scrolly: i32,
    ) -> Result<(i32, i32), NulError> {
        let mut pdimension: Option<Dimension> = None;
        match parent {
            Some(p_dim) => {
                (cumulative_startx, cumulative_starty) =
                    self.re_initialize_win(p_dim, cumulative_startx, cumulative_starty);

                pdimension = Some(*self.get_dim());
            }
            None => {
                self.set_visited(true);
            }
        }
        let window = self.get_win();

        let pad = self.get_pad();
        let styles = self.get_style_before_populate();
        // let dimension = self.get_dim_unmut();
        match window {
            Some(window) => {
                wrefresh(window);
            }
            None => {}
        }

        let dimension = self.get_dim_unmut();
        match pad {
            Some(val) => {
                let win_height = match dimension.display_height {
                    DimensionType::PERCEN(_) => dimension.height,
                    DimensionType::DIMENS(val) => {
                        if val < 0 {
                            dimension.height - 2
                        } else {
                            dimension.height
                        }
                    }
                };
                let win_width = match dimension.display_width {
                    DimensionType::PERCEN(_) => dimension.width,
                    DimensionType::DIMENS(val) => {
                        if val < 0 {
                            dimension.width - 2
                        } else {
                            dimension.width
                        }
                    }
                };
                apply_stylying(val, dimension, styles);
                prefresh(
                    val,
                    0,
                    0,
                    dimension.starty + 1,
                    dimension.startx + 1,
                    dimension.starty + win_height,
                    dimension.startx + win_width,
                );
            }
            None => {}
        }

        let mut cumx = 0;
        let mut cumy = 0;
        let scrollx = dimension.scrollx;
        let scrolly = dimension.scrolly;
        // reprint the values from the datastructure to the pad
        self.get_children()
            .iter_mut()
            .skip(scrollx as usize)
            .skip(scrolly as usize)
            .try_for_each(|child| {
                (cumx, cumy) = child.display_content(pdimension, cumx, cumy, 0)?;
                Ok(())
            })?;

        // marking all elements before scrolly as not visited that means there event handler must not be called as they are not visible
        self.get_children()
            .iter_mut()
            .take(scrolly as usize)
            .try_for_each(|child| {
                child.set_visited_all(false);
                Ok(())
            })?;
        Ok((cumulative_startx, cumulative_starty))
    }

        // mvwprintw(window, 0, 0, &format!("{} {}\n", win_height, win_width))?;
    /**
        clears the windows recursively
    */
    fn clear_win(&mut self) -> Result<(), NulError> {
        let window = self.get_win();

        let pad = self.get_pad();
        match window {
            Some(window) => {
                wclear(window);
                wmove(window, 0, 0);
                wrefresh(window);
            }
            None => {}
        }
        match pad {
            Some(val) => {
                wclear(val);
                wmove(val, 0, 0);
                wrefresh(val);
            }
            None => {}
        }
        // reprint the values from the datastructure to the pad
        self.get_children().iter_mut().try_for_each(|child| {
            child.clear_win()?;
            Ok(())
        })?;
        Ok(())
    }

    /**
     * Calculates and updates self.dim
     * This is supposed to be called while re-initializing the screen
     * different values of height,width and startx defined where will be the window place
     */
    fn extract_dimensions(
        &mut self,
        parent: Dimension,
        mut cumulative_startx: i32,
        mut cumulative_starty: i32,
    ) -> (i32, i32) {
        let dimension = self.get_dim();
        let mut win_height = match dimension.display_height {
            DimensionType::PERCEN(val) => (((parent.height - 4) as f32) * val).floor() as i32,
            DimensionType::DIMENS(val) => val,
        };
        let mut win_width = match dimension.display_width {
            DimensionType::PERCEN(val) => (((parent.width - 4) as f32) * val).floor() as i32,
            DimensionType::DIMENS(val) => val,
        };
        let mut initial_startx = dimension.initial_startx;
        let mut initial_starty = dimension.initial_starty;
        if initial_startx < 0 || initial_starty == -2 {
            initial_startx = cumulative_startx;
        }
        if initial_starty < 0 || initial_startx == -2 {
            initial_starty = cumulative_starty;
        }

        if win_width < 0 {
            win_width = parent.width - initial_startx;
        } else if win_width > parent.width {
            win_width = parent.width - 1;
        }
        if win_height < 0 {
            win_height = parent.height - initial_starty;
        } else if win_height > parent.height {
            win_height = parent.height - 1;
        }

        if win_height + initial_starty >= parent.height - 1 {
            if dimension.initial_starty == -2 {
                cumulative_startx += win_width;
                cumulative_starty = 0;
                initial_startx = cumulative_startx;
                initial_starty = cumulative_starty;
            } else {
                win_height = parent.height - initial_starty
            }
        }
        if win_width + initial_startx >= parent.width - 1 {
            if dimension.initial_startx == -2 {
                cumulative_starty += win_height;
                cumulative_startx = 0;
                initial_starty = cumulative_starty;
                initial_startx = cumulative_startx;
            } else {
                win_width = parent.width - initial_startx
            }
        }

        dimension.height = win_height;
        dimension.width = win_width;

        // NOTE: Note best way wey must either substract 2 if parent have window else not but that info need to be passed all the way from top to here
        // this 4 is just hardcoded
        if (initial_startx >= parent.width - 2 && parent.width > 4)
            || (initial_starty >= parent.height - 2 && parent.height > 4)
        {
            // exeeds the screen so do not render this
            let x = getmaxx(stdscr());
            initial_startx = x;
        }

        dimension.startx = initial_startx + parent.startx;
        dimension.starty = initial_starty + parent.starty;


        if dimension.initial_startx < 0 {
            cumulative_startx += win_width;
        }
        if dimension.initial_starty < 0 {
            cumulative_starty += win_height;
        }
        (cumulative_startx, cumulative_starty)
    }
}

#[derive(Clone, Copy)]
pub enum DimensionType {
    PERCEN(f32),
    DIMENS(i32),
}

#[derive(Clone)]
pub enum State {
    LIST(Vec<State>),
    VALUE(Item), // to be replaced by enum which contains all the possible structs, Window expect to display
}

#[derive(Clone)]
pub enum Item {
    STRING(String),
    DIRECTORY(Arc<Dirent>),
    SORT(SortButton),
    NUM(f32),
    INFO((String, String)),
}

#[derive(Clone)]
pub struct SortButton {
    pub name: String,
    pub context: Arc<String>,
}

#[derive(Clone, Debug)]
pub struct DirInfo {
    path: Arc<String>,
}

#[derive(Clone, Debug)]
pub struct Message {
    pub mtype: MessageType,
    pub content: Option<Arc<String>>,
}

pub struct Selected {
    pub win_type: WinType,
    pub index: Vec<i32>,
}

#[derive(Clone, Copy, Debug)]
pub enum MessageType {
    READDIR,
    GOBACK,
    SORTBYNAME,
    SORTBYSIZE,
    RELOADSTORAGE,
    RELOADFILE,
    RELOADFOLDER,
    RELOAD,
    CHANGEFILEINFO,
}

#[derive(Clone, Copy, Eq, Hash, PartialEq, Debug)]
pub enum WinType {
    FOLDERWIN,
    FILEWIN,
    STORAGEWIN,
}

pub enum STYLETYPE {
    LEFTBORDER,
    TOPBORDER,
    BOTTOMBORDER,
    RIGHTBORDER,
    STARTCOLOR,
    REMOVECOLOR,
    FULLBORDER,
    SPECIALCHARS,
}
