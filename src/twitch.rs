use std::f32;
use std::io::Read;
use std::io::Write;
use std::io::stdout;
use std::fs;
use std::fs::File;
use std::process::Command;
use std::process::Stdio;
use std::time::Instant;

use rustc_serialize::json::Json;
use scoped_threadpool::Pool;

use https;

fn m3u8_to_vector(data: String) -> Vec<String> {
	let m3u8_split = data.split("\n");

	let mut vec = Vec::new();

	for line in m3u8_split {
		if line != "" && line.chars().nth(0).unwrap() != '#' {
			vec.push(String::from(line));
		}
	}
	return vec;
}

fn vod_m3u8(vod: &str) -> String {
	let auth_raw = https::to_string(format!("https://api.twitch.tv/api/vods/{}/access_token",vod),true);
	let auth_json = Json::from_str(&*auth_raw).unwrap();
	let auth = auth_json.as_object().unwrap();

	let mut nauth = format!("{}",auth["token"].as_string().unwrap());
	nauth = str::replace(&*nauth,"\\","");

//	Because of the provided https certificate, use ttvnw.net to prevent errors.
	let mut url = format!("https://usher.ttvnw.net/vod/{}?nauthsig={}&nauth={}&allow_source=true&player=twitchweb&allow_spectre=true&allow_audio_only=true", vod, auth["sig"], nauth);

// Really lazy way of removing quotation characters.
	url = str::replace(&*url, "sig=\"", "sig=");
	url = str::replace(&*url, "\"&nauth=", "&nauth=");
	url = str::replace(&*url, "\"&allow","&allow");

	return url;
}

fn download_array(input: Vec<String>) -> usize {
	let start = Instant::now();
	let mut count: usize = 0;
	let divisor = input.len();

	print!("0% complete");
//	The number here is threads to use.
	let mut f = File::create("toffmpeg.txt").unwrap();
//	This averages ~12Mb/s on quality low
	let mut pool = Pool::new(4);

	pool.scoped(|scope| {
		for url in input {
			count += 1;
			let _ = f.write_all(format!("file 'chunk_{}.ts'\n",count).as_bytes());
			scope.execute(move || {
				let name = format!("chunk_{}.ts",count);
				let _ = https::to_file(url,name);

				print!("\r{}% downloaded - {} seconds remaining                           ",
					(count*100)/divisor,
				//	There's probably a better way than to cast f32 every single time.
					((start.elapsed().as_secs() as f32 / count as f32) * (divisor-count) as f32) as usize
				);
			});
		}
	});
	//There are some cases where the 100% dialog is overwritten by a 99% dialog.
	//This is a cosmetic fix, as by the time we reach this we're 100% done anyways.
	println!("\r100% downloaded                                             ");

	return count;
}

//	https://msdn.microsoft.com/en-us/library/aa365247
fn sanitize_filename(input: &str) -> String {
	let invalid: [&str; 9] = [">","<",":","\"","/","\\","|","?","*"];
	let mut output: String = String::from(input);
	for n in 0..9 {
		output = str::replace(&*output, invalid[n], "_");
	}
	return output;
}

pub fn get_vod(vod: &str, quality: usize) {

	let info_raw = https::to_string(format!("https://api.twitch.tv/kraken/videos/{}",vod),true);
	let info_json = Json::from_str(&*info_raw).unwrap();
	let info = info_json.as_object().unwrap();

//	I'm like 80% sure that highlight and archive have the same m3u8 file.
	let quality_list: [&str; 6] = match info["broadcast_type"].as_string().unwrap() {
		"archive" 	=> ["chunked","high","medium","low","mobile","audio_only"],
		"highlight"	=> ["chunked","high","medium","low","mobile","audio_only"],
		"upload" 	=> ["720p","480p","360p","240p","144p","audio_only"],
		_ => { panic!("This shouldn't happen -> quality_list match"); },
	};

	let filename = format!("{}.mp4",sanitize_filename(info["title"].as_string().unwrap()));

	println!("Preparing to download '{}'",filename);

	let word: &str = quality_list[quality];
	let word_len = word.chars().count()+1;

	let m3u8_raw = https::to_string(vod_m3u8(vod),false);
	let m3u8_index = m3u8_to_vector(m3u8_raw);

	let vod_url = m3u8_index[quality].clone();
	let offset = vod_url.find(word).unwrap()+word_len;
	let vod_base = &vod_url[..offset];

	let vod_raw = https::to_string(vod_url.clone(),false);
	let vod_iter = vod_raw.split("\n");

	let mut vod_data = Vec::new();
	for line in vod_iter {
		if line != "" && line.chars().nth(0).unwrap() != '#' {
			vod_data.push(format!("{}{}",vod_base,line));
		}
	}

	//Add one because of the for loop
	let count = download_array(vod_data)+1;

//	https://github.com/rust-lang/rust/issues/30098
	let child = Command::new("ffmpeg")
		.args(&["-loglevel","panic","-hide_banner","-stats","-f","concat","-i","toffmpeg.txt","-c","copy","-bsf:a","aac_adtstoasc"])
		.arg(&filename)
		.stdout(Stdio::piped())
		.spawn()
		.unwrap_or_else(|e| { panic!("failed to execute process: {}", e) });
	let mut out = child.stdout.unwrap();
	let mut read_buf = [0u8; 64];
//	let mut out_buf: Vec<u8> = Vec::new();
	while let Ok(size) = out.read(&mut read_buf) {
		if size == 0 {
			break;
		}
		stdout().write_all(&read_buf).unwrap();
//		out_buf.extend(read_buf.iter());
	}

	for f in 1..count {
		let _ = fs::remove_file(format!("chunk_{}.ts",f));
	}
	let _ = fs::remove_file("toffmpeg.txt");

	println!("Finished! Output file: '{}'",filename);
}