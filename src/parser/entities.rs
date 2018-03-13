use std;
use super::RustyNzbResult;

macro_rules! give_some {
    ($to_replace:expr, $replacement:expr) => {
        ::std::mem::replace($to_replace, Some($replacement));
    };
}


#[derive(Debug)]
pub struct Segment {
    pub bytes: usize,
    pub number: u32,
    pub article_id: String,
}

impl Segment {
    pub fn new(bytes: usize, number: u32, article_id: String) -> Self {
        Segment {
            bytes,
            number,
            article_id: article_id,
        }
    }
}

#[derive(Debug)]
pub struct NzbFile {
    pub filename: String,
    pub segments: Vec<Segment>,
    pub groups: Vec<String>,
}

impl NzbFile {
    pub fn new(filename: String, groups: Vec<String>, segments: Vec<Segment>) -> Self {
        NzbFile {
            filename: filename,
            segments: segments,
            groups: groups
        }
    }
}

#[derive(Default)]
pub struct NzbFileBuilder {
    filename: Option<String>,
    segments: Vec<Segment>,
    groups: Vec<String>,
}

impl NzbFileBuilder {
    pub fn set_filename<T>(&mut self, filename: T) -> &mut Self
        where T: Into<String> {
        give_some!(&mut self.filename, filename.into());
        self
    }

    pub fn add_group<T>(&mut self, group: T) -> &mut Self
        where T: Into<String> {
        self.groups.push(group.into());
        self
    }

    pub fn add_segment(&mut self, segment: Segment) -> &mut Self {
        self.segments.push(segment.into());
        self
    }

    pub fn clear(&mut self) {
        self.filename = None;
        self.groups.clear();
        self.segments.clear();
    }

    pub fn create(&mut self) -> RustyNzbResult<NzbFile> {
        if self.filename.is_none() {
            bail!("NzbFileBuilder requires a filename to be set.");
        }
        if self.groups.is_empty() {
            bail!("NzbFileBuilder requires a group to be set.");
        }
        let mut old_segments = Vec::new();
        let mut old_groups = Vec::new();
        std::mem::swap(&mut old_segments, &mut self.segments);
        std::mem::swap(&mut old_groups, &mut self.groups);
        Ok(NzbFile::new(self.filename.take().unwrap(),
                        old_groups, old_segments))
    }
}

#[derive(Default)]
pub struct SegmentBuilder {
    bytes: Option<usize>,
    number: Option<u32>,
    article_id: Option<String>,
}

impl SegmentBuilder {
    pub fn set_bytes(&mut self, bytes: usize) -> &mut Self {
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
