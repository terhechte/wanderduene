use self::super::super::dune_post::{DunePost, DunePostTime};
use std::path::{PathBuf, Path};
use org_parser::dune_post::*;
use org_parser::cache_db::CacheDB;
use std::fs;

pub struct OrgParser {
    folder: PathBuf,
    max_threads: i32,
    org_extension: &'static str,
    cache_db: CacheDB
}

impl OrgParser {
    pub fn new<T: AsRef<Path>>(folder: T, max_threads: i32) -> OrgParser {
        OrgParser {
            folder: folder.as_ref().to_owned(),
            max_threads,
            org_extension: "org",
            cache_db: CacheDB::new("cache.db")
        }
    }

    pub fn parse(&self) -> Vec<DunePost> {
        let mut matches: Vec<DunePost> = Vec::new();
        let result = fs::read_dir(&self.folder);
        if let Err(x) = result {
            return matches;
        }
        for entry in result.unwrap() {
            let path = match entry {
                Ok(x) => x,
                Err(_) => continue,
            };
            match path.file_name().into_string() {
                Ok(string) => {
                    if string.contains(&self.org_extension) {
                        match DunePost::new(&string, &path.path(), &self.cache_db) {
                            Ok(blog) => {
                                matches.push(blog);
                            }
                            Err(e) => {
                                println!("Could not parse {}", string);
                                println!("Error: {}", e);
                                continue;
                            }
                        }
                    }
                }
                Err(e) => {
                    println!("Invalid filename: ");
                    continue;
                }
            };
        }
        /*matches.sort_by(|a, b| {
            b.timestamp().cmp(&b.timestamp())
        });
        matches.reverse();*/
        return matches;
    }
}
