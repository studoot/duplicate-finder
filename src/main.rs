use std::sync::{Arc, Mutex};
use std::vec::Vec;

use ignore::WalkBuilder;
use structopt::StructOpt;

mod duplicate_finder;
use duplicate_finder::DuplicateFinder;
mod output;
use output::{OutputType, Outputter, get_outputter};

#[derive(StructOpt)]
#[structopt(name = "duplicate-finder",
            about = "Searchs for duplicated files in directory tree(s).")]
struct CLI {
    #[structopt(short = "p", long = "parallel", help = "Use multi-threaded finder")] parallel: bool,
    #[structopt(short = "t", long = "output-type", help = "Specify output format (CSV, JSON)",
                default_value = "CSV")]
    output_type: OutputType,
    #[structopt(name = "directory", help = "Directory tree root(s)", required = true)]
    input:
        Vec<String>,
}

fn main() {
    let opts = CLI::from_args();


    let use_threads = opts.parallel;
    for dir in opts.input {
        let mut dir_walker = WalkBuilder::new(dir);
        dir_walker
            .hidden(false)
            .git_exclude(false)
            .git_ignore(false)
            .git_global(false)
            .ignore(false)
            .parents(false)
            .threads(num_cpus::get());

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
        let mut duplicates = duplicate_finder.get_duplicates(use_threads);
        duplicates.sort_by_key(|duplicate| duplicate.first().map(|&p| p.clone()));

        let _outputter = get_outputter(opts.output_type);
        for duplicate_list in &duplicates {
            println!("[");
            for duplicate in duplicate_list {
                println!("  {}", duplicate.to_string_lossy());
            }
            println!("]");
        }
        println!(
            "{} duplicate groups with a total of {} files",
            duplicates.len(),
            duplicates.iter().fold(0, |count, ds| count + ds.len())
        );
    }
}
