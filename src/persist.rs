use rustc_serialize::*;
use csv;

use std::io::Write;
use std::fs::File;
use std::fs::create_dir_all;
use std::path::Path;
use std::path::PathBuf;
use std::io::Read;

use std::env::home_dir;

use flate2::read::GzDecoder;

use hyper::Client;
use hyper::header::*;
use std::time::Duration;


pub fn get_base_directory() -> PathBuf {
	match home_dir() {
		Some(home_path) => {
			let ed_path = home_path.join(".elite_trader" );
			match create_dir_all( &ed_path ) {
				Ok(_) => {},
				Err(_) => panic!("Failed to create base directory")
			}
			ed_path.to_path_buf()
		},
		None => Path::new(".").to_path_buf()
	}
}

pub fn read_json<T: Decodable>( path: &Path ) -> T {
	let mut file = File::open( path ).unwrap();
//	println!("Reading file {}", path.to_str().unwrap() );
	
	let mut s = String::new();
	file.read_to_string(&mut s).unwrap();
	
//	println!("Decoding file {}", path.to_str().unwrap() );
	match json::decode(&s) {
		Ok(result) => result,
		Err(reason) => panic!("Failed to parse file {}, reason: {}", path.to_str().unwrap(), reason)
	}
}

pub fn read_text_from_file( file: &mut File ) -> String {
	let mut s = String::new();
	file.read_to_string(&mut s).unwrap();
	
	s
}

pub fn read_json_from_file<T: Decodable>( file: &mut File ) -> T {
	let mut s = String::new();
	file.read_to_string(&mut s).unwrap();
	
//	println!("Decoding file {}", path.to_str().unwrap() );
	match json::decode(&s) {
		Ok(result) => result,
		Err(reason) => panic!("Failed to parse file, reason: {}", reason)
	}
}

pub fn write_json<T: Encodable>( path: &Path, data: &T ) {
	let mut file = match File::create( path ) {
		Ok(file) => file,
		Err(reason) => panic!( "Failed to create file {}, reason: {}", path.to_str().unwrap(), reason )
	};
	
//	println!("Encoding file {}", path.to_str().unwrap() );
	
	let string = json::encode( data ).unwrap();
	let bytes : &[u8] = string.as_bytes();
	
//	println!("Writing file {}", path.to_str().unwrap() );
	match file.write_all(bytes) {
		Err(reason) => panic!( "Failed to write file {}, reason: {}", path.to_str().unwrap(), reason ),
		_ => {}
	}; 
}

fn http_read(url: &String) -> String {
	let mut client = Client::new();
	client.set_read_timeout(Some(Duration::new(600,0)));
	client.set_write_timeout(Some(Duration::new(600,0)));
	let mut res = client.get(url)
		.header(
			AcceptEncoding(
				vec!(
					QualityItem::new(Encoding::Gzip, Quality(1000))
				)
			)
		).send().unwrap();
        
    
    let mut bytes = Vec::new();
	match res.read_to_end(&mut bytes) {
		Ok(_) => {},
		Err(reason) => { panic!("Failed to load URL {}, reason: {}", url, reason) }
	}
	
	let mut body = String::new();
	
    let mut decoder = match GzDecoder::new(&bytes[..]) {
    	Ok(v) => v,
    	Err(reason) => panic!("Failed to initalize Gzip decoder for HTTP download: {}", reason)
    };
    
    match decoder.read_to_string( &mut body ) {
    	Err(reason) => panic!("Failed to unzip contents: {}", reason),
    	_ => {}
    }
    
    body
}

pub fn http_read_json<T: Decodable>( url: &String ) -> T {
	let body = http_read( url );
    
    match json::decode(&body) {
		Ok(result) => result,
		Err(reason) => panic!("Failed to parse response from URL {}, reason: {}", url, reason)
	}
}

pub fn http_read_csv<T: Decodable>( url: &String, has_headers: bool ) -> Vec<T> {
	let body = http_read( url );
	
    let mut rdr = csv::Reader::from_string(body).has_headers( has_headers );
	rdr.decode()
		.filter(|e| e.is_ok() )
		.map(|e| e.unwrap() )
		.collect::<Vec<T>>()
}