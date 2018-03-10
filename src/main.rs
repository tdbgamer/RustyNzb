extern crate rustynzb;
extern crate nntp;
extern crate env_logger;

use std::fs::File;
use std::io::BufReader;

use rustynzb::parser::entities::*;
use rustynzb::parser::parse_nzb;
use nntp::{NNTPStream, NNTPResult};

fn download(segment: &Segment, usenet: &mut NNTPStream) -> NNTPResult<()> {
    let article = usenet.article_by_id(&format!("<{}>", &segment.article_id))?;
    println!("{:?}", article.body);
    Ok(())
}

fn main() {
    env_logger::init();

    let mut usenet = NNTPStream::connect(("us.bintube.com", 443)).unwrap();
    usenet.login("", "").unwrap();
    let filename = std::env::args().nth(1).unwrap();
    let file = match File::open(filename) {
        Ok(file) => file,
        Err(e) => { return rustynzb::errors::exit_with_error(e); }
    };
    let mut file = BufReader::new(file);
    let mut nzb_files = match parse_nzb(&mut file) {
        Ok(files) => files,
        Err(e) => { return rustynzb::errors::exit_with_error(e); }
    };
    for file in &nzb_files {
        for segment in &file.segments {
            for group in &file.groups {
                println!("Trying group: {}", group);
                match download(&segment, &mut usenet) {
                    Ok(_) => { break; }
                    Err(e) => {println!("{:?}", e)}
                }
            }
        }
    }
}
