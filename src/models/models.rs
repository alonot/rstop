use std::{
    collections::HashMap,
    ffi::NulError,
    fs,
    hash::Hash,
    sync::{mpsc::Sender, Arc},
};

use ncurses::{
    attr_t, box_, clear, doupdate, getmaxx, getmaxyx, getyx, keypad, ll::WINDOW, mvwhline, mvwprintw, mvwvline, newpad, newwin, nodelay, prefresh, refresh, stdscr, waddch, wclear, wmove, wprintw, wrefresh, ACS_DARROW, ACS_HLINE, ACS_VLINE, BUTTON1_PRESSED, BUTTON2_PRESSED, BUTTON3_PRESSED, BUTTON4_PRESSED, BUTTON5_PRESSED, LINES, MEVENT
};

use crate::{total_size_to_string, LOG};

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

    pub fn checkMouseClick(
        &mut self,
        mut content: std::sync::RwLockWriteGuard<'_, HashMap<WinType, State>>,
        event: &mut MEVENT,
        tx_frontend: Arc<Sender<Message>>,
    ) -> Result<(), String> {
        self.windows.iter_mut().try_for_each(|(name, win)| {
            let win_content = content.get_mut(name);
            match win_content {
                Some(val) => win.checkMouseEvent(val, event, tx_frontend.clone()),
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
            STYLETYPE::INITCOLOR => {}
            STYLETYPE::REMOVECOLOR => {}
            STYLETYPE::FULLBORDER => {
                box_(win, 0, 0);
            }
            STYLETYPE::SPECIALCHARS => {
                waddch(win, *attr);
                getyx(win, &mut y, &mut x);
            },
        }
    }
    wmove(win, y, x); // to again move the print cursor where it was
}

pub trait DisplayContent {
    fn get_win(&self) -> Option<WINDOW>;
    fn get_pad(&self) -> Option<WINDOW>;
    fn set_win(&mut self, window: WINDOW);
    fn set_visited(&mut self, _: bool);
    fn get_visited(&mut self) -> bool;
    fn set_pad(&mut self, window: WINDOW);
    fn clear_for_resize(&mut self);
    fn get_title(&self) -> Option<String>;
    fn create_win(&self) -> bool;
    fn get_children(&mut self) -> &mut Vec<Box<dyn DisplayContent>>;
    fn add_child(&mut self, win: Box<dyn DisplayContent>);
    /**
     * returns styling to be applied in a specific order
     */
    fn get_style(&self) -> &Vec<(STYLETYPE, attr_t)>;
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

    fn set_visited_all(&mut self,
        val: bool) {
        self.set_visited(val);
        self.get_children().iter_mut().for_each(|child| {
            child.set_visited_all(val);
        });
    }


    fn calculate_next_states(&mut self,dir_info: &Arc<Dirent>) -> Vec<State> {
        // LOG!(format!("dirent"));
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
        let mut next_states ;
        match dirent {
            Dirent::AGGREGATE(mutex) => {
                let agg = mutex.lock().unwrap();
            // // LOG!(format!("Expanded: {} {}", agg.expanded, agg.common_name));
                if agg.expanded {
                    let name_clone = Arc::new(agg.common_name.clone());
                  // LOG!(format!("Len: {}", agg.dirents.len()));
                    next_states = vec![
                        State::VALUE(Item::STRING(format!("Close X"))),
                        State::LIST(vec![
                            State::VALUE(Item::STRING(name)),
                            State::VALUE(Item::STRING(format!(
                                "{}",
                                total_size_to_string(size)
                            ))),
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
                    for dir_entry in &agg.dirents {
                        storage_val.push(State::VALUE(Item::DIRECTORY(Arc::new(
                            Dirent::VALUE(dir_entry.clone()),
                        ))));
                    }
                    next_states.push(State::LIST(storage_val));
                    next_states.push(State::VALUE(Item::STRING(format!(""))));
                } else {
                    next_states = vec![
                        State::VALUE(Item::STRING(name.clone())),
                        State::VALUE(Item::STRING(format!(
                            "{}",
                            total_size_to_string(size)
                        ))),
                        State::VALUE(Item::STRING(format!("{}%", percent))),
                    ];
                }
            }
            Dirent::VALUE(_) => {
                next_states = vec![
                    State::VALUE(Item::STRING(name.clone())),
                    State::VALUE(Item::STRING(format!(
                        "{}",
                        total_size_to_string(size)
                    ))),
                    State::VALUE(Item::STRING(format!("{}%", percent))),
                ];
            }
        }
        next_states
    }

    fn checkMouseEvent(
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
                    win.checkMouseEvent(win_state, &mut event, tx_frontend.clone())
                })?,
            State::VALUE(item) => match item {
                Item::STRING(val) => {
                    // LOG!(format!("val {}", val));
                }
                Item::DIRECTORY(dir_info) => {
                    let mut next_states = self.calculate_next_states(dir_info);
                    self.get_children()
                    .iter_mut()
                    .zip(next_states.iter_mut())
                    .try_for_each(|(win, state)| {
                        win.checkMouseEvent(state, event, tx_frontend.clone())
                    })
                    .map_err(|e| e.to_string())?;
                    // LOG!(format!("dirent {}", event.id));
                }
                Item::SORT(_) => {
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
        if event.x >= startx && event.y >= starty && event.x <= width && event.y <= height {
            let mut res: bool = false;
            // LOG!(format!(
            //     "5:{} {} {} {}",
            //     event.bstate & BUTTON5_PRESSED as u32,
            //     event.id,
            //     starty,
            //     height
            // ));
            // numbers decided by multiple loggings
            if event.bstate & BUTTON1_PRESSED as u32 == 2 {
                res = self.left_click(t, tx_frontend)?
                // left mouse clicked
            } else if event.bstate & BUTTON3_PRESSED as u32 == 2048 {
                // right click
                res = self.right_click(t, tx_frontend)?
            } else if event.bstate & BUTTON5_PRESSED as u32 == 0 {
                // scroll up
                res = self.scroll_up(t, tx_frontend)?
            } else if event.bstate & BUTTON4_PRESSED as u32 == 0 {
                // scroll down
                res = self.scroll_down(t, tx_frontend)?
            }
            if res {
                event.id = -1;
            }
            // LOG!(format!(
            //     "{}",
            //     event.id,
            // ));
        }
        Ok(())
    }

    //////     DEFAULT        ////////////
    fn populate(&mut self, state: &State) -> Result<(), NulError> {
        match state {
            State::LIST(list) => (*self.get_children())
                .iter_mut()
                .zip(list.iter())
                .try_for_each(|(win, win_state)| win.populate(win_state))?,
            State::VALUE(val) => {
                self.display_state(val)?;
            }
        }
        Ok(())
    }

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
            let win = newwin(height, width , starty, startx);
            nodelay(win, true);
            keypad(win, true);
            self.set_win(win);
        }
        // LOG!("Here");
        if self.get_children().is_empty() {

            let pad = newpad(height + 1, width + 1);
            keypad(pad, true);
            nodelay(pad, true);
            self.set_pad(pad);
        }
        (cumulative_startx, cumulative_starty)
    }

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
        let title = self.get_title();
        let styles = self.get_style();
        // let dimension = self.get_dim_unmut();
        match window {
            Some(window) => {
                apply_stylying(window, self.get_dim_unmut(), styles);
                match title {
                    Some(val) => {
                        if val.len() != 0 {
                            // mvwprintw(window, 0, 1, &format!("{}_ {} {} {} {}",val, dimension.startx, dimension.starty, dimension.height, dimension.width))?;
                            mvwprintw(window, 0, 1, &format!("{}", val))?;
                        }
                    }
                    None => {
                        // box_(window, 0, 0);
                        // mvwprintw(window, 0, 1, &format!("__ {} {} {} {} {}", dimension.startx, dimension.starty, dimension.height, dimension.width, cumulative_starty))?;
                    }
                }
                wrefresh(window);
            }
            None => {}
        }

        let dimension = self.get_dim_unmut();
        // mvwprintw(window, 0, 0, &format!("{} {}\n", win_height, win_width))?;
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
                // box_(val, 0 ,0);
                // wprintw(val, &format!("__ {} {} {} {} {} {}", dimension.starty + dimension.height, dimension.starty >= 37, dimension.starty , dimension.width, cumulative_starty, cumulative_startx));
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
        let width = dimension.width;
        // reprint the values from the datastructure to the pad
        self.get_children().iter_mut().skip(scrollx as usize). skip(scrolly as usize).try_for_each(|child| {
            (cumx, cumy) = child.display_content(pdimension, cumx, cumy, 0)?;
            Ok(())
        })?;
        self.get_children().iter_mut().take(scrolly as usize).try_for_each(|child| {
            child.set_visited_all(false);
            Ok(())
        })?;
        Ok((cumulative_startx, cumulative_starty))
    }

    fn clear_win(
        &mut self,
    ) -> Result<(), NulError> {
        let window = self.get_win();

        let pad = self.get_pad();
        let title = self.get_title();
        let styles = self.get_style();
        // let dimension = self.get_dim_unmut();
        match window {
            Some(window) => {
                wclear(window);
                wmove(window, 0, 0);
                wrefresh(window);
            }
            None => {}
        }

        // mvwprintw(window, 0, 0, &format!("{} {}\n", win_height, win_width))?;
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
        } else  if win_width > parent.width {
            win_width = parent.width - 1;
        }
        if win_height < 0 {
            win_height = parent.height - initial_starty;
        } else if win_height > parent.height {
            win_height = parent.height - 1;
        }

        if win_height + initial_starty >= parent.height - 1 {
            if dimension.initial_starty == -2  {

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
        if (initial_startx >= parent.width - 2 && parent.width > 4)  || (initial_starty >= parent.height - 2 && parent.height > 4) {
            // exeeds the screen so do not render this
            let x = getmaxx(stdscr());
            initial_startx = x;
        }

        dimension.startx = initial_startx + parent.startx;
        dimension.starty = initial_starty + parent.starty;

        // dimension.scrolly = dimension.starty;
        // LOG!(format!("{}", dimension.starty));

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

#[derive(Clone, Copy, Debug)]
pub enum MessageType {
    READDIR,
    GOBACK,
    SORTBYNAME,
    SORTBYSIZE,
    RELOADSTORAGE,
    RELOAD,
    CHANGEFILEINFO
}

#[derive(Eq, Hash, PartialEq, Debug)]
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
    INITCOLOR,
    REMOVECOLOR,
    FULLBORDER,
    SPECIALCHARS
}
