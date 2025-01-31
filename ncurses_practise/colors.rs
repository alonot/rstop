use std::{ffi::NulError, fmt::Error};

use ncurses::{addch, addstr, attroff, attrset, endwin, getch, has_colors, init_pair, initscr, ll::{mvaddch, napms, noecho, refresh, start_color, use_default_colors}, printw, ACS_ULCORNER, A_BOLD, COLOR_BLACK, COLOR_BLUE, COLOR_GREEN, COLOR_PAIR, COLOR_WHITE, COLOR_YELLOW, OK};

macro_rules! unsafe_call {
    ($fn:ident $(,$param:expr)*) => {
        unsafe {
            $fn(
                $($param,)*
            )
        }
    };
}

fn print_with_colors() -> Result<(), NulError>{
    if (has_colors()) {
        if (unsafe_call!(start_color) == OK) {
            unsafe_call!(use_default_colors);

            // important
            init_pair(1, COLOR_YELLOW, -1);
            init_pair(2, COLOR_BLUE, COLOR_WHITE);
            init_pair(3, COLOR_GREEN, COLOR_BLACK);  // good for only for black background terminal 
            
            //


            let mut row = 5; let mut col = 0;
            for i in 'a'..'z' {
                let cpair = COLOR_PAIR((i as i16 % 3) + 1);
                if cpair == 2 {
                    attrset(A_BOLD);
                }
                attrset(cpair);
                unsafe_call!(mvaddch,row, col, i as u32);
                row += 1;col += 1;
                unsafe_call!(refresh);
                attroff(cpair);
                if cpair == 2 {
                    attroff(A_BOLD);
                }
                unsafe_call!(napms,120); // sleeps for 10 ms
            }
            attrset(COLOR_PAIR(1) | A_BOLD);
        } else {
            addstr("Cannot Start Colors")?;
            unsafe_call!(refresh);
        }
    } else {
        addstr("Does not support Colors\n")?;
        unsafe_call!(refresh);
    }
    Ok(())
}

fn main() {
    println!("Hello, world!");
    initscr();

    unsafe_call!(noecho);

    addch(ACS_ULCORNER());
    
    addstr("---------------------------\n|    Welcome      |\n------------------------\n")?;

    print_with_colors()?;

    unsafe_call!(refresh);
    addstr("\n Press `q` exit...\n")?;
    unsafe_call!(refresh);

    loop {
        let ch= getch();
        if ch == 'q' as i32 || ch == 'Q' as i32 {
            break;
        }
    }
    
    endwin();

    Ok(())
}
