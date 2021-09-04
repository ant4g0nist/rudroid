use xmas_elf::program;
use crate::core::unicorn::unicorn_const::Protection;

pub struct TerminalSize {
	width : u16,
	height : u16
}

impl Default for TerminalSize {
	fn default() -> Self {
		use libc::ioctl;
		use libc::isatty;
		use libc::{winsize as WinSize, TIOCGWINSZ};
		
		let mut winsize = WinSize {
			ws_row: 0,
			ws_col: 0,
			ws_xpixel: 0,
			ws_ypixel: 0,
		};

		if unsafe { ioctl(libc::STDOUT_FILENO, TIOCGWINSZ.into(), &mut winsize) } == -1 {
			TerminalSize {
				width : 20,
				height : 20
			}
		}	

		else {
			TerminalSize {
				width : winsize.ws_col-1,
				height : winsize.ws_row-1
			}
		}		
	}
}

enum Colors {
	RESET,
	BLACK,
	RED,
	GREEN,
	YELLOW,
	BLUE,
	MAGENTA,
	CYAN,
	WHITE,
	DEFAULT
}

fn give_me_color(color: Colors) -> String {

	match color {
		Colors::RESET => {
			"\x1b[0m".to_string()
		},
		Colors::BLACK => {
			"\x1b[30m".to_string()
		},
		Colors::RED => {
			"\x1b[31m".to_string()
		},
		Colors::GREEN => {
			"\x1b[32m".to_string()
		},
		Colors::YELLOW => {
			"\x1b[33m".to_string()
		},
		Colors::BLUE => {
			"\x1b[34m".to_string()
		},
		Colors::MAGENTA => {
			"\x1b[35m".to_string()
		},
		Colors::CYAN => {
			"\x1b[36m".to_string()
		},
		Colors::WHITE => {
			"\x1b[37m".to_string()
		},
		Colors::DEFAULT => {
			"\x1b[39m".to_string()
		},
	}
}

pub fn draw_line() {
	let horizontal_line = "-";
	let terminal_size : TerminalSize = Default::default();
	
	let line_color = give_me_color(Colors::GREEN);
	let msg_color  = give_me_color(Colors::RED);
	let reset 	   = give_me_color(Colors::RESET);
	
	println!("{} {} {}", line_color, (0..terminal_size.width-2).map(|_|horizontal_line).collect::<String>() , reset);
}

pub fn context_title(msg: Option<&str>) {
	let horizontal_line = "-";
	let terminal_size : TerminalSize = Default::default();
	let line_color = give_me_color(Colors::GREEN);
	let msg_color  = give_me_color(Colors::RED);
	let reset 	   = give_me_color(Colors::RESET);

	match msg {
		Some(msg) => {

            let trail_len: u16 = (msg.len() + 12) as u16;
			let mut title = String::new();

			title.push_str(&give_me_color(Colors::GREEN));
			title.push_str(&format!(" {} [ ", (0..4).map(|_|horizontal_line).collect::<String>()));
			title.push_str(&msg_color);
			title.push_str(msg);
			title.push_str(&give_me_color(Colors::GREEN));
			title.push_str(" ] ");
			if terminal_size.width  > trail_len {
				title.push_str(&(0..terminal_size.width - trail_len).map(|_|horizontal_line).collect::<String>());
			}
			else {
				title.push_str(&(0..20).map(|_|horizontal_line).collect::<String>());
			}
			
			println!("{} {}", title, reset);

		},

		None	  => println!("{} {} {}", line_color, (0..terminal_size.width-2).map(|_|horizontal_line).collect::<String>() , reset),
	}
}

pub enum DebugLevel {
	DEBUG,
	INFO,
	ERROR
}

pub fn log(msg: &str, level: DebugLevel) {
	let mut debug_color = give_me_color(Colors::MAGENTA);
	let mut debug_type 	= "DEFAULT";
	let reset 		= give_me_color(Colors::RESET);

	match level {
		DebugLevel::DEBUG => {
			debug_color = give_me_color(Colors::GREEN);
			debug_type 	= "DEBUG";
		},
		DebugLevel::INFO => {
			debug_color = give_me_color(Colors::CYAN);
			debug_type 		= "INFO";
		},
		DebugLevel::ERROR => {
			debug_color = give_me_color(Colors::RED);
			debug_type 		= "ERROR";
		}
	}
	println!("{}[{}]: {} {}", debug_color, debug_type, reset, msg);	
}

pub fn to_uc_permissions(perms: program::Flags) -> Protection {
	let mut uc_perms: Protection = Protection::NONE;

	if perms.is_execute() {
		uc_perms = uc_perms | Protection::EXEC;
		// assumes read if execute
		uc_perms = uc_perms | Protection::READ;
	}

    if perms.is_write() {
		uc_perms = uc_perms | Protection::WRITE;
    }

    if perms.is_read() {
		uc_perms = uc_perms | Protection::READ;
	}
	
	uc_perms
}