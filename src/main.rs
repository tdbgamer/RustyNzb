extern crate rustynzb;

use std::fs::File;
use std::io::BufReader;

use rustynzb::parser::parse_nzb;

fn main() {
    let filename = std::env::args().nth(1).unwrap();
    let file = match File::open(filename) {
        Ok(file) => file,
        Err(e) => { return rustynzb::errors::exit_with_error(e); }
    };
    let mut file = BufReader::new(file);
    let nzb_files = match parse_nzb(&mut file) {
        Ok(files) => files,
        Err(e) => { return rustynzb::errors::exit_with_error(e); }
    };
    println!("{:?}", nzb_files);
}
