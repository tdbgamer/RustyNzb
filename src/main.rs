extern crate rustynzb;
extern crate nntp;
extern crate env_logger;
extern crate yenc;
extern crate rayon;
#[macro_use]
extern crate failure;
#[macro_use]
extern crate log;

use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;

use rayon::prelude::*;
use rustynzb::parser::entities::*;
use rustynzb::parser::parse_nzb;
use nntp::{NNTPStream, NNTPResult};

fn download(segment: &Segment, usenet: &mut NNTPStream) -> NNTPResult<Vec<u8>> {
    let bytes = usenet.body_by_id_bytes(&format!("<{}>", &segment.article_id))?;
    Ok(bytes)
}

fn main() {
    env_logger::init();

    let mut usenet = NNTPStream::connect(("us.bintube.com", 443)).unwrap();
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
    nzb_files.iter().map(|file| {
        let mut usenet = NNTPStream::connect(("us.bintube.com", 443)).unwrap();
        usenet.login(env!("USENETUSER"), env!("USENETPASS")).unwrap();
        let mut out_bytes = Vec::new();
        let mut current_group = "NONE";
        for segment in &file.segments {
            for (idx, group) in file.groups.iter().enumerate() {
                if group != current_group {
                    usenet.group(&group)
                        .expect("Failed to change group");
                    current_group = &group;
                }
                match download(&segment, &mut usenet) {
                    Ok(ref mut data) => {
                        out_bytes.append(data);
                        break;
                    }
                    Err(e) => {
                        info!("Error downloading segment '{}' using group '{}'", segment.article_id, group);
                        if idx == file.groups.len() {
                            bail!("Failed to download segment {}: {:?}", segment.article_id, e)
                        }
                    }
                };
            }
        }
        usenet.quit()
            .expect("Failed to QUIT NNTP connection");
        Ok((&file.filename, out_bytes))
        })
        .filter(|res| {
            match *res {
                Ok(_) => true,
                Err(ref e) => {
                    warn!("{:?}", e);
                    false
                }
            }
        })
        .map(|elm| elm.unwrap())
        .map(|(filename, data)| {
            (filename, yenc::ydecode_buffer(&data).unwrap())
        })
        .map(|(filename, data)| {
            let mut file_out = File::create("download/".to_string() + &filename).unwrap();
            file_out.write_all(&data).unwrap();
        })
        .collect::<()>();
}
