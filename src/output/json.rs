use std::io::Write;

use crate::duplicate_finder::Duplicates;
use crate::Outputter;

pub struct JsonOutputter {}

impl Outputter for JsonOutputter {
    fn output(&self, _stream: &mut dyn Write, _d : &Duplicates) -> () {
       
    }
}
