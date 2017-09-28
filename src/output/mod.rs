use std::fmt;
use std::io::Write;
use std::result::Result;
use std::str::FromStr;

use duplicate_finder::Duplicates;

mod csv;
mod json;
use self::csv::CsvOutputter;
use self::json::JsonOutputter;

#[derive(Clone, Copy)]
pub enum OutputType {
    CSV,
    JSON,
}

#[derive(Debug)]
pub struct ParseOutputTypeError;

pub trait Outputter {
    fn output(&self, stream: &mut Write, d : &Duplicates) -> ();
}

pub fn get_outputter(o: OutputType) -> Box<Outputter> {
    match o {
        OutputType::CSV => Box::new(CsvOutputter {}),
        OutputType::JSON => Box::new(JsonOutputter {}),
    }
}

impl fmt::Display for ParseOutputTypeError {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(fmt, "provided string did not match any OutputType variant")
    }
}

impl ::std::error::Error for ParseOutputTypeError {
    fn description(&self) -> &str {
        "provided string did not match any  OutputType variant"
    }
}

impl FromStr for OutputType {
    type Err = ParseOutputTypeError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "CSV" => Ok(OutputType::CSV),
            "JSON" => Ok(OutputType::JSON),
            _ => Err(ParseOutputTypeError),
        }
    }
}
