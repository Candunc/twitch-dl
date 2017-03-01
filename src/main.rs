extern crate rustc_serialize;
#[macro_use] extern crate hyper;
extern crate hyper_rustls;
extern crate scoped_threadpool;
extern crate docopt;

use docopt::Docopt;
use std::process;

mod https;
mod twitch;

/* Exit Codes:
1: Missing arguments
2: Invalid arguments
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