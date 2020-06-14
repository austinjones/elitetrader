use csv::{self, Reader};

use std::fs::create_dir_all;
use std::fs::File;
use std::io::Read;
use std::io::{Cursor, Write};
use std::path::Path;
use std::path::PathBuf;

use std::env::home_dir;

use serde::de::DeserializeOwned;
use serde::Serialize;

pub fn get_base_directory() -> PathBuf {
    match home_dir() {
        Some(home_path) => {
            let ed_path = home_path.join(".elite_trader");
            match create_dir_all(&ed_path) {
                Ok(_) => {}
                Err(_) => panic!("Failed to create base directory"),
            }
            ed_path.to_path_buf()
        }
        None => Path::new(".").to_path_buf(),
    }
}

pub fn read_json<T: DeserializeOwned>(path: &Path) -> T {
    let mut file = File::open(path).unwrap();

    let mut s = String::new();
    file.read_to_string(&mut s).unwrap();

    match serde_json::from_str(&s) {
        Ok(result) => result,
        Err(reason) => panic!(
            "Failed to parse file {}, reason: {}",
            path.to_str().unwrap(),
            reason
        ),
    }
}

pub fn read_text_from_file(file: &mut File) -> String {
    let mut s = String::new();
    file.read_to_string(&mut s).unwrap();

    s
}

pub fn read_json_from_file<T>(file: &mut File) -> T
where
    T: DeserializeOwned,
{
    let mut s = String::new();
    file.read_to_string(&mut s).unwrap();

    match serde_json::from_str(&s) {
        Ok(result) => result,
        Err(reason) => panic!("Failed to parse file, reason: {}", reason),
    }
}

pub fn write_json<T>(path: &Path, data: &T)
where
    T: Serialize,
{
    let mut file = match File::create(path) {
        Ok(file) => file,
        Err(reason) => panic!(
            "Failed to create file {}, reason: {}",
            path.to_str().unwrap(),
            reason
        ),
    };

    let string = serde_json::to_string(data).unwrap();
    let bytes: &[u8] = string.as_bytes();

    match file.write_all(bytes) {
        Err(reason) => panic!(
            "Failed to write file {}, reason: {}",
            path.to_str().unwrap(),
            reason
        ),
        _ => {}
    };
}

fn http_read(url: &String) -> String {
    let client = reqwest::blocking::Client::new();
    let resp = client
        .get(url)
        .header("Accept-Encoding", "gzip, deflate, sdch")
        .send()
        .unwrap();

    resp.text().unwrap()
}

pub fn http_read_json<T: DeserializeOwned>(url: &String) -> T {
    let body = http_read(url);

    match serde_json::from_str(&body) {
        Ok(result) => result,
        Err(reason) => panic!(
            "Failed to parse response from URL {}, reason: {}",
            url, reason
        ),
    }
}

pub fn http_read_csv<T>(url: &String) -> Vec<T>
where
    T: DeserializeOwned,
{
    let body = http_read(url);

    let mut rdr = Reader::from_reader(Cursor::new(body));

    rdr.deserialize().map(|e| e.unwrap()).collect::<Vec<T>>()
}
