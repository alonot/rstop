use std::{any::{self, Any}, collections::HashMap, ffi::{CString, NulError}};

use ncurses::{wprintw, WINDOW};

use super::models::{Dimension, DimensionType, DisplayContent, Item};

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

        fn add_child(&mut self, win: Box<dyn DisplayContent>) {
            self.children.push(win);
        }
        fn with_border(&self) -> bool {
            self.with_border
        }
    };
}

pub struct Window {
    win: Option<WINDOW>,
    pad: Option<WINDOW>,
    title: Option<String>,
    children: Vec<Box<dyn DisplayContent>>,
    dimension: Dimension,
    with_border: bool,
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
        style: Option<&HashMap<String, &dyn Any>>
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
            with_border: false
        };
        match style {
            Some(style) => {
                if style.contains_key("with_border") {
                    let with_border: &&dyn Any = style.get("with_border").expect("msg");
                    if with_border.is::<bool>() {
                        win.with_border = *with_border.downcast_ref::<bool>().unwrap();
                    }
                }
            },
            None => {},
        }
        // win.re_initialize_win(height, width);
        win
    }
}

macro_rules! genNulError {
    () => {
        _ = CString::new("\0")? // always throws NulError
    };
}

impl DisplayContent for Window {
    implement_getters_setters!();

    fn display_state(&mut self, state: &Item) -> Result<(), NulError> {
        genNulError!();
        Ok(())
    }
}

pub struct TextBox {
    win: Option<WINDOW>,
    pad: Option<WINDOW>,
    title: Option<String>,
    children: Vec<Box<dyn DisplayContent>>,
    dimension: Dimension,
    with_border: bool,
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
    ) -> TextBox {
        let win = TextBox {
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
            with_border: false
        };
        // win.re_initialize_win(height, width);
        win
    }
}

impl DisplayContent for TextBox {
    implement_getters_setters!();

    fn display_state(&mut self, state: &Item) -> Result<(), NulError> {
        let value: &String = match state {
            Item::DIRECTORY(dir_info) => {
                genNulError!();
                &format!("")
            }
            Item::STRING(val) => val,
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
    children: Vec<Box<dyn DisplayContent>>,
    dimension: Dimension,
    with_border: bool,
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
    ) -> FileInfoWin {
        let mut children: Vec<Box<dyn DisplayContent>> = vec![];
        children.push(Box::new(TextBox::new(
            0,
            -2,
            DimensionType::DIMENS(1),
            DimensionType::PERCEN(0.5),
        )));
        children.push(Box::new(TextBox::new(
            0,
            -2,
            DimensionType::DIMENS(1),
            DimensionType::PERCEN(0.5),
        )));
        children.push(Box::new(TextBox::new(
            0,
            -2,
            DimensionType::DIMENS(1),
            DimensionType::PERCEN(0.5),
        )));
        children.push(Box::new(TextBox::new(
            0,
            -2,
            DimensionType::DIMENS(1),
            DimensionType::PERCEN(0.5),
        )));
        children.push(Box::new(TextBox::new(
            0,
            -2,
            DimensionType::DIMENS(1),
            DimensionType::PERCEN(0.5),
        )));
        children.push(Box::new(TextBox::new(
            0,
            -2,
            DimensionType::DIMENS(1),
            DimensionType::PERCEN(0.5),
        )));
        children.push(Box::new(TextBox::new(
            0,
            -2,
            DimensionType::DIMENS(1),
            DimensionType::PERCEN(0.5),
        )));
        children.push(Box::new(TextBox::new(
            0,
            -2,
            DimensionType::DIMENS(1),
            DimensionType::PERCEN(0.5),
        )));
        children.push(Box::new(TextBox::new(
            0,
            -2,
            DimensionType::DIMENS(1),
            DimensionType::PERCEN(0.5),
        )));
        children.push(Box::new(TextBox::new(
            0,
            -2,
            DimensionType::DIMENS(1),
            DimensionType::PERCEN(0.5),
        )));

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
            with_border: true
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
        };
        let pad = self.pad.expect("Empty pad : TextBox");
        wprintw(pad, &value)?;

        Ok(())
    }
}
