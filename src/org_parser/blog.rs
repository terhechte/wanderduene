use std::fs;
use std::path::Path;
use std::io::prelude::*;
use std::error::Error;
use std::fmt;
use std::fs::File;
use std::process::Command;
use std::str;
use sha2::{Sha256, Digest};
use fileinfo::FileInfo;

#[derive(Debug)]
struct BlogError {
    message: String,
}

impl Error for BlogError {
    fn description(&self) -> &str {
        &self.message
    }
}

impl fmt::Display for BlogError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Blog Error: {}", self.message)
    }
}

// First test at doing something in rust. Find-in-file
// step 1: find all files in subdirectories matching glob
// step 2: do a regex search in each file for the given input

pub type Identifier = String;

pub struct BlogConfiguration {
    // The name of this blog
    pub blog_name: String,
    // The folder that contains the posts
    pub posts_folder: String,
    pub post_extension: String,
    // List of supported post types with the Fn that converts them info meta information
    //    post_file_extensions: Vec<(String, Fn(FileMatch) -> FileInfo)>
    pub posts_per_page: i8,
}

#[derive(PartialEq)]
pub enum BlogEntryType {
    Org,
    Markdown,
}


pub struct OrgBlogParser<'a> {
    configuration: &'a BlogConfiguration,
}

impl<'a> Blog<'a> {
    pub fn new(configuration: &'a BlogConfiguration) -> Blog {
        let db = Database::<String>::open("rustest.cache").unwrap();
        Blog {
            configuration: configuration,
            entries: Blog::parse(configuration, db),
        }
    }

    fn parse(configuration: &BlogConfiguration, database: Database<String>) -> Vec<BlogEntry> {
        let mut matches: Vec<BlogEntry> = Vec::new();
        let path = Path::new(&configuration.posts_folder);
        let result = fs::read_dir(path);
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
                    if string.contains(&configuration.post_extension) {
                        match BlogEntry::new(&string, &configuration, &database) {
                            Ok(blog) => {
                                println!("Parsed {}", string);
                                if !blog.info.is_disabled() {
                                    matches.push(blog);
                                }
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
        matches.sort_by(|a, b| {
            b.timestamp().cmp(&b.timestamp())
        });
        matches.reverse();
        return matches;
    }
}
