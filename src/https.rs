use hyper::Client;
use hyper::header::Headers;
use hyper::net::HttpsConnector;
use hyper::status::StatusCode;

use hyper_rustls::TlsClient;

use std::io::prelude::*;
use std::io::Error;
use std::fs::File;

const CLIENT_ID: &'static str = "y7c66dozeuhufau5a1p3xs8n0axiok";

header! { (ClientID, "Client-ID") => [String] }
header! { (Accept, "Accept") => [String] }


pub fn to_string(url: String,twitch: bool) -> String {
	let client = Client::with_connector(HttpsConnector::new(TlsClient::new()));

	let mut headers = Headers::new();
	if twitch == true {
		headers.set(Accept("application/vnd.twitchtv.v5+json".to_string()));
		headers.set(ClientID(CLIENT_ID.to_string()));
	}

	let mut res = client.get(&*url)
		.headers(headers)
		.send().unwrap();

	let mut buffer = String::new();
	let _ = res.read_to_string(&mut buffer);

	if res.status == StatusCode::Ok {
		return buffer;
	} else {
		//Todo: Learn how to do proper error handling.
		println!("{}",buffer);
		panic!(format!("Error in https_request: {}",res.status));
	}
}

pub fn to_file(url: String, name: String) -> Result<(), Error> {
	//http://stackoverflow.com/a/41451006/1687505
	let client = Client::with_connector(HttpsConnector::new(TlsClient::new()));
	let mut res = client.get(&*url).send().unwrap();
	let mut file = try!(File::create(name));

	let mut buf = [0; 128 * 1024];
	loop {
		let len = match res.read(&mut buf) {
			Ok(0) => break, //End of file reached.
			Ok(len) => len,
			Err(err) => panic!(format!("Error in https_save: {}",err)),
		};
		try!(file.write_all(&buf[..len]));
	}
	Ok(())
}
