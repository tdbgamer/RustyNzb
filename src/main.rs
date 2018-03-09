extern crate rustynzb;

use rustynzb::{NzbFileBuilder, NzbFile};
use rustynzb::errors;

fn main() {
    let mut builder = NzbFileBuilder::default();
    builder.set_filename("file1");
    match builder.create() {
        Ok(_) => {},
        Err(e) => errors::exit_with_error(e),
    };
}
