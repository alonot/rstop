use ncurses::{
    attr_t, attroff, attron, attrset, waddstr, wattroff, wattrset, wbkgd, wprintw, COLOR_PAIR, COLOR_WHITE, WINDOW
};
use ncurses::{ACS_DARROW, ACS_UARROW};
use core::num;
use std::fs;
use std::ops::Deref;
use std::sync::mpsc::Sender;
use std::vec;
use std::{
    any::Any,
    collections::HashMap,
    ffi::{CString, NulError},
    sync::Arc,
};
use util::*;

use crate::models::models::MessageType;
use crate::util::{self, PAIR_WHITE_BLACK};
use crate::LOG;

use super::data_models::Dirent;
use super::models::{
    apply_stylying, Dimension, DimensionType, DisplayContent, Item, Message, State, STYLETYPE,
};

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

        fn set_visited(&mut self, val: bool) {
            self.visited = val;
        }

        fn get_visited(&mut self) -> bool {
            self.visited
        }

        fn set_style_on(&mut self, val: bool) {
            self.style_on = val;
        }

        fn get_style_on(&mut self) -> bool {
            self.style_on
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
        
        fn get_children_unmut(&self) -> &Vec<Box<dyn DisplayContent>> {
            &self.children
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
        fn get_style_before_populate(&self) -> &Vec<(STYLETYPE, attr_t)> {
            &self.styles_before_populate
        }
        fn get_style_after_populate(&self) -> &Vec<(STYLETYPE, attr_t)> {
            &self.styles_after_populate
        }
        fn create_win(&self) -> bool {
            self.create_win
        }
    };
}

macro_rules! implement_listeners {
    () => {
        fn left_click(&mut self, t: &mut State, tx: Arc<Sender<Message>>) -> Result<bool, String> {
            if let Some(handler) = &self.left_click_handler {
                return handler(t, tx); // Call the function if it exists
            }
            Ok(false)
        }
        fn right_click(&mut self, t: &mut State, tx: Arc<Sender<Message>>) -> Result<bool, String> {
            if let Some(handler) = &self.right_click_handler {
                return handler(t, tx); // Call the function if it exists
            }
            Ok(false)
        }
        fn scroll_up(&mut self, t: &mut State, tx: Arc<Sender<Message>>) -> Result<bool, String> {
            if let Some(handler) = &self.scroll_up_handler {
                return handler(t, tx); // Call the function if it exists
            }
            Ok(false)
        }
        fn scroll_down(&mut self, t: &mut State, tx: Arc<Sender<Message>>) -> Result<bool, String> {
            if let Some(handler) = &self.scroll_down_handler {
                return handler(t, tx); // Call the function if it exists
            }
            // LOG!("FALSE");
            Ok(false)
        }
    };
}

pub struct Window {
    win: Option<WINDOW>,
    pad: Option<WINDOW>,
    visited: bool,
    style_on: bool,
    title: Option<String>,
    children: Vec<Box<dyn DisplayContent>>,
    dimension: Dimension,
    create_win: bool,
    styles_before_populate: Vec<(STYLETYPE, attr_t)>,
    styles_after_populate: Vec<(STYLETYPE, attr_t)>,
    left_click_handler:
        Option<Arc<dyn Fn(&mut State, Arc<Sender<Message>>) -> Result<bool, String>>>,
    right_click_handler:
        Option<Arc<dyn Fn(&mut State, Arc<Sender<Message>>) -> Result<bool, String>>>,
    scroll_up_handler:
        Option<Arc<dyn Fn(&mut State, Arc<Sender<Message>>) -> Result<bool, String>>>,
    scroll_down_handler:
        Option<Arc<dyn Fn(&mut State, Arc<Sender<Message>>) -> Result<bool, String>>>,
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
        styles_before_populate: Vec<(STYLETYPE, attr_t)>,
        styles_after_populate: Vec<(STYLETYPE, attr_t)>,
    ) -> Window {
        let mut win = Window {
            children: vec![],
            title: Some(title),
            dimension: Dimension {
                height: 0,
                width: 0,
                startx: 0,
                starty: 0,
                scrollx: 0,
                scrolly: 0,
                lines: 0,
                initial_startx: startx,
                initial_starty: starty,
                display_height,
                display_width,
            },
            win: None,
            pad: None,
            visited: false,
            style_on: true,
            create_win: false,
            styles_after_populate,
            styles_before_populate,
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
                        .downcast_ref::<Arc<dyn Fn(&mut State,  Arc<Sender<Message>>) -> Result<bool, String>>>()
                    {
                        win.left_click_handler = Some(Arc::clone(handler));
                    }
                }
                if let Some(right_click_handler) = style.get("right_click") {
                    if let Some(handler) = right_click_handler
                        .downcast_ref::<Arc<dyn Fn(&mut State,  Arc<Sender<Message>>) -> Result<bool, String>>>()
                    {
                        win.right_click_handler = Some(Arc::clone(handler));
                    }
                }
                if let Some(scroll_up_handler) = style.get("scroll_up") {
                    if let Some(handler) = scroll_up_handler
                        .downcast_ref::<Arc<dyn Fn(&mut State,  Arc<Sender<Message>>) -> Result<bool, String>>>()
                    {
                        win.scroll_up_handler = Some(Arc::clone(handler));
                    }
                }
                if let Some(scroll_down_handler) = style.get("scroll_down") {
                    if let Some(handler) = scroll_down_handler
                        .downcast_ref::<Arc<dyn Fn(&mut State,  Arc<Sender<Message>>) -> Result<bool, String>>>()
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
    fn left_click(&mut self, t: &mut State, tx: Arc<Sender<Message>>) -> Result<bool, String> {
        if let Some(handler) = &self.left_click_handler {
            return handler(t, tx); // Call the function if it exists
        }
        Ok(false)
    }

    fn display_state(&mut self, _: &Item) -> Result<(), NulError> {
        // genNulError!();
        Ok(())
    }
}

pub struct ProgressBar {
    win: Option<WINDOW>,
    pad: Option<WINDOW>,
    visited: bool,
    style_on: bool,
    title: Option<String>,
    children: Vec<Box<dyn DisplayContent>>,
    dimension: Dimension,
    create_win: bool,
    styles_before_populate: Vec<(STYLETYPE, attr_t)>,
    styles_after_populate: Vec<(STYLETYPE, attr_t)>,
    bar_colors: Vec<i16>,
}

impl ProgressBar {
    /**
       if display_height = -1 expands to last of the screen
       Similarly for width
    */
    pub fn new(
        startx: i32,
        starty: i32,
        display_height: DimensionType,
        display_width: DimensionType,
        styles_before_populate: Vec<(STYLETYPE, attr_t)>,
        styles_after_populate: Vec<(STYLETYPE, attr_t)>,
    ) -> ProgressBar {
        let win = ProgressBar {
            children: vec![],
            title: None,
            dimension: Dimension {
                height: 0,
                width: 0,
                startx: 0,
                starty: 0,
                scrollx: 0,
                scrolly: 0,
                lines: 0,
                initial_startx: startx,
                initial_starty: starty,
                display_height,
                display_width,
            },
            win: None,
            pad: None,
            visited: false,
            style_on: true,
            create_win: false,

            styles_after_populate,
            styles_before_populate,
            bar_colors: vec![
                PAIR_BLACK_BLUE,
                PAIR_BLACK_YELLOW,
                PAIR_BLACK_MAGENTA,
                PAIR_BLACK_GREEN,
                PAIR_BLACK_CYAN,
                PAIR_BLACK_TURQUOISE,
                PAIR_BLACK_RED,
            ],
        };
        // win.re_initialize_win(height, width);
        win
    }
}

impl DisplayContent for ProgressBar {
    implement_getters_setters!();

    fn display_state(&mut self, state: &Item) -> Result<(), NulError> {
        // genNulError!();
        // LOG!("Namaste");
        match state {
            Item::NUM(val) => {
                let width = self.get_dim_unmut().width as f32;
                let width = (width * val).ceil().clamp(0., width) as usize;

                match self.pad {
                    Some(pad) => {
                        let num_colors = self.bar_colors.len();
                        // let acwidth = self.get_dim_unmut().width as usize;

                        if num_colors > 0 {
                            // let segment_width = acwidth / num_colors; // Base width per color segment

                            // let mut offset = 0;
                            // let mut prev = 0;
                            let ptr = state as *const _ as usize;
                            let i = ((width + ptr) % num_colors);
                            let color_pair = self.bar_colors[i];
                            wattrset(pad, COLOR_PAIR(color_pair));
                            waddstr(pad, &" ".repeat(width)); // Draw segment
                            wattroff(pad, COLOR_PAIR(color_pair));
                            // while offset < width {
                            //     offset += segment_width;
                            //     offset = offset.min(width);
                            //     let segment_size = offset - prev; // Final width for this color
                            //     let color_pair = self.bar_colors[i];
                            //     prev = offset;

                            //     if segment_size > 0 {
                            //         wattrset(pad, COLOR_PAIR(color_pair));
                            //         waddstr(pad, &" ".repeat(segment_size)); // Draw segment
                            //         wattroff(pad, COLOR_PAIR(color_pair));
                            //     }
                            // }
                        }
                    }
                    None => {
                        LOG!("PAD NOT FOUND for progress bar");
                    }
                }
            }
            _ => {}
        }
        Ok(())
    }
}

pub struct ScrollView {
    win: Option<WINDOW>,
    pad: Option<WINDOW>,
    visited: bool,
    style_on: bool,
    title: Option<String>,
    children: Vec<Box<dyn DisplayContent>>,
    dimension: Dimension,
    create_win: bool,
    styles_before_populate: Vec<(STYLETYPE, attr_t)>,
    styles_after_populate: Vec<(STYLETYPE, attr_t)>,
    scroll_up_handler:
        Option<Arc<dyn Fn(&mut State, Arc<Sender<Message>>) -> Result<bool, String>>>,
    scroll_down_handler:
        Option<Arc<dyn Fn(&mut State, Arc<Sender<Message>>) -> Result<bool, String>>>,
}

impl ScrollView {
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
        styles_before_populate: Vec<(STYLETYPE, attr_t)>,
        styles_after_populate: Vec<(STYLETYPE, attr_t)>,
    ) -> ScrollView {
        let mut win = ScrollView {
            children: vec![],
            title: Some(title),
            dimension: Dimension {
                height: 0,
                width: 0,
                startx: 0,
                starty: 0,
                scrollx: 0,
                scrolly: 0,
                lines: 0,
                initial_startx: startx,
                initial_starty: starty,
                display_height,
                display_width,
            },
            win: None,
            pad: None,
            visited: false,
            style_on: true,
            create_win: false,

            styles_after_populate,
            styles_before_populate,
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
                if let Some(scroll_up_handler) = style.get("scroll_up") {
                    if let Some(handler) = scroll_up_handler
                        .downcast_ref::<Arc<dyn Fn(&mut State,  Arc<Sender<Message>>) -> Result<bool, String>>>()
                    {
                        win.scroll_up_handler = Some(Arc::clone(handler));
                    }
                }
                if let Some(scroll_down_handler) = style.get("scroll_down") {
                    if let Some(handler) = scroll_down_handler
                        .downcast_ref::<Arc<dyn Fn(&mut State,  Arc<Sender<Message>>) -> Result<bool, String>>>()
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

impl DisplayContent for ScrollView {
    implement_getters_setters!();

    // implement_listeners!();

    fn scroll_down(
        &mut self,
        _: &mut State,
        tx_frontend: Arc<Sender<Message>>,
    ) -> Result<bool, String> {
        let len = self.get_children().len();
        let visited = self.get_visited();
        let dim = self.get_dim_unmut();
        if visited && (dim.height > 2 && len as i32 - dim.height >= dim.scrolly as i32 - 2)
            || (len as i32 - dim.height >= dim.scrolly as i32 && dim.height <= 2)
        {
            self.get_dim().scrolly += 1;
            let _ = tx_frontend.send(Message {
                content: Some(Arc::new(
                    self.title
                        .clone()
                        .expect("Title is necessary for scroll window"),
                )),
                mtype: MessageType::RELOAD,
            });
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn scroll_up(
        &mut self,
        _: &mut State,
        tx_frontend: Arc<Sender<Message>>,
    ) -> Result<bool, String> {
        let visited = self.get_visited();
        // LOG!(format!("{}", visited));
        if visited && 1 <= self.get_dim_unmut().scrolly as usize {
            self.get_dim().scrolly -= 1;
            let _ = tx_frontend.send(Message {
                content: Some(Arc::new(
                    self.title
                        .clone()
                        .expect("Title is necessary for scroll window"),
                )),
                mtype: MessageType::RELOAD,
            });
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn display_state(&mut self, _: &Item) -> Result<(), NulError> {
        // genNulError!();
        Ok(())
    }
}

pub struct TextBox {
    win: Option<WINDOW>,
    pad: Option<WINDOW>,
    visited: bool,
    style_on: bool,
    title: Option<String>,
    children: Vec<Box<dyn DisplayContent>>,
    dimension: Dimension,
    create_win: bool,
    styles_before_populate: Vec<(STYLETYPE, attr_t)>,
    styles_after_populate: Vec<(STYLETYPE, attr_t)>,
    left_click_handler:
        Option<Arc<dyn Fn(&mut State, Arc<Sender<Message>>) -> Result<bool, String>>>,
    right_click_handler:
        Option<Arc<dyn Fn(&mut State, Arc<Sender<Message>>) -> Result<bool, String>>>,
    scroll_up_handler:
        Option<Arc<dyn Fn(&mut State, Arc<Sender<Message>>) -> Result<bool, String>>>,
    scroll_down_handler:
        Option<Arc<dyn Fn(&mut State, Arc<Sender<Message>>) -> Result<bool, String>>>,
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
        styles_before_populate: Vec<(STYLETYPE, attr_t)>,
        styles_after_populate: Vec<(STYLETYPE, attr_t)>,
    ) -> TextBox {
        let mut win = TextBox {
            children: vec![],
            title: None,
            dimension: Dimension {
                height: 0,
                width: 0,
                startx: 0,
                starty: 0,
                scrollx: 0,
                scrolly: 0,
                lines: 0,
                initial_startx: startx,
                initial_starty: starty,
                display_height,
                display_width,
            },
            win: None,
            pad: None,
            visited: false,
            style_on: true,
            create_win: false,

            styles_after_populate,
            styles_before_populate,
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
                        .downcast_ref::<Arc<dyn Fn(&mut State,  Arc<Sender<Message>>) -> Result<bool, String>>>()
                    {
                        Some(handler) => {
                            win.left_click_handler = Some(Arc::clone(handler));
                        }
                        None => {}
                    }
                }
                if let Some(right_click_handler) = style.get("right_click") {
                    if let Some(handler) = right_click_handler
                        .downcast_ref::<Arc<dyn Fn(&mut State,  Arc<Sender<Message>>) -> Result<bool, String>>>()
                    {
                        win.right_click_handler = Some(Arc::clone(handler));
                    }
                }
                if let Some(scroll_up_handler) = style.get("scroll_up") {
                    if let Some(handler) = scroll_up_handler
                        .downcast_ref::<Arc<dyn Fn(&mut State,  Arc<Sender<Message>>) -> Result<bool, String>>>()
                    {
                        win.scroll_up_handler = Some(Arc::clone(handler));
                    }
                }
                if let Some(scroll_down_handler) = style.get("scroll_down") {
                    if let Some(handler) = scroll_down_handler
                        .downcast_ref::<Arc<dyn Fn(&mut State,  Arc<Sender<Message>>) -> Result<bool, String>>>()
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
            Item::STRING(val) => val,
            _ => {
                genNulError!();
                &format!("")
            }
        };
        // let pad = self.pad.expect(&format!("Empty pad : TextBox {}", value));
        match self.pad {
            Some(pad) => {
                // wbkgd(pad, COLOR_PAIR(PAIR_BLACK_TURQUOISE)); // when this is selected
                wprintw(pad, &value)?;
                // wbkgd(pad, COLOR_PAIR(PAIR_WHITE_BLACK));
            }
            None => {
                // LOG!(format!("TEXT PAD NOT FOUND {}",value));
            }
        }

        Ok(())
    }
}

pub struct FileInfoWin {
    win: Option<WINDOW>,
    pad: Option<WINDOW>,
    visited: bool,
    style_on: bool,
    title: Option<String>,
    create_win: bool,
    children: Vec<Box<dyn DisplayContent>>,
    dimension: Dimension,
    styles_before_populate: Vec<(STYLETYPE, attr_t)>,
    styles_after_populate: Vec<(STYLETYPE, attr_t)>,
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
        styles_before_populate: Vec<(STYLETYPE, attr_t)>,
        styles_after_populate: Vec<(STYLETYPE, attr_t)>,
    ) -> FileInfoWin {
        let mut children: Vec<Box<dyn DisplayContent>> = vec![];
        for _ in 0..10 {
            children.push(Box::new(InfoBox::new(
                0,
                -2,
                DimensionType::DIMENS(1),
                DimensionType::PERCEN(0.5),
                vec![],
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
                scrollx: 0,
                scrolly: 0,
                lines: 0,
                initial_startx: startx,
                initial_starty: starty,
                display_height,
                display_width,
            },
            win: None,
            pad: None,
            visited: false,
            style_on: true,
            create_win: false,
            styles_before_populate,
            styles_after_populate,
        };
        // win.re_initialize_win(height, width);
        win
    }
}

impl DisplayContent for FileInfoWin {
    implement_getters_setters!();

    fn display_state(&mut self, state: &Item) -> Result<(), NulError> {
        let _: &String = match state {
            Item::STRING(val) => val,
            _ => {
                genNulError!();
                &format!("")
            }
        };
        // let pad = self.pad.expect("Empty pad : TextBox");
        // wprintw(pad, &value)?;

        Ok(())
    }
}

pub struct InfoBox {
    win: Option<WINDOW>,
    pad: Option<WINDOW>,
    visited: bool,
    style_on: bool,
    title: Option<String>,
    create_win: bool,
    children: Vec<Box<dyn DisplayContent>>,
    dimension: Dimension,
    styles_before_populate: Vec<(STYLETYPE, attr_t)>,
    styles_after_populate: Vec<(STYLETYPE, attr_t)>,
}



impl InfoBox {
    /**
       if display_height = -1 expands to last of the screen
       Similarly for width
    */
    pub fn new(
        startx: i32,
        starty: i32,
        display_height: DimensionType,
        display_width: DimensionType,
        styles_before_populate: Vec<(STYLETYPE, attr_t)>,
        styles_after_populate: Vec<(STYLETYPE, attr_t)>,
    ) -> InfoBox {
        let children: Vec<Box<dyn DisplayContent>> = vec![
            Box::new(TextBox::new(
                -1  ,
                0,
                DimensionType::DIMENS(1),
                DimensionType::PERCEN(0.4),
                None,
                vec![(STYLETYPE::STARTCOLOR, COLOR_PAIR(PAIR_YELLOW_BLACK))],
                vec![(STYLETYPE::REMOVECOLOR, COLOR_PAIR(PAIR_YELLOW_BLACK))],
            )),
            Box::new(TextBox::new(
                -1,
                0,
                DimensionType::DIMENS(1),
                DimensionType::PERCEN(0.6),
                None,
                vec![(STYLETYPE::STARTCOLOR, COLOR_PAIR(PAIR_ORANGE_BLACK))],
                vec![(STYLETYPE::REMOVECOLOR, COLOR_PAIR(PAIR_ORANGE_BLACK))],
            )),
        ];
        let win = InfoBox {
            children: children,
            title: None,
            dimension: Dimension {
                height: 0,
                width: 0,
                startx: 0,
                starty: 0,
                scrollx: 0,
                scrolly: 0,
                lines: 0,
                initial_startx: startx,
                initial_starty: starty,
                display_height,
                display_width,
            },
            win: None,
            pad: None,
            visited: false,
            style_on: true,
            create_win: false,
            styles_after_populate,
            styles_before_populate
        };
        // win.re_initialize_win(height, width);
        win
    }

}

impl DisplayContent for InfoBox {
    implement_getters_setters!();

    fn display_state(&mut self, state: &Item) -> Result<(), NulError> {
        match state {
            Item::INFO((str1, str2)) => {
                // LOG!(format!("LOGGING: {}", str2));
                let next_states = [State::VALUE(Item::STRING((*str1).to_owned())),State::VALUE(Item::STRING((*str2).to_owned())),];
                self.children
                    .iter_mut()
                    .zip(next_states)
                    .try_for_each(|(win, state)| win.populate(&state))?;
            }
            _ => {}
        };

        Ok(())
    }

}


pub struct StorageWin {
    win: Option<WINDOW>,
    pad: Option<WINDOW>,
    visited: bool,
    style_on: bool,
    title: Option<String>,
    create_win: bool,
    children: Vec<Box<dyn DisplayContent>>,
    dimension: Dimension,
    styles_before_populate: Vec<(STYLETYPE, attr_t)>,
    styles_after_populate: Vec<(STYLETYPE, attr_t)>,
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
        styles_before_populate: Vec<(STYLETYPE, attr_t)>,
        styles_after_populate: Vec<(STYLETYPE, attr_t)>,
    ) -> StorageWin {
        let mut win = StorageWin {
            children: vec![],
            title: None,
            dimension: Dimension {
                height: 0,
                width: 0,
                startx: 0,
                starty: 0,
                scrollx: 0,
                scrolly: 0,
                lines: 0,
                initial_startx: startx,
                initial_starty: starty,
                display_height,
                display_width,
            },
            win: None,
            pad: None,
            visited: false,
            style_on: true,
            create_win: false,
            styles_after_populate,
            styles_before_populate
        };
        win.collapse();
        // win.re_initialize_win(height, width);
        win
    }

    fn expand(&mut self, len: usize) {
        // LOG!("Expan");
        self.children.clear();

        let sort_by_name: Arc<dyn Fn(&mut State, Arc<Sender<Message>>) -> Result<bool, String>> =
            Arc::new(
                |t: &mut State, tx_frontend: Arc<Sender<Message>>| -> Result<bool, String> {
                    match t {
                        State::LIST(_) => {}
                        State::VALUE(item) => {
                            if let Item::SORT(val) = item {
                                // LOG!(format!("Lets go {}", val.context));
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
                move |t: &mut State, tx_frontend: Arc<Sender<Message>>| -> Result<bool, String> {
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

        self.children.push(Box::new(TextBox::new(
            0,
            -1,
            DimensionType::DIMENS(1),
            DimensionType::DIMENS(-1),
            None,
            vec![
                (STYLETYPE::TOPBORDER, 0),
                (STYLETYPE::SPECIALCHARS, ACS_DARROW()),
            ],
            vec![]
        )));

        // let mut style: HashMap<String, &dyn Any> = HashMap::new();

        self.children.push(Box::new(HeaderWin::new(
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
            DimensionType::DIMENS(len as i32 + 2),
            DimensionType::DIMENS(-1),
            None,
            vec![],
            vec![],
        ));

        for _ in 0..len {
            storage_inner_window.add_child(Box::new(StorageWin::new(
                0,
                -1,
                DimensionType::DIMENS(1),
                DimensionType::PERCEN(1.),
                vec![],
            vec![],
            )));
        }
        self.children.push(storage_inner_window);

        // self.children.push(Box::new(TextBox::new(
        //     0,
        //     -1,
        //     DimensionType::DIMENS(1),
        //     DimensionType::DIMENS(-1),
        //     None,
        //     vec![
        //         (STYLETYPE::TOPBORDER, 0),
        //         (STYLETYPE::SPECIALCHARS, ACS_UARROW()),
        //     ],
        //     vec![],
        // )));
    }

    fn collapse(&mut self) {
        self.children.clear();

        for _ in 0..3 {
            self.children.push(Box::new(TextBox::new(
                -2,
                0,
                DimensionType::DIMENS(1),
                DimensionType::PERCEN(0.2),
                None,
                vec![],
                vec![],
            )));
        }
        self.children.push(Box::new(ProgressBar::new(
            -1,
            0,
            DimensionType::DIMENS(1),
            DimensionType::DIMENS(-1),
            vec![],
            vec![],
        )));
    }
}

impl DisplayContent for StorageWin {
    implement_getters_setters!();

    fn display_state(&mut self, state: &Item) -> Result<(), NulError> {
        match state {
            Item::DIRECTORY(dir_info) => {
                // LOG!("LOGGING: DIRECTORY");
                let next_states = self.calculate_next_states(dir_info);
                self.children
                    .iter_mut()
                    .zip(next_states)
                    .try_for_each(|(win, state)| win.populate(&state))?;
            }
            _ => {}
        };

        Ok(())
    }

    fn left_click(
        &mut self,
        t: &mut State,
        tx_frontend: Arc<Sender<Message>>,
    ) -> Result<bool, String> {
        // LOG!("Here");
        match t {
            State::LIST(_) => {}
            State::VALUE(item) => match item {
                Item::DIRECTORY(dir_info) => {
                    let dirent = &**dir_info;
                    match dirent {
                        Dirent::AGGREGATE(mutex) => {
                            let mut agg = mutex.lock().unwrap();
                            let dim = self.get_dim();
                            // LOG!(format!("EXP: {} {}",agg.expanded, agg.common_name));
                            if !agg.expanded {
                                // LOG!(format!("To Expand: {}", agg.common_name));
                                let direntrys = &agg.dirents;
                                let len = direntrys.len();
                                dim.display_height = DimensionType::DIMENS(3 + len as i32);
                                self.expand(len);
                            } else {
                                // LOG!(format!("To Collaps: {}", agg.common_name));
                                dim.display_height = DimensionType::DIMENS(1);
                                self.collapse();
                            }
                            agg.expanded = !agg.expanded;
                            // LOG!("SENDING");
                            let _ = tx_frontend.send(Message {
                                content: None,
                                mtype: MessageType::RELOADSTORAGE,
                            });
                        }
                        Dirent::VALUE(val) => {
                            // let entry = val.lock().unwrap();
                            // LOG!(format!("G {}",entry.name));
                            let _ = tx_frontend.send(Message {
                                content: Some(Arc::new(val.lock().unwrap().name.clone())),
                                mtype: MessageType::CHANGEFILEINFO,
                            });
                        }
                    }
                }
                _ => {}
            },
        }
        Ok(true)
    }
}

pub struct HeaderWin {
    win: Option<WINDOW>,
    pad: Option<WINDOW>,
    visited: bool,
    style_on: bool,
    title: Option<String>,
    create_win: bool,
    children: Vec<Box<dyn DisplayContent>>,
    dimension: Dimension,
    styles_before_populate: Vec<(STYLETYPE, attr_t)>,
    styles_after_populate: Vec<(STYLETYPE, attr_t)>,
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
        sort_by_name: Arc<dyn Fn(&mut State, Arc<Sender<Message>>) -> Result<bool, String>>,
        sort_by_size: Arc<dyn Fn(&mut State, Arc<Sender<Message>>) -> Result<bool, String>>,
        styles_before_populate: Vec<(STYLETYPE, attr_t)>,
        styles_after_populate: Vec<(STYLETYPE, attr_t)>,
    ) -> HeaderWin {
        // LOG!("=------");
        let mut children: Vec<Box<dyn DisplayContent>> = vec![];
        let colors = [PAIR_YELLOW_BLACK, PAIR_BLUE_BLACK, PAIR_GREEN_BLACK];
        for i in 0..3 {
            children.push(Box::new(TextBox::new(
                -2,
                0,
                DimensionType::DIMENS(1),
                DimensionType::PERCEN(0.2),
                None,
                vec![(STYLETYPE::STARTCOLOR, COLOR_PAIR(colors[i]))],
                vec![(STYLETYPE::REMOVECOLOR, COLOR_PAIR(colors[i]))],
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
            vec![(STYLETYPE::STARTCOLOR, COLOR_PAIR(PAIR_DARKPURPLE_BLACK))],
                vec![(STYLETYPE::REMOVECOLOR, COLOR_PAIR(PAIR_DARKPURPLE_BLACK))],
        )));
        style.clear();
        style.insert(format!("left_click"), &sort_by_size);
        children.push(Box::new(Button::new(
            -2,
            0,
            DimensionType::DIMENS(1),
            DimensionType::PERCEN(0.2),
            Some(&style),
            vec![(STYLETYPE::STARTCOLOR, COLOR_PAIR(PAIR_TURQUOISE_BLACK))],
                vec![(STYLETYPE::REMOVECOLOR, COLOR_PAIR(PAIR_TURQUOISE_BLACK))],
        )));

        let win = HeaderWin {
            children,
            title: None,
            dimension: Dimension {
                height: 0,
                width: 0,
                startx: 0,
                starty: 0,
                scrollx: 0,
                scrolly: 0,
                lines: 0,
                initial_startx: startx,
                initial_starty: starty,
                display_height,
                display_width,
            },
            win: None,
            pad: None,
            visited: false,
            style_on: true,
            create_win: false,
            styles_after_populate,
            styles_before_populate
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

    fn left_click(&mut self, _: &mut State, _: Arc<Sender<Message>>) -> Result<bool, String> {
        // LOG!("J");
        Ok(false)
    }
}

pub struct Button {
    win: Option<WINDOW>,
    pad: Option<WINDOW>,
    visited: bool,
    style_on: bool,
    title: Option<String>,
    children: Vec<Box<dyn DisplayContent>>,
    dimension: Dimension,
    create_win: bool,
    styles_before_populate: Vec<(STYLETYPE, attr_t)>,
    styles_after_populate: Vec<(STYLETYPE, attr_t)>,
    left_click_handler:
        Option<Arc<dyn Fn(&mut State, Arc<Sender<Message>>) -> Result<bool, String>>>,
    right_click_handler:
        Option<Arc<dyn Fn(&mut State, Arc<Sender<Message>>) -> Result<bool, String>>>,
    scroll_up_handler:
        Option<Arc<dyn Fn(&mut State, Arc<Sender<Message>>) -> Result<bool, String>>>,
    scroll_down_handler:
        Option<Arc<dyn Fn(&mut State, Arc<Sender<Message>>) -> Result<bool, String>>>,
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
        styles_before_populate: Vec<(STYLETYPE, attr_t)>,
        styles_after_populate: Vec<(STYLETYPE, attr_t)>,
    ) -> Button {
        let mut win = Button {
            children: vec![],
            title: None,
            dimension: Dimension {
                height: 0,
                width: 0,
                startx: 0,
                starty: 0,
                scrollx: 0,
                scrolly: 0,
                lines: 0,
                initial_startx: startx,
                initial_starty: starty,
                display_height,
                display_width,
            },
            win: None,
            pad: None,
            visited: false,
            style_on: true,
            create_win: false,

            styles_after_populate,
            styles_before_populate,
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
                        .downcast_ref::<Arc<dyn Fn(&mut State,  Arc<Sender<Message>>) -> Result<bool, String>>>()
                    {
                        Some(handler) => {
                            win.left_click_handler = Some(Arc::clone(handler));
                        }
                        None => {}
                    }
                }
                if let Some(right_click_handler) = style.get("right_click") {
                    if let Some(handler) = right_click_handler
                        .downcast_ref::<Arc<dyn Fn(&mut State,  Arc<Sender<Message>>) -> Result<bool, String>>>()
                    {
                        win.right_click_handler = Some(Arc::clone(handler));
                    }
                }
                if let Some(scroll_up_handler) = style.get("scroll_up") {
                    if let Some(handler) = scroll_up_handler
                        .downcast_ref::<Arc<dyn Fn(&mut State,  Arc<Sender<Message>>) -> Result<bool, String>>>()
                    {
                        win.scroll_up_handler = Some(Arc::clone(handler));
                    }
                }
                if let Some(scroll_down_handler) = style.get("scroll_down") {
                    if let Some(handler) = scroll_down_handler
                        .downcast_ref::<Arc<dyn Fn(&mut State,  Arc<Sender<Message>>) -> Result<bool, String>>>()
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
            Item::SORT(sort_button) => &sort_button.name,
            _ => {
                genNulError!();
                &format!("")
            }
        };
        match self.pad {
            Some(pad) => {
                wprintw(pad, &value)?;
            }
            None => {
                // LOG!(format!("BUTTON PAD NOT FOUND {}",value));
            }
        }

        Ok(())
    }
}
