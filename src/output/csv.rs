use std::io::Write;

use crate::duplicate_finder::Duplicates;
use crate::Outputter;

pub struct CsvOutputter {}

impl Outputter for CsvOutputter {
    fn output(&self, _stream: &mut dyn Write, _d : &Duplicates) -> () {

    }
}
