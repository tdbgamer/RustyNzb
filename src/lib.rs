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

#[derive(Debug)]
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

#[derive(Debug)]
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
    pub fn set_filename<T>(&mut self, filename: T)
        where T: Into<String> {
        std::mem::replace(&mut self.filename, Some(filename.into()));
    }

    pub fn add_segment(&mut self, segment: Segment) {
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

pub fn parse_nzb(file: &mut BufRead) -> RustyNzbResult<Vec<NzbFile>> {
    let mut reader = Reader::from_reader(file);
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
                            match attr.key {
                                b"subject" => {
                                    let subject = attr.unescaped_value().sync()?;
                                    let subject = String::from_utf8_lossy(&subject);
                                    let filename = subject.split("\"").nth(1);
                                    if let Some(file) = filename {
                                        nzb_builder.set_filename(String::from(file));
                                    } else {
                                        bail!("Filename not set in tag at: {}", reader.buffer_position());
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                    b"segment" => {
                        let mut seg_bytes: Option<u32> = None;
                        let mut seg_number: Option<u32> = None;
                        let mut seg_article_id: Option<String> = None;
                        for attr in e.attributes() {
                            let attr = attr.sync()?;
                            match attr.key {
                                b"bytes" => {
                                    let bytes = attr.unescaped_value().sync()?;
                                    let bytes = String::from_utf8_lossy(&bytes);
                                    if let Ok(bytes) = bytes.parse::<u32>() {
                                        std::mem::replace(&mut seg_bytes, Some(bytes));
                                    }
                                }
                                b"number" => {
                                    let number = attr.unescaped_value().sync()?;
                                    let number = String::from_utf8_lossy(&number);
                                    if let Ok(number) = number.parse::<u32>() {
                                        std::mem::replace(&mut seg_number, Some(number));
                                    }
                                }
                                _ => {}
                            }
                        }
                        if let Ok(article_id) = e.unescape_and_decode(&reader) {
                            std::mem::replace(&mut seg_article_id, Some(article_id));
                        }

                        match (seg_bytes, seg_number, seg_article_id) {
                            (Some(bytes), Some(number), Some(article_id)) => {
                                nzb_builder.add_segment(Segment::new(bytes, number, article_id));
                            }
                            _ => {
                                bail!("Invalid segment reached at: {}", reader.buffer_position());
                            }
                        }
                    }
                    _ => {}
                }
            }
            Ok(Event::End(ref e)) => {
                match e.name() {
                    b"file" => {
                        files.push(nzb_builder.create()?);
                    }
                    _ => {}
                }
            }
            Ok(Event::Eof) => { break; }
            Err(e) => panic!("Error at position {}: {:?}", reader.buffer_position(), e),
            _ => (),
        }
        buf.clear();
    }
    Ok(files)
}