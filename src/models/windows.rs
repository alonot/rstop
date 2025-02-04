use ncurses::{attr_t, wprintw, ITEM, WINDOW};
use std::fs;
use std::{
    any::{self, Any},
    collections::HashMap,
    ffi::{CString, NulError},
    sync::Arc,
};

use crate::{total_size_to_string, LOG};

use super::data_models::Dirent;
use super::models::{Dimension, DimensionType, DisplayContent, Item, State, STYLETYPE};

macro_rules! implement_getters_setters {
    () => {
        fn get_win(&self) -> Option<WINDOW> {
            return self.win;
        }

        fn get_pad(&self) -> Option<WINDOW> {
            return self.pad;
        }

        fn set_win(&mut self, window: WINDOW) {
            self.win = Some(window);
        }

        fn set_pad(&mut self, window: WINDOW) {
            self.pad = Some(window);
        }

        fn clear_for_resize(&mut self) {
            self.pad = None;
            self.win = None;
        }

        fn get_title(&self) -> Option<String> {
            return self.title.clone();
        }

        fn get_children(&mut self) -> &mut Vec<Box<dyn DisplayContent>> {
            &mut self.children
        }

        fn get_dim(&mut self) -> &mut Dimension {
            return &mut self.dimension;
        }
        fn get_dim_unmut(&self) -> &Dimension {
            return &self.dimension;
        }

        fn add_child(&mut self, win: Box<dyn DisplayContent>) {
            self.children.push(win);
        }
        fn get_style(&self) -> &Vec<(STYLETYPE, attr_t)> {
            &self.styles
        }
        fn create_win(&self) -> bool {
            self.create_win
        }
    };
}

macro_rules! implement_listeners {
    () => {
        fn left_click(&mut self, t: &mut State) -> Result<bool, String> {
            if let Some(handler) = &self.left_click_handler {
                return handler(t); // Call the function if it exists
            }
            Ok(false)
        }
        fn right_click(&mut self, t: &mut State) -> Result<bool, String> {
            if let Some(handler) = &self.right_click_handler {
                return handler(t); // Call the function if it exists
            }
            Ok(false)
        }
        fn scroll_up(&mut self, t: &mut State) -> Result<bool, String> {
            if let Some(handler) = &self.scroll_up_handler {
                return handler(t); // Call the function if it exists
            }
            Ok(false)
        }
        fn scroll_down(&mut self, t: &mut State) -> Result<bool, String> {
            if let Some(handler) = &self.scroll_down_handler {
                return handler(t); // Call the function if it exists
            }
            Ok(false)
        }
    };
}

pub struct Window {
    win: Option<WINDOW>,
    pad: Option<WINDOW>,
    title: Option<String>,
    children: Vec<Box<dyn DisplayContent>>,
    dimension: Dimension,
    create_win: bool,
    styles: Vec<(STYLETYPE, attr_t)>,
    left_click_handler: Option<Arc<dyn Fn(&mut State) -> Result<bool, String>>>,
    right_click_handler: Option<Arc<dyn Fn(&mut State) -> Result<bool, String>>>,
    scroll_up_handler: Option<Arc<dyn Fn(&mut State) -> Result<bool, String>>>,
    scroll_down_handler: Option<Arc<dyn Fn(&mut State) -> Result<bool, String>>>,
}

impl Window {
    /**
       if display_height = -1 expands to last of the screen
       Similarly for width
    */
    pub fn new(
        title: String,
        startx: i32,
        starty: i32,
        display_height: DimensionType,
        display_width: DimensionType,
        style: Option<&HashMap<String, &dyn Any>>,
        css_styles: Vec<(STYLETYPE, attr_t)>,
    ) -> Window {
        let mut win = Window {
            children: vec![],
            title: Some(title),
            dimension: Dimension {
                height: 0,
                width: 0,
                startx: 0,
                starty: 0,
                initial_startx: startx,
                initial_starty: starty,
                display_height,
                display_width,
            },
            win: None,
            pad: None,
            create_win: false,
            styles: css_styles,
            left_click_handler: None,
            right_click_handler: None,
            scroll_up_handler: None,
            scroll_down_handler: None,
        };
        match style {
            Some(style) => {
                if style.contains_key("with_border") {
                    let with_border: &&dyn Any = style.get("with_border").expect("msg");
                    if with_border.is::<bool>() {
                        win.create_win = *with_border.downcast_ref::<bool>().unwrap();
                    }
                }
                if let Some(left_click_handler) = style.get("left_click") {
                    if let Some(handler) = left_click_handler
                        .downcast_ref::<Arc<dyn Fn(&mut State) -> Result<bool, String>>>()
                    {
                        win.left_click_handler = Some(Arc::clone(handler));
                    }
                }
                if let Some(right_click_handler) = style.get("right_click") {
                    if let Some(handler) = right_click_handler
                        .downcast_ref::<Arc<dyn Fn(&mut State) -> Result<bool, String>>>()
                    {
                        win.right_click_handler = Some(Arc::clone(handler));
                    }
                }
                if let Some(scroll_up_handler) = style.get("scroll_up") {
                    if let Some(handler) = scroll_up_handler
                        .downcast_ref::<Arc<dyn Fn(&mut State) -> Result<bool, String>>>()
                    {
                        win.scroll_up_handler = Some(Arc::clone(handler));
                    }
                }
                if let Some(scroll_down_handler) = style.get("scroll_down") {
                    if let Some(handler) = scroll_down_handler
                        .downcast_ref::<Arc<dyn Fn(&mut State) -> Result<bool, String>>>()
                    {
                        win.scroll_down_handler = Some(Arc::clone(handler));
                    }
                }
            }
            None => {}
        }
        // win.re_initialize_win(height, width);
        win
    }
}

macro_rules! genNulError {
    () => {
        _ = CString::new("\0da")? // always throws NulError
    };
}

impl DisplayContent for Window {
    implement_getters_setters!();

    // implement_listeners!();
    fn left_click(&mut self, t: &mut State) -> Result<bool, String> {
        if let Some(handler) = &self.left_click_handler {
            return handler(t); // Call the function if it exists
        }
        Ok(false)
    }

    fn display_state(&mut self, state: &Item) -> Result<(), NulError> {
        // genNulError!();
        Ok(())
    }
}

pub struct TextBox {
    win: Option<WINDOW>,
    pad: Option<WINDOW>,
    title: Option<String>,
    children: Vec<Box<dyn DisplayContent>>,
    dimension: Dimension,
    create_win: bool,
    styles: Vec<(STYLETYPE, attr_t)>,
    left_click_handler: Option<Arc<dyn Fn(&mut State) -> Result<bool, String>>>,
    right_click_handler: Option<Arc<dyn Fn(&mut State) -> Result<bool, String>>>,
    scroll_up_handler: Option<Arc<dyn Fn(&mut State) -> Result<bool, String>>>,
    scroll_down_handler: Option<Arc<dyn Fn(&mut State) -> Result<bool, String>>>,
}

impl TextBox {
    /**
       if display_height = -1 expands to last of the screen
       Similarly for width
    */
    pub fn new(
        startx: i32,
        starty: i32,
        display_height: DimensionType,
        display_width: DimensionType,
        style: Option<&HashMap<String, &dyn Any>>,
        css_styles: Vec<(STYLETYPE, attr_t)>,
    ) -> TextBox {
        let mut win = TextBox {
            children: vec![],
            title: None,
            dimension: Dimension {
                height: 0,
                width: 0,
                startx: 0,
                starty: 0,
                initial_startx: startx,
                initial_starty: starty,
                display_height,
                display_width,
            },
            win: None,
            pad: None,
            create_win: false,
            styles: css_styles,
            left_click_handler: None,
            right_click_handler: None,
            scroll_up_handler: None,
            scroll_down_handler: None,
        };
        match style {
            Some(style) => {
                if style.contains_key("with_border") {
                    let with_border: &&dyn Any = style.get("with_border").expect("msg");
                    if with_border.is::<bool>() {
                        win.create_win = *with_border.downcast_ref::<bool>().unwrap();
                    }
                }
                if let Some(left_click_handler) = style.get("left_click") {
                    match left_click_handler
                        .downcast_ref::<Arc<dyn Fn(&mut State) -> Result<bool, String>>>()
                    {
                        Some(handler) => {
                            win.left_click_handler = Some(Arc::clone(handler));
                        }
                        None => {}
                    }
                }
                if let Some(right_click_handler) = style.get("right_click") {
                    if let Some(handler) = right_click_handler
                        .downcast_ref::<Arc<dyn Fn(&mut State) -> Result<bool, String>>>()
                    {
                        win.right_click_handler = Some(Arc::clone(handler));
                    }
                }
                if let Some(scroll_up_handler) = style.get("scroll_up") {
                    if let Some(handler) = scroll_up_handler
                        .downcast_ref::<Arc<dyn Fn(&mut State) -> Result<bool, String>>>()
                    {
                        win.scroll_up_handler = Some(Arc::clone(handler));
                    }
                }
                if let Some(scroll_down_handler) = style.get("scroll_down") {
                    if let Some(handler) = scroll_down_handler
                        .downcast_ref::<Arc<dyn Fn(&mut State) -> Result<bool, String>>>()
                    {
                        win.scroll_down_handler = Some(Arc::clone(handler));
                    }
                }
            }
            None => {}
        }
        // win.re_initialize_win(height, width);
        win
    }
}

impl DisplayContent for TextBox {
    implement_getters_setters!();

    implement_listeners!();

    fn display_state(&mut self, state: &Item) -> Result<(), NulError> {
        let value: &String = match state {
            
            Item::DIRECTORY(dir_info) => {
                genNulError!();
                &format!("")
            }
            Item::STRING(val) => val,
            Item::SORT(sort_button) => {
                genNulError!();
                &format!("")
            },
        };
        let pad = self.pad.expect("Empty pad : TextBox");
        wprintw(pad, &value)?;

        Ok(())
    }
}

pub struct FileInfoWin {
    win: Option<WINDOW>,
    pad: Option<WINDOW>,
    title: Option<String>,
    create_win: bool,
    children: Vec<Box<dyn DisplayContent>>,
    dimension: Dimension,
    styles: Vec<(STYLETYPE, attr_t)>,
}

impl FileInfoWin {
    /**
       if display_height = -1 expands to last of the screen
       Similarly for width
    */
    pub fn new(
        startx: i32,
        starty: i32,
        display_height: DimensionType,
        display_width: DimensionType,
        styles: Vec<(STYLETYPE, attr_t)>,
    ) -> FileInfoWin {
        let mut children: Vec<Box<dyn DisplayContent>> = vec![];
        for _ in 0..10 {
            children.push(Box::new(TextBox::new(
                0,
                -2,
                DimensionType::DIMENS(1),
                DimensionType::PERCEN(0.5),
                None,
                vec![],
            )));
        }

        let win = FileInfoWin {
            children,
            title: Some("".to_owned()),
            dimension: Dimension {
                height: 0,
                width: 0,
                startx: 0,
                starty: 0,
                initial_startx: startx,
                initial_starty: starty,
                display_height,
                display_width,
            },
            win: None,
            pad: None,
            create_win: false,
            styles,
        };
        // win.re_initialize_win(height, width);
        win
    }
}

impl DisplayContent for FileInfoWin {
    implement_getters_setters!();

    fn display_state(&mut self, state: &Item) -> Result<(), NulError> {
        let value: &String = match state {
            
            Item::DIRECTORY(dir_info) => {
                genNulError!();
                &format!("")
            }
            Item::STRING(val) => val,
            Item::SORT(sort_button) => {
                genNulError!();
                &format!("")
            },
        };
        let pad = self.pad.expect("Empty pad : TextBox");
        wprintw(pad, &value)?;

        Ok(())
    }
}

pub struct StorageWin {
    win: Option<WINDOW>,
    pad: Option<WINDOW>,
    title: Option<String>,
    create_win: bool,
    children: Vec<Box<dyn DisplayContent>>,
    dimension: Dimension,
    styles: Vec<(STYLETYPE, attr_t)>,
}

impl StorageWin {
    /**
       if display_height = -1 expands to last of the screen
       Similarly for width
    */
    pub fn new(
        startx: i32,
        starty: i32,
        display_height: DimensionType,
        display_width: DimensionType,
        styles: Vec<(STYLETYPE, attr_t)>,
    ) -> StorageWin {
        let mut children: Vec<Box<dyn DisplayContent>> = vec![];
        for _ in 0..3 {
            children.push(Box::new(TextBox::new(
                -2,
                0,
                DimensionType::DIMENS(1),
                DimensionType::PERCEN(0.2),
                None,
                vec![],
            )));
        }

        let win = StorageWin {
            children,
            title: None,
            dimension: Dimension {
                height: 0,
                width: 0,
                startx: 0,
                starty: 0,
                initial_startx: startx,
                initial_starty: starty,
                display_height,
                display_width,
            },
            win: None,
            pad: None,
            create_win: false,
            styles,
        };
        // win.re_initialize_win(height, width);
        win
    }
}

impl DisplayContent for StorageWin {
    implement_getters_setters!();

    fn display_state(&mut self, state: &Item) -> Result<(), NulError> {
        match state {
            
            Item::DIRECTORY(dir_info) => {
                // LOG!("LOGGING: DIRECTORY");
                let dirent = &**dir_info;
                let (name, size,percent) = match dirent {
                    crate::models::data_models::Dirent::AGGREGATE(mutex) => {
                        let agg = mutex.lock().unwrap();
                        (format!("*.{}",agg.common_name), agg.total_size, agg.percent)
                    },
                    crate::models::data_models::Dirent::VALUE(mutex) => {
                        let dirent = mutex.lock().unwrap();
                        (format!("{}",dirent.name), dirent.size, dirent.percent)
                    },
                };
                let next_states = vec![
                    State::VALUE(Item::STRING(name)),
                    State::VALUE(Item::STRING(format!("{}", total_size_to_string(size)))),
                    State::VALUE(Item::STRING(format!("{}%",percent))),
                ];
                self.children.iter_mut().zip(next_states).try_for_each(|(win, state)| {
                    win.populate(&state)
                })?;
            }
            Item::STRING(_) => {},
            Item::SORT(sort_button) => {
                
            },
        };

        Ok(())
    }
}

pub struct HeaderWin {
    win: Option<WINDOW>,
    pad: Option<WINDOW>,
    title: Option<String>,
    create_win: bool,
    children: Vec<Box<dyn DisplayContent>>,
    dimension: Dimension,
    styles: Vec<(STYLETYPE, attr_t)>,
}

impl HeaderWin {
    /**
       if display_height = -1 expands to last of the screen
       Similarly for width
    */
    pub fn new(
        startx: i32,
        starty: i32,
        display_height: DimensionType,
        display_width: DimensionType,
        sort_by_name: Arc<dyn Fn(&mut State) -> Result<bool, String>>,
        sort_by_size: Arc<dyn Fn(&mut State) -> Result<bool, String>>,
        styles: Vec<(STYLETYPE, attr_t)>,
    ) -> HeaderWin {
        let mut children: Vec<Box<dyn DisplayContent>> = vec![];
        for _ in 0..3 {
            children.push(Box::new(TextBox::new(
                -2,
                0,
                DimensionType::DIMENS(1),
                DimensionType::PERCEN(0.2),
                None,
                vec![],
            )));
        }
        let mut style: HashMap<String, &dyn Any> = HashMap::new();
        style.insert(format!("left_click"), &sort_by_name);
        children.push(Box::new(Button::new(
            -2,
            0,
            DimensionType::DIMENS(1),
            DimensionType::PERCEN(0.2),
            Some(&style),
            vec![],
        )));
        style.clear();
        style.insert(format!("left_click"), &sort_by_size);
        children.push(Box::new(Button::new(
            -2,
            0,
            DimensionType::DIMENS(1),
            DimensionType::PERCEN(0.2),
            Some(&style),
            vec![],
        )));

        let win = HeaderWin {
            children,
            title: None,
            dimension: Dimension {
                height: 0,
                width: 0,
                startx: 0,
                starty: 0,
                initial_startx: startx,
                initial_starty: starty,
                display_height,
                display_width,
            },
            win: None,
            pad: None,
            create_win: false,
            styles,
        };
        // win.re_initialize_win(height, width);
        win
    }
}

impl DisplayContent for HeaderWin {
    implement_getters_setters!();

    fn display_state(&mut self, _: &Item) -> Result<(), NulError> {
        Ok(())
    }
}


pub struct Button {
    win: Option<WINDOW>,
    pad: Option<WINDOW>,
    title: Option<String>,
    children: Vec<Box<dyn DisplayContent>>,
    dimension: Dimension,
    create_win: bool,
    styles: Vec<(STYLETYPE, attr_t)>,
    left_click_handler: Option<Arc<dyn Fn(&mut State) -> Result<bool, String>>>,
    right_click_handler: Option<Arc<dyn Fn(&mut State) -> Result<bool, String>>>,
    scroll_up_handler: Option<Arc<dyn Fn(&mut State) -> Result<bool, String>>>,
    scroll_down_handler: Option<Arc<dyn Fn(&mut State) -> Result<bool, String>>>,
}

impl Button {
    /**
       if display_height = -1 expands to last of the screen
       Similarly for width
    */
    pub fn new(
        startx: i32,
        starty: i32,
        display_height: DimensionType,
        display_width: DimensionType,
        style: Option<&HashMap<String, &dyn Any>>,
        css_styles: Vec<(STYLETYPE, attr_t)>,
    ) -> Button {
        let mut win = Button {
            children: vec![],
            title: None,
            dimension: Dimension {
                height: 0,
                width: 0,
                startx: 0,
                starty: 0,
                initial_startx: startx,
                initial_starty: starty,
                display_height,
                display_width,
            },
            win: None,
            pad: None,
            create_win: false,
            styles: css_styles,
            left_click_handler: None,
            right_click_handler: None,
            scroll_up_handler: None,
            scroll_down_handler: None,
        };
        match style {
            Some(style) => {
                if style.contains_key("with_border") {
                    let with_border: &&dyn Any = style.get("with_border").expect("msg");
                    if with_border.is::<bool>() {
                        win.create_win = *with_border.downcast_ref::<bool>().unwrap();
                    }
                }
                if let Some(left_click_handler) = style.get("left_click") {
                    match left_click_handler
                        .downcast_ref::<Arc<dyn Fn(&mut State) -> Result<bool, String>>>()
                    {
                        Some(handler) => {
                            win.left_click_handler = Some(Arc::clone(handler));
                        }
                        None => {}
                    }
                }
                if let Some(right_click_handler) = style.get("right_click") {
                    if let Some(handler) = right_click_handler
                        .downcast_ref::<Arc<dyn Fn(&mut State) -> Result<bool, String>>>()
                    {
                        win.right_click_handler = Some(Arc::clone(handler));
                    }
                }
                if let Some(scroll_up_handler) = style.get("scroll_up") {
                    if let Some(handler) = scroll_up_handler
                        .downcast_ref::<Arc<dyn Fn(&mut State) -> Result<bool, String>>>()
                    {
                        win.scroll_up_handler = Some(Arc::clone(handler));
                    }
                }
                if let Some(scroll_down_handler) = style.get("scroll_down") {
                    if let Some(handler) = scroll_down_handler
                        .downcast_ref::<Arc<dyn Fn(&mut State) -> Result<bool, String>>>()
                    {
                        win.scroll_down_handler = Some(Arc::clone(handler));
                    }
                }
            }
            None => {}
        }
        // win.re_initialize_win(height, width);
        win
    }
}

impl DisplayContent for Button {
    implement_getters_setters!();

    implement_listeners!();

    fn display_state(&mut self, state: &Item) -> Result<(), NulError> {
        let value: &String = match state {
            
            Item::DIRECTORY(dir_info) => {
                genNulError!();
                &format!("")
            }
            Item::STRING(val) => {
                genNulError!();
                &format!("")
            },
            Item::SORT(sort_button) => {
                &sort_button.name
            },
        };
        let pad = self.pad.expect("Empty pad : TextBox");
        wprintw(pad, &value)?;

        Ok(())
    }
}