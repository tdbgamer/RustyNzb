use std;
use super::RustyNzbResult;

macro_rules! give_some {
    ($to_replace:expr, $replacement:expr) => {
        ::std::mem::replace($to_replace, Some($replacement));
    };
}


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
    pub fn set_filename<T>(&mut self, filename: T) -> &mut Self
        where T: Into<String> {
        give_some!(&mut self.filename, filename.into());
        self
    }

    pub fn add_segment(&mut self, segment: Segment) -> &mut Self {
        self.segments.push(segment.into());
        self
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

#[derive(Default)]
pub struct SegmentBuilder {
    bytes: Option<u32>,
    number: Option<u32>,
    article_id: Option<String>,
}

impl SegmentBuilder {
    pub fn set_bytes(&mut self, bytes: u32) -> &mut Self {
        give_some!(&mut self.bytes, bytes);
        self
    }

    pub fn set_number(&mut self, number: u32) -> &mut Self {
        give_some!(&mut self.number, number);
        self
    }

    pub fn set_article_id<T>(&mut self, article_id: T) -> &mut Self
        where T: Into<String> {
        give_some!(&mut self.article_id, article_id.into());
        self
    }

    pub fn clear(&mut self) {
        self.bytes = None;
        self.number = None;
        self.article_id = None;
    }

    pub fn is_empty(&self) -> bool {
        match (&self.bytes, &self.number, &self.article_id) {
            (&None, &None, &None) => true,
            _ => false
        }
    }

    pub fn create(&mut self) -> RustyNzbResult<Segment> {
        match (&self.bytes, &self.number, &self.article_id) {
            (&Some(..), &Some(..), &Some(..)) => {}
            _ => { bail!("Bytes, number, and article_id are required to create a Segment"); }
        };
        Ok(Segment::new(self.bytes.take().unwrap(),
                        self.number.take().unwrap(),
                        self.article_id.take().unwrap()))
    }
}
