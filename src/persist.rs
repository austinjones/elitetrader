use rustc_serialize::*;

use std::io::Write;
use std::fs::File;
use std::path::Path;
use std::io::Read;

use flate2::read::GzDecoder;

use hyper::Client;
use hyper::header::*;

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

pub fn http_read_json<T: Decodable>( url: &String ) -> T {
	let mut client = Client::new();
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
	
    let mut decoder = match GzDecoder::new(bytes.as_slice()) {
    	Ok(v) => v,
    	Err(reason) => panic!("Failed to initalize Gzip decoder for HTTP download: {}", reason)
    };
    
    match decoder.read_to_string( &mut body ) {
    	Err(reason) => panic!("Failed to unzip contents: {}", reason),
    	_ => {}
    };
    
    match json::decode(&body) {
		Ok(result) => result,
		Err(reason) => panic!("Failed to parse response from URL {}, reason: {}", url, reason)
	}
}