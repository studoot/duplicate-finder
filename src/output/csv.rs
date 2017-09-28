use std::io::Write;

use duplicate_finder::Duplicates;
use ::Outputter;

pub struct CsvOutputter {}

impl Outputter for CsvOutputter {
    fn output(&self, stream: &mut Write, d : &Duplicates) -> () {

    }
}
