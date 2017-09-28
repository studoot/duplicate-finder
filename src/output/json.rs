use std::io::Write;

use duplicate_finder::Duplicates;
use ::Outputter;

pub struct JsonOutputter {}

impl Outputter for JsonOutputter {
    fn output(&self, stream: &mut Write, d : &Duplicates) -> () {
       
    }
}
