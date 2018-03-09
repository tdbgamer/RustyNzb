extern crate log;
extern crate quick_xml;
#[macro_use]
extern crate failure;

pub mod errors;

use errors::{RustyNzbResult, ResultExt};

use std::io::BufRead;
use std::borrow::Cow;

use quick_xml::reader::Reader;
use quick_xml::events::Event;

pub struct Segment {
    bytes: u32,
    number: u32,
    article_id: String,
}

impl Segment {
    pub fn new<T>(bytes: u32, number: u32, article_id: T) -> Self
        where T: Into<String> {
        Segment {
            bytes,
            number,
            article_id: article_id.into(),
        }
    }
}

pub struct NzbFile {
    filename: String,
    segments: Vec<Segment>,
}

impl NzbFile {
    pub fn new<T>(filename: T, segments: Vec<Segment>) -> Self
        where T: Into<String> {
        NzbFile {
            filename: filename.into(),
            segments: segments,
        }
    }
}

#[derive(Default)]
pub struct NzbFileBuilder {
    filename: Option<String>,
    segments: Vec<Segment>,
}

impl NzbFileBuilder {
    pub fn set_filename<T>(&mut self, mut filename: T)
        where T: Into<String> {
        std::mem::replace(&mut self.filename, Some(filename.into()));
    }

    pub fn add_segment<T>(&mut self, segment: Segment) {
        self.segments.push(segment.into());
    }

    pub fn clear(&mut self) {
        self.filename = None;
        self.segments.clear();
    }

    pub fn create(&mut self) -> RustyNzbResult<NzbFile> {
        if self.filename.is_none() {
            bail!("NzbFileBuild requires a filename to be set.");
        }
        let mut old_segments = Vec::new();
        std::mem::swap(&mut old_segments, &mut self.segments);
        Ok(NzbFile::new(self.filename.take().unwrap(),
                        old_segments))
    }
}

fn parse_nzb(filename: &mut BufRead) -> RustyNzbResult<Vec<NzbFile>> {
    let mut reader = Reader::from_reader(filename);
    let mut buf = Vec::new();
    let mut nzb_builder = NzbFileBuilder::default();
    let mut files = Vec::new();
    loop {
        match reader.read_event(&mut buf) {
            Ok(Event::Start(ref e)) => {
                match e.name() {
                    b"file" => {
                        for attr in e.attributes() {
                            let attr = attr.sync()?;
                            if attr.key == b"subject" {
                                let subject = attr.unescaped_value().sync()?;
                                let subject = String::from_utf8_lossy(&subject);
                                let filename = subject.split("\"").nth(1);
                                if let Some(file) = filename {
                                    nzb_builder.set_filename(String::from(file));
                                } else {
                                    bail!("Filename not set in tag at: {}", reader.buffer_position());
                                }
                            }
                        }
                    },
                    b"segment" => {},
                    _ => {},
                }
            },
            Ok(Event::End(ref e)) => {
                match e.name() {
                    b"file" => {
                        files.push(nzb_builder.create()?);
                    },
                    _ => {},
                }
            },
            Ok(Event::Eof) => { break; },
            Err(e) => panic!("Error at position {}: {:?}", reader.buffer_position(), e),
            _ => (),
        }
        buf.clear();
    }
    Ok(files)
}