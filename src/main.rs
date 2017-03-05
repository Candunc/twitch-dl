extern crate rustc_serialize;
#[macro_use] extern crate hyper;
extern crate hyper_rustls;
extern crate scoped_threadpool;
extern crate docopt;

use docopt::Docopt;
use std::process;
use std::process::Command;
use std::env::consts::OS;

mod https;
mod twitch;

/* Exit Codes:
1: Missing arguments
2: Invalid arguments
3: Missing dependency
*/

const USAGE: &'static str = "
Usage: twitch-dl <source> <quality>
Where quality is chunked, high, medium, low, or mobile.
";

#[derive(Debug, RustcDecodable)]
struct Args {
	arg_source: String,
	arg_quality: String,
}

fn main() {
	verify_deps();

	let args: Args = Docopt::new(USAGE)
		.and_then(|d| d.decode())
		.unwrap_or_else(|e| e.exit());

	let quality: usize = match args.arg_quality.to_lowercase().as_ref() {
		"highest"	=> 0,
		"source"	=> 0,
		"chunked"	=> 0,
		"high"		=> 1,
		"medium"	=> 2,
		"low"		=> 3,
		"mobile"	=> 4,
		"lowest"	=> 4,
		_ => { println!("Invalid quality argument, run --help"); process::exit(2);},
	};

	// This is a lazy search for the twitch VOD id.
	// It appears numeric, so we search for a string containing a large number.
	let mut num: Result<u32, std::num::ParseIntError>;
	let mut source: &str = "";
	let source_iter = args.arg_source.split("/");
	for part in source_iter {
		num = part.parse::<u32>();
		match num {
			Ok(_) => source = part,
			Err(_) => { },
		}
	}

	if source == "" {
		println!("Couldn't find a VOD to download!\nInput was: {}",args.arg_source);
		process::exit(2);
	}
	twitch::get_vod(source,quality);
}

//	Do not return value as we exit on failure.
fn verify_deps() {
	let name: &str = match OS {
		"windows"	=> "where.exe",
		"linux"		=> "which",
		"macos"		=> "which",
		_			=> { panic!(format!("Unsupported operating system! File an error report -> main.rs/verify_deps {}",OS)); }
	};

	let output = Command::new(name)
		.arg("ffmpeg")
		.output()	//Grab output to suppress stdout & stderr
		.unwrap_or_else(|e| { panic!("failed to execute process: {}", e) });
	let status = output.status.code().unwrap();

	if status != 0 {
		println!("Could not find ffmpeg installed! Please install it and try again.");
		process::exit(3);
	}
}