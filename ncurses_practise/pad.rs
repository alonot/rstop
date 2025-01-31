use std::ffi::NulError;

use ncurses::{addstr, box_, cbreak, clear, endwin, getch, getmaxyx, initscr, keypad, mvwaddch, mvwprintw, newpad, newwin, noecho, prefresh, refresh, stdscr, wprintw, wrefresh, COLS, KEY_RESIZE, LINES, WINDOW};

macro_rules! unsafe_call {
    ($fn:ident $(,$param:expr)*) => {
        unsafe {
            $fn(
                $($param,)*
            )
        }
    };
}

fn initialize_screen() -> Result<(), NulError>{
    endwin();
    clear();

    let border_my_win: WINDOW;let my_win : WINDOW;
    let mut height:i32= 0;
    let mut width:i32 = 0;
    getmaxyx(stdscr(), &mut height, &mut width);
    let startx = (COLS() - width) / 2;
    let starty = (LINES() - height) / 2;
    addstr(&format!("{}, {}, {}, {}",height, width, LINES(), COLS() ))?;
    refresh();

    border_my_win = newwin(height, width, starty, startx);
    
    box_(border_my_win, 0, 0);
    my_win = newwin(height - 2, width - 2, starty + 1, startx + 1);
    mvwprintw(my_win, 0, 0, &format!("{} {}\n", height, width))?;
    // wprintw(my_win, "Hello\n")?;
    
    // for _ in 1..(LINES() + 100) {
    //     wprintw(my_win, "Hello\n")?;
    // }
    wrefresh(border_my_win);
    wrefresh(my_win);

    let pad_height = height + 20;
    let pad_width = 20;
    let pad = newpad(pad_height, pad_width);
    let mut disp_char = 'a';
    wprintw(pad, &format!("{} {}\n", pad_height, pad_width))?;
    for  x in 1..(pad_height + 1)
    {
        for  y in 1..(pad_width + 1)
        {
            wprintw(pad, &format!("File1_file0_file10.txt\n"))?;
        }
    }
    // box_(pad, 0, 0);
    prefresh(pad, 0, 0, 2, 1, height - 2,width - 2);
    
    

    Ok(())

}

fn main() -> Result<(), NulError> 
{
    initscr();
    // cbreak();
    keypad(stdscr(), true);
    noecho();
    initialize_screen()?;

    loop {

        let ch= getch();
        if ch == KEY_RESIZE {
            initialize_screen()?; 
        }
        if ch == 'q' as i32 || ch == 'Q' as i32 {
            break;
        }
    }

    endwin();
    Ok(())

}
