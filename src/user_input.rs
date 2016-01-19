use std::str::FromStr;
use std::io::stdin;

pub fn prompt_confirm( message: &str ) -> bool {
	println!( "{} (y/n): ", message );
	
	let val = read_line();
	
	println!("");
	
	match &val.to_lowercase()[..] {
		"y" | "yes" => true,
		"n" | "no" => false,
		_ => prompt_confirm( message )
	}
}

pub fn prompt_value( flag: &str, description: &str ) -> String {
	println!( "Please provide the flag -{}, or enter the {} now: ", flag, description );
	
	let val = read_line();
	
	println!("");
	
	val
}

pub fn read_line() -> String {
	let mut str = String::new();
	match stdin().read_line(&mut str) {
		Err(reason) => panic!("Failed to read line: {}", reason ),
		_ => {}
	};
	str.trim().to_string()
}

pub fn read_price_update<T: FromStr>( description: &str ) -> T {
	println!("Enter the updated {}:", description );
	
	let line = read_line();
	let val = match T::from_str( &line[..] ) {
		Ok(price) => price,
		Err(_) => {
			println!("Failed to parse answer '{}'.  Please try again.", line);
			read_price_update( description )
		}
	};
	
	println!("");
	val
}