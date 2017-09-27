#![feature(integer_atomics)]

#[macro_use]
extern crate clap;
extern crate ignore;
extern crate multimap;
extern crate rayon;

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use ignore::WalkBuilder;
use multimap::MultiMap;

struct DuplicateFinder {
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

    static mut FILES_HASHED: u64 = 0;

    fn get_file_hash(p: &PathBuf) -> Option<u64> {
        if let Ok(mut f) = File::open(p) {
            let mut bytes = Vec::new();
            if let Ok(_) = f.read_to_end(&mut bytes) {
                unsafe {
                    FILES_HASHED = FILES_HASHED + 1;
                }
                let mut hasher = DefaultHasher::new();
                hasher.write(bytes.as_slice());
                return Some(hasher.finish());
            }
        }
        None
    }

    pub fn file_hashes() -> u64 {
        unsafe { FILES_HASHED }
    }

    pub fn collect_duplicates_par(possible_dups: &Vec<PathBuf>) -> Vec<Vec<&PathBuf>> {
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

fn main() {
    let matches = clap_app!(myapp =>
        (version: crate_version!())
        (author: "Stuart Dootson<stuart.dootson@rolls-royce.com>")
        (about: "Find duplicate files in directory tree(s)")
        (@arg INPUT: +multiple +required "Set the input directory(s) to use")
        (@arg PARALLEL: -p --parallel "Use multi-threaded engine")
    ).get_matches();


    if let Some(dirs) = matches.values_of("INPUT") {
        let use_threads = matches.is_present("PARALLEL");
        for dir in dirs {
            let mut dir_walker = WalkBuilder::new(dir);
            dir_walker
                .hidden(false)
                .git_exclude(false)
                .git_ignore(false)
                .git_global(false)
                .ignore(false)
                .parents(false);

            let duplicate_finder: Arc<Mutex<DuplicateFinder>> =
                Arc::new(Mutex::new(DuplicateFinder::new()));
            if use_threads {
                dir_walker.build_parallel().run(|| {
                    let dup_finder = duplicate_finder.clone();
                    Box::new(move |result| {
                        if let Ok(entry) = result {
                            if let Ok(metadata) = entry.metadata() {
                                if metadata.file_type().is_file() {
                                    dup_finder
                                        .lock()
                                        .unwrap()
                                        .process_entry(entry.path(), metadata.len());
                                }
                            }
                        }
                        ignore::WalkState::Continue
                    })
                });
            } else {
                let mut duplicate_finder = duplicate_finder.lock().unwrap();
                let mut files_found = std::collections::hash_set::HashSet::new();
                for result in dir_walker.build() {
                    // Each item yielded by the iterator is either a directory entry or an
                    // error, so either print the path or the error.

                    match result {
                        Ok(entry) => if let Ok(metadata) = entry.metadata() {
                            files_found.insert(entry.path().to_path_buf());
                            if metadata.file_type().is_file() {
                                duplicate_finder.process_entry(entry.path(), metadata.len());
                            }
                        },
                        Err(err) => println!("ERROR: {}", err),
                    }
                }
            }
            let duplicate_finder = duplicate_finder.lock().unwrap();
            println!("{} files scanned", duplicate_finder.get_entry_count());
            let duplicates = duplicate_finder.get_duplicates(use_threads);
            // for duplicate_list in duplicates {
            //     println!("[");
            //     for duplicate in duplicate_list {
            //         println!("  {}", duplicate.to_string_lossy());
            //     }
            //     println!("]");
            // }
            println!("{} file hashes", detail::file_hashes());
            println!(
                "{} duplicate groups with a total of {} files",
                duplicates.len(),
                duplicates.iter().fold(0, |count, ds| count + ds.len())
            );
        }
    }
}
