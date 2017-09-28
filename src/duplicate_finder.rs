use std::path::{Path, PathBuf};

use multimap::MultiMap;

pub type Duplicates<'a> = Vec<Vec<&'a PathBuf>>;

pub struct DuplicateFinder {
    all_files: MultiMap<u64, PathBuf>,
    entry_count: usize,
}

mod detail {
    use std::collections::hash_map::DefaultHasher;
    use std::fs::File;
    use std::hash::Hasher;
    use std::io::prelude::*;
    use std::path::PathBuf;

    use multimap::MultiMap;
    use rayon::prelude::*;

    use duplicate_finder::Duplicates;

    fn get_file_hash(p: &PathBuf) -> Option<u64> {
        if let Ok(mut f) = File::open(p) {
            let mut bytes = Vec::new();
            if let Ok(_) = f.read_to_end(&mut bytes) {
                let mut hasher = DefaultHasher::new();
                hasher.write(bytes.as_slice());
                return Some(hasher.finish());
            }
        }
        None
    }

    pub fn collect_duplicates_par(possible_dups: &Vec<PathBuf>) -> Duplicates {
        let duplicates_as_vec: Vec<_> = possible_dups
            .par_iter()
            .map(|p| get_file_hash(p).map(|h| (h, p)))
            .filter(|o| o.is_some())
            .map(|o| o.unwrap())
            .collect();
        let mut duplicates: MultiMap<u64, &PathBuf> = MultiMap::new();
        duplicates_as_vec
            .iter()
            .for_each(|&(k, v)| duplicates.insert(k, v));
        duplicates
            .iter_all()
            .filter(|&(_, ref possible_duplicates)| {
                possible_duplicates.len() > 1
            })
            .map(|(_, ref possible_duplicates)| *possible_duplicates)
            .cloned()
            .collect()
    }

    pub fn collect_duplicates_seq(possible_dups: &Vec<PathBuf>) -> Vec<Vec<&PathBuf>> {
        let mut duplicates: MultiMap<u64, &PathBuf> = MultiMap::new();
        possible_dups.iter().for_each(|p| {
            get_file_hash(p).map(|h| duplicates.insert(h, p));
            ()
        });
        duplicates
            .iter_all()
            .filter(|&(_, ref possible_duplicates)| {
                possible_duplicates.len() > 1
            })
            .map(|(_, ref possible_duplicates)| *possible_duplicates)
            .cloned()
            .collect()
    }
}

impl DuplicateFinder {
    pub fn new() -> DuplicateFinder {
        DuplicateFinder {
            all_files: MultiMap::new(),
            entry_count: 0,
        }
    }
    pub fn process_entry(&mut self, p: &Path, s: u64) -> () {
        self.entry_count = self.entry_count + 1;
        self.all_files.insert(s, p.to_path_buf());
    }
    pub fn get_entry_count(&self) -> usize {
        self.entry_count
    }
    pub fn get_duplicates(&self, par: bool) -> Vec<Vec<&PathBuf>> {
        if par {
            self.all_files
                .iter_all()
                .filter(|&(_, ref possible_duplicates)| {
                    possible_duplicates.len() > 1
                })
                .flat_map(|(_, ref possible_duplicates)| {
                    detail::collect_duplicates_par(&possible_duplicates)
                })
                .collect()
        } else {
            self.all_files
                .iter_all()
                .filter(|&(_, ref possible_duplicates)| {
                    possible_duplicates.len() > 1
                })
                .flat_map(|(_, ref possible_duplicates)| {
                    detail::collect_duplicates_seq(&possible_duplicates)
                })
                .collect()
        }
    }
}
