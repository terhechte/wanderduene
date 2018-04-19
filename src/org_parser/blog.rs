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

//#[derive(Debug)]
pub struct BlogEntry {
    // The global identifier used to link this blogpost from tags, etc
    pub identifier: Identifier,
    pub filename: String,
    pub info: FileInfo,
    // instead of using a time library, we're just using
    // the strings from the filename. everything low level
    pub time_info: (String, String, String),
    pub title: String,
    // The raw contents, not converted yet
    pub contents_raw: String,
    // The Pandoc Output
    pub contents_html: String,
    pub entry_type: BlogEntryType,
    pub route: String,
}

impl BlogEntry {
    pub fn new(
        filename: &String,
        configuration: &BlogConfiguration,
        database: &Database<String>
    ) -> Result<BlogEntry, Box<Error>> {
        let name = filename.clone().replace(".org", "");
        let (title, route, year, month, day) = match BlogEntry::parse_filename(&name) {
            Some(n) => n,
            None => {
                return Err(Box::new(BlogError {
                    message: format!("Could not parse filename {}", &name),
                }))
            }
        };
        let path = Path::new(&configuration.posts_folder).join(filename.clone());
        // we will read the file twice, which is ok for this simple, awful, blog project
        //let contents =
        let contents: String = match File::open(&path).ok().and_then(move |mut file| {
            let mut buf = String::new();
            file.read_to_string(&mut buf);
            return Some(buf);
        }) {
            Some(b) => b,
            None => {
                return Err(Box::new(
                    BlogError { message: format!("Could read file {}", &name) },
                ))
            }
        };

        let mut hasher = Sha256::default();
        hasher.input(&contents.as_bytes());
        let hash = hasher.result();
        let identifier = &filename.clone();

        let fileinfo = FileInfo::new(path.as_path());

        let contents_html: String;
        if let Ok(retrieved_content) = database.retrieve(identifier) {
            contents_html = retrieved_content;
        } else {
            contents_html = BlogEntry::render_pandoc(&path, fileinfo.has_toc())?;
            database.insert(identifier, contents_html.clone());
            database.flush().unwrap();
        }

        // maybe a "router" struct that receives entities (i.e. BlogEntry)  and maps them to routes
        //let full_route = format!("{}/{}/{}/{}.html", year, month, day, route);
        Ok(BlogEntry {
            identifier: name,
            filename: filename.clone(),
            info: fileinfo,
            time_info: (year, month, day),
            title: title,
            contents_raw: contents,
            contents_html: contents_html,
            entry_type: BlogEntryType::Org,
            route: route,
        })
    }

    fn render_pandoc(path: &Path, has_toc: bool) -> Result<String, Box<Error>> {
        //println!("path!: {:?}", &path);
        ///         .args(&["-l", "-a"])
        let mut args: Vec<&str> = vec!["--template", "./htmltemplate.html", "-s"];
        if has_toc {
            args.push("--toc");
        }
        let mut output = Command::new("pandoc")
            .args(&args)
            .arg(path.to_str().unwrap())
            .output()?;
        println!("errrr: {:?}", str::from_utf8(&output.stderr));
        Ok(str::from_utf8(&output.stdout)? .to_owned())
    }

    fn parse_filename(filename: &String) -> Option<(String, String, String, String, String)> {
        let components: Vec<&str> = filename.split("-").collect();
        if components.len() < 4 {
            return None;
        }
        let title_components = &components[3..];
        let title = title_components.connect(" ");
        let route = title_components.connect("-");
        return Some((
            title,
            route,
            components[0].to_owned(),
            components[1].to_owned(),
            components[2].to_owned(),
        ));
    }

    pub fn timeinfo_string(&self) -> String {
        format!(
            "{}-{}-{}",
            self.time_info.0,
            self.time_info.1,
            self.time_info.2
        )
    }

    // Fake timestamp for ordering without requiring another crate.
    pub fn timestamp(&self) -> i64 {
        let base_year: i64 = 2010;
        let year = self.time_info.0.to_string().parse::<i64>().unwrap();
        let month = self.time_info.1.to_string().parse::<i64>().unwrap();
        let day = self.time_info.2.to_string().parse::<i64>().unwrap();
        let year = year - base_year;
        // println!("ts for {}: year: {} month: {} day: {}", &self.timeinfo_string(), &year, &month, &day);
        return (year * 12 * 31) + (month * 31) + day;
//        let (Some(year), Some(month), Some(day)) = match self.time_info
    }

//    pub fn formatted_date(&self) -> String {
//        format!("")
//    }
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
