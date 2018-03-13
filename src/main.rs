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
use std::fs::OpenOptions;

fn download(segment: &Segment, usenet: &mut NNTPStream) -> NNTPResult<Vec<u8>> {
    let bytes = usenet.body_by_id_unknown_bytes(&format!("<{}>", &segment.article_id))?;
    Ok(bytes)
}

fn main() {
    env_logger::init();

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
    nzb_files.par_iter().map(|file| {
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
                    Ok(data) => {
                        out_bytes.push(data);
                        break;
                    }
                    Err(e) => {
                        info!("Error downloading segment '{}' using group '{}'", segment.article_id, group);
                        if idx == file.groups.len() {
                            bail!("Failed to download segment {}: {:?}", segment.article_id, e)
                        }
                        bail!("Failed to download segment {}: {:?}", segment.article_id, e)
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
            (filename, data.into_iter().flat_map(decode).collect::<Vec<u8>>())
        })
        .map(|(filename, data)| {
            let mut file_out = OpenOptions::new()
                .write(true)
                .create(true)
                .open("download/".to_string() + &filename).unwrap();
            file_out.write_all(&data).unwrap();
        })
        .collect::<()>();
}

fn decode(data: Vec<u8>) -> Vec<u8> {
    let mut start = 0;
    while !&data[start..start+2].starts_with(b"=y") {
        start += &data[start..].iter().position(|byte| byte == &b'\n')
            .expect("No newline in file") + 1;
    }
    for _ in 0..2 {
        start += &data[start..].iter().position(|byte| byte == &b'\n')
            .expect("No newline in file") + 1;
    }

    let end = data.iter().rposition(|byte| byte == &b'\r')
        .expect("No newline in file");
    debug!("Firstline: {:?}", String::from_utf8(data[..start].to_owned()).unwrap());
    debug!("Lastline: {:?}", String::from_utf8(data[end..].to_owned()).unwrap());
    yenc::ydecode_buffer(&data[start+1..end]).unwrap()
}
