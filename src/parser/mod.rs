use std::io::BufRead;
use quick_xml::reader::Reader;
use quick_xml::events::Event;
use self::entities::{NzbFile, NzbFileBuilder, SegmentBuilder};
pub use errors::{RustyNzbResult, ResultExt};

pub mod entities;

pub fn parse_nzb(file: &mut BufRead) -> RustyNzbResult<Vec<NzbFile>> {
    let mut reader = Reader::from_reader(file);
    let mut buf = Vec::new();
    let mut nzb_builder = NzbFileBuilder::default();
    let mut segment_builder = SegmentBuilder::default();
    let mut files = Vec::new();
    let mut in_group = false;
    loop {
        match reader.read_event(&mut buf) {
            Ok(Event::Start(ref e)) => {
                debug!("Start tag: {}", &String::from_utf8_lossy(e.name()));
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
                        for attr in e.attributes() {
                            let attr = attr.sync()?;
                            match attr.key {
                                b"bytes" => {
                                    let bytes = attr.unescaped_value().sync()?;
                                    let bytes = String::from_utf8_lossy(&bytes);
                                    if let Ok(bytes) = bytes.parse::<usize>() {
                                        segment_builder.set_bytes(bytes);
                                    }
                                }
                                b"number" => {
                                    let number = attr.unescaped_value().sync()?;
                                    let number = String::from_utf8_lossy(&number);
                                    if let Ok(number) = number.parse::<u32>() {
                                        segment_builder.set_number(number);
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                    b"group" => {
                        in_group = true;
                    }
                    _ => {}
                }
            }
            Ok(Event::Text(ref e)) => {
                debug!("Text: {}", &String::from_utf8_lossy(&e.unescaped().sync()?));
                if !segment_builder.is_empty() {
                    debug!("Setting segment article id");
                    let article_id = e.unescaped().sync()?;
                    segment_builder.set_article_id(String::from_utf8_lossy(&article_id));
                } else if in_group {
                    debug!("Adding group");
                    nzb_builder.add_group(String::from_utf8_lossy(&e.unescaped().sync()?));
                }
            }
            Ok(Event::End(ref e)) => {
                debug!("End Tag: {}", &String::from_utf8_lossy(e.name()));
                match e.name() {
                    b"file" => {
                        files.push(nzb_builder.create()?);
                    }
                    b"segment" => {
                        nzb_builder.add_segment(segment_builder.create()?);
                    }
                    b"group" => {
                        in_group = false;
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
