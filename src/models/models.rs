use std::{
    collections::HashMap,
    ffi::NulError,
    sync::Arc,
};

use ncurses::{box_, clear, getmaxyx, ll::WINDOW, mvwprintw, newpad, newwin, prefresh, refresh, stdscr, wprintw, wrefresh};

pub struct Screen {
    windows: HashMap<Arc<String>, Box<dyn DisplayContent>>,
    current_dir: Arc<String>,
    pub dim: Dimension,
}

impl Screen {
    pub fn new() -> Screen {
        Screen {
            windows: HashMap::new(),
            current_dir: Arc::new("/".to_string()),
            dim: Dimension {
                height: 0,
                width: 0,
                startx: 0,
                starty: 0,
                initial_startx: -1,
                initial_starty: -1,
                display_height: DimensionType::DIMENS(-1),
                display_width: DimensionType::DIMENS(-1),
            },
        }
    }
    pub fn add_window(&mut self, name: Arc<String>, win: Box<dyn DisplayContent>) {
        self.windows.insert(name, win);
    }

    pub fn update_height(&mut self) {
        getmaxyx(stdscr(), &mut self.dim.height, &mut self.dim.width);
    }

    pub fn get_window(&mut self, name: Arc<String>) -> &mut Box<dyn DisplayContent> {
        self.windows
            .get_mut(&name)
            .expect(&format!("Key {name} not found"))
    }

    pub fn refresh_screen(&mut self, re_size : bool) -> Result<(), NulError> {
        clear();
        refresh();
        let mut pdimension = None;
        if re_size {
            pdimension = Some(self.dim);
        }
        let mut cumx = 0;  
        let mut cumy = 0;  
        self.windows
            .iter_mut()
            .try_for_each(|(_, win)| {(cumx, cumy) = win.display_content(pdimension, cumx, cumy)?; Ok(()) })?;
        Ok(())
    }

    pub fn populate(&mut self, content: std::sync::RwLockReadGuard<'_, HashMap<Arc<String>, State>>) -> Result<() , NulError> {
        self.windows
            .iter_mut()
            .try_for_each(|(name, win)| {
                let win_content = content.get(name);
                match win_content {
                    Some(val) =>
                    win.populate(val),
                    None => Ok(())
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
    pub initial_startx: i32,
    pub initial_starty: i32,
    pub display_height: DimensionType,
    pub display_width: DimensionType,
}

pub trait DisplayContent {
    fn get_win(&self) -> Option<WINDOW>;
    fn get_pad(&self) -> Option<WINDOW>;
    fn set_win(&mut self, window: WINDOW);
    fn set_pad(&mut self, window: WINDOW);
    fn clear_for_resize(&mut self);
    fn get_title(&self) -> Option<String>;
    fn with_border(&self) -> bool;
    fn get_children(&mut self) -> &mut Vec<Box<dyn DisplayContent>>;
    fn add_child(&mut self, win: Box<dyn DisplayContent>);
    /**
     * Returns Dimension
     */
    fn get_dim(&mut self) -> &mut Dimension;

    /**
     * Return NulError if the object does not expect to print anything
     */
    fn display_state(&mut self, state: &Item) -> Result<(), NulError>;

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

    fn re_initialize_win(&mut self, parent: Dimension, mut cumulative_startx: i32, mut cumulative_starty: i32) -> (i32, i32) {
        (cumulative_startx,cumulative_starty) = self.extract_dimensions(parent, cumulative_startx,  cumulative_starty);
        let dimension = self.get_dim();
        let height = dimension.height;
        let width = dimension.width;
        let startx = dimension.startx;
        let starty = dimension.starty; 
        self.clear_for_resize();

        // Now we can safely mutate `self` after we've finished getting the dimension
        
        if self.get_children().is_empty() {
            self.set_pad(newpad(height, width));
        } 
        if self.with_border() {
            let win = newwin(
                height,
                width,
                starty,
                startx,
            );
            self.set_win(win);
        }
        (cumulative_startx, cumulative_starty)

    }

    fn display_content(&mut self, parent: Option<Dimension>,  mut cumulative_startx: i32, mut cumulative_starty: i32) -> Result<(i32, i32), NulError> {
        let mut pdimension : Option<Dimension> = None;
        // let mut height = 6;
        match parent  {
            Some(p_dim) => {
                // height = p_dim.height;
                (cumulative_startx, cumulative_starty) = self.re_initialize_win(p_dim, cumulative_startx, cumulative_starty);

                pdimension = Some(*self.get_dim());
                // let pad = self.get_pad();
                // match pad {
                //     Some(val) => 
                //     _ = wprintw(val, &format!("_ {} {} {} {}", p_dim.startx, p_dim.starty, p_dim.height, p_dim.width))?,
                //     None => {}
                // };
            },
            None => {},
        }
        let window = self.get_win();

        let pad = self.get_pad();
        // let is_pad = match pad {
        //     Some(val) => true,
        //     None => false,
        // };
        let title = self.get_title();
        // let is_last = self.get_children().is_empty();
        let dimension = self.get_dim();
        match window {
            Some(window) => {
                match title {
                    Some(val) => {
                        if val.len() != 0 {
                            box_(window, 0, 0);
                            mvwprintw(window, 0, 1, &format!("{}_ {} {} {} {}",val, dimension.startx, dimension.starty, dimension.height, dimension.width))?;
                            mvwprintw(window, 0, 1, &format!("{}",val))?;
                        }
                    }
                    None => {
                        // box_(window, 0, 0);
                        // mvwprintw(window, 0, 1, &format!("__ {} {} {} {} {}", dimension.startx, dimension.starty, dimension.height, dimension.width, cumulative_starty))?;
                    }
                }
                wrefresh(window);
            },
            None => {

            },
        }

        // mvwprintw(window, 0, 0, &format!("{} {}\n", win_height, win_width))?;
        match pad {
            Some(val) => {
                let win_height = match dimension.display_height {
                    DimensionType::PERCEN(_) => {dimension.height},
                    DimensionType::DIMENS(val) => {
                        if val < 0 {
                            dimension.height - 2                        
                        } else {
                            dimension.height
                        }
                    },
                };
                let win_width = match dimension.display_width {
                    DimensionType::PERCEN(_) => {dimension.width},
                    DimensionType::DIMENS(val) => {
                        if val < 0 {
                            dimension.width - 2                         
                        } else {
                            dimension.width
                        }
                    },
                };
                // box_(val, 0 ,0);
                // wprintw(val, &format!("__ {} {} {} {} {} {}", dimension.starty + dimension.height, dimension.initial_starty, dimension.starty + dimension.height >= 0, dimension.width, cumulative_starty, cumulative_startx));
                prefresh(
                    val,
                    0,
                    0,
                    dimension.starty + 1,
                    dimension.startx + 1,
                    dimension.starty + win_height,
                    dimension.startx + win_width
                );
            },
            None => {}
        }

        let mut cumx = 0;
        let mut cumy = 0;
        // reprint the values from the datastructure to the pad
        self.get_children()
        .iter_mut()
        .try_for_each(|child| {(cumx, cumy) = child.display_content(pdimension,cumx, cumy)?; Ok(())})?;
        Ok((cumulative_startx, cumulative_starty))
    }

    /**
     * Calculates and updates self.dim
     * This is supposed to be called while re-initializing the screen
     */
    fn extract_dimensions(&mut self, parent: Dimension, mut cumulative_startx: i32, mut cumulative_starty: i32) -> (i32, i32){
        let dimension = self.get_dim();
        let mut win_height = match dimension.display_height {
            DimensionType::PERCEN(val) => (((parent.height - 4) as f32) * val).floor() as i32,
            DimensionType::DIMENS(val) => val,
        };
        let mut win_width = match dimension.display_width {
            DimensionType::PERCEN(val) => (((parent.width - 4) as f32) * val).floor() as i32,
            DimensionType::DIMENS(val) => val,
        };
        let mut initial_startx = dimension.initial_startx ;
        let mut initial_starty = dimension.initial_starty ;
        if initial_startx < 0 || initial_starty == -2{
            initial_startx = cumulative_startx;
        }
        if initial_starty < 0 || initial_startx == -2{
            initial_starty = cumulative_starty;
        }
        
        if win_width < 0 {
            win_width = parent.width - initial_startx;
        }
        if win_height < 0 {
            win_height = parent.height - initial_starty;
        }
        dimension.height = win_height;
        dimension.width = win_width;
        
        if dimension.initial_starty == -2 && win_height + parent.starty + initial_starty >= parent.height {
            cumulative_startx += win_width;
            cumulative_starty = 0;
            initial_startx = cumulative_startx;
            initial_starty = cumulative_starty;
        }
        if dimension.initial_startx == -2 && win_width + parent.startx + initial_startx >= parent.width{
            cumulative_starty += win_height;
            cumulative_startx = 0;
            initial_starty = cumulative_starty;
            initial_startx = cumulative_startx;
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
    DIRECTORY(DirInfo),
}

#[derive(Clone)]
pub struct DirInfo {
    path: Arc<String>,
}

#[derive(Clone)]
pub struct Message {
    pub mtype: MessageType,
    pub content: Arc<String>
}

#[derive(Clone, Copy)]
pub enum MessageType {
    READDIR
}