extern crate multimap;
extern crate walkdir;

use walkdir::WalkDir;
use multimap::MultiMap;
use std::path::PathBuf;

struct DuplicateFinder {
    all_files: MultiMap<u64, PathBuf>,
}

mod detail {
    use std::io::BufReader;
    use std::io::prelude::*;
    use std::fs::File;
    use std::path::PathBuf;

    fn files_are_equal(p1: &PathBuf, p2: &PathBuf) -> bool {
        if let Ok(f1) = File::open(p1) {
            if let Ok(f2) = File::open(p2) {
                println!("Comparing {:?} and {:?}", p1, p2);
                let b1 = BufReader::new(f1).bytes();
                let b2 = BufReader::new(f2).bytes();
                return b1.zip(b2)
                    .all(|(read_byte1, read_byte2)| match (read_byte1, read_byte2) {
                        (Ok(byte1), Ok(byte2)) if byte1 == byte2 => true,
                        _ => false,
                    });
            }
        }
        false
    }

    pub fn collect_duplicates(possible_dups: &Vec<PathBuf>) -> Vec<Vec<&PathBuf>> {
        let mut duplicates: Vec<Vec<&PathBuf>> = Vec::new();
        if possible_dups.len() > 1 {
            duplicates.push(vec![&possible_dups[0]]);
            possible_dups.iter().skip(1).for_each(|f1| {
                match duplicates
                    .iter()
                    .position(|files2| files_are_equal(f1, &files2[0]))
                {
                    Some(insertion_pos) => duplicates[insertion_pos].push(f1),
                    None => duplicates.push(vec![&f1]),
                }
            })
        }
        duplicates
            .into_iter()
            .filter(|ref one_set_of_duplicates| one_set_of_duplicates.len() > 1)
            .collect()
    }

}

impl DuplicateFinder {
    pub fn new() -> DuplicateFinder {
        DuplicateFinder {
            all_files: MultiMap::new(),
        }
    }
    pub fn process_entry(&mut self, entry: &walkdir::DirEntry) -> () {
        if let Ok(metadata) = entry.metadata() {
            if metadata.file_type().is_file() {
                self.all_files
                    .insert(metadata.len(), entry.path().to_path_buf());
            }
        }
    }
    pub fn get_duplicates(&self) -> Vec<Vec<&PathBuf>> {
        self.all_files
            .iter_all()
            .filter(|&(_, ref possible_duplicates)| {
                possible_duplicates.len() > 1
            })
            .flat_map(|(_, ref possible_duplicates)| {
                detail::collect_duplicates(&possible_duplicates)
            })
            .collect()
    }
}


fn main() {
    let mut duplicate_finder = DuplicateFinder::new();
    let mut files_scanned: u64 = 0;
    for result in WalkDir::new(".") {
        // Each item yielded by the iterator is either a directory entry or an
        // error, so either print the path or the error.

        match result {
            Ok(entry) => {
                duplicate_finder.process_entry(&entry);
                files_scanned = files_scanned + 1;
            }
            Err(err) => println!("ERROR: {}", err),
        }
    }
    for duplicate_list in duplicate_finder.get_duplicates() {
        println!("[");
        for duplicate in duplicate_list {
            println!("  {}", duplicate.to_string_lossy());
        }
        println!("]");
    }
    println!("{} files scanned", files_scanned);
}
