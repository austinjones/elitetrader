extern crate time;

use std::fs::create_dir_all;
use std::fs::read_dir;
use std::path::Path;
use std::path::PathBuf;
use rustc_serialize::Encodable;
use rustc_serialize::Decodable;
use time::get_time;

use persist::*;

// there is a big problem with the design of the adjustment system.
// the adjustments are applied as patches to the Universe
// this is nice to apply, but has two performance downsides
// 1) applying a patch requires enumerating the entire Universe system array - O(n^2) performance
// 2) since the Station struct holds the station time adjustment data, 
//		we need to reindex the entire universe after each adjustment
//		we cannot reindex individual systems because the Octree cannot update or replace items


fn get_adjustments_dir() -> PathBuf {
	let basedir = get_base_directory();
	let adjustment_path = basedir.join( Path::new( "adjustments" ) );
	
	match create_dir_all(&adjustment_path) {
		Ok(_) => {},
		Err(reason) => panic!("Failed to create adjustments dir {}: {}", 
			adjustment_path.to_str().unwrap_or("<unknown>"),
			reason 
		)
	}
	
	adjustment_path
}

// todo: make category an enum?
// this would allow callers to save/load adjustments using the generic method
// or maybe we could create a trait.. hmm...
pub fn save_adjustment<A: Encodable>( category: &str, adjustment: &A ) {
	let timespec = get_time();
	let time_str = format!("{:010}-{:09}", timespec.sec, timespec.nsec);
	
	let filename = format!("adjustment_{}.{}.json", category, time_str);
	let filepath = get_adjustments_dir().join( filename );
	
	write_json( &filepath, adjustment );
}

struct LoadedAdjustment<T> {
	filename: String,
	adjustment: T
}

pub fn load_adjustments<A: Decodable>( category: &str ) -> Vec<A> {
	let mut vec = Vec::new();
	
	let prefix_string = "adjustment_".to_string() + category + ".";
	let prefix = &prefix_string[..];
	
	match read_dir( get_adjustments_dir() ) {
		Ok(results) => {
			for entry in results {
				let entry = match entry {
					Ok(o) => o,
					Err(_) => { 
						continue; 
					}
				};
				
				let path = entry.path();
				let filename = match path.file_name() {
					Some(f) => match f.to_str() {
						Some(f2) => f2.to_string(),
						None => { 
							continue; 
						}
					},
					None => { 
						continue; 
					}
				};
				
				if !filename.starts_with(prefix) {
					continue;
				}
				
				let val : A = read_json( path.as_path() );
				vec.push( LoadedAdjustment{ filename: filename, adjustment: val } );
			}
		},
		Err(reason) => panic!("Failed to read adjustments dir {}: {}", 
			get_adjustments_dir().as_path().to_str().unwrap_or("<unknown>"),
			reason 
		)
	}
	
	vec.sort_by(|a,b| a.filename.partial_cmp(&b.filename).expect("Failed to compare adjustment filename order") );
	// workaround for a lifetime problem
	let newvec = vec.drain(..).map(|e| e.adjustment).collect();
	newvec
}