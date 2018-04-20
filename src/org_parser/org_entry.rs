use org_parser::cache_db::CacheDB;
use org_parser::org_error::OrgError;
use org_parser::fileinfo::FileInfo;

use std::fs; use std::path::{Path, PathBuf};
use std::io::prelude::*;
use std::error::Error;
use std::fmt;
use std::fs::File;
use std::process::Command;
use std::str;
use sha2::{Sha256, Digest};

#[derive(Debug)]
pub struct OrgEntry {
    pub identifier: String,
    pub filename: String,
    pub info: FileInfo,
    pub time_info: (String, String, String),
    pub title: String,
    // The raw contents, not converted yet
    pub contents_raw: String,
    // The Pandoc Output
    pub contents_html: String,
    pub route: String,
}

impl OrgEntry {
    pub fn new(
        filename: &str,
        path: &PathBuf,
        cache: &CacheDB)
        -> Result<OrgEntry, OrgError> {
        let name = filename.clone().replace(".org", "");
        let (title, route, year, month, day) = match OrgEntry::parse_filename(&name) {
            Some(n) => n,
            None => {
                return Err(OrgError {
                    message: format!("Could not parse filename {}", &name),
                })
            }
        };
        // we will read the file twice, which is ok for this simple, awful, blog project
        //let contents =
        println!("path: {:?}", &path);
        let contents: String = match File::open(&path).ok().and_then(move |mut file| {
            let mut buf = String::new();
            file.read_to_string(&mut buf);
            return Some(buf);
        }) {
            Some(b) => b,
            None => {
                return Err(
                    OrgError { message: format!("Could read file {}", &name) },
                )
            }
        };

        let mut hasher = Sha256::default();
        hasher.input(&contents.as_bytes());
        let hash = hasher.result();
        let identifier = &filename.clone();

        let fileinfo = FileInfo::new(path.as_path());

        let contents_html: String;
        if let Some(retrieved_content) = cache.cache_entry(identifier) {
            contents_html = retrieved_content;
        } else {
            contents_html = OrgEntry::render_pandoc(&path, fileinfo.has_toc())
                .map_err(|e|OrgError { message: format!("Error: {}", e) })?;
            cache.set_cache_entry(identifier, &contents_html);
        }

        Ok(OrgEntry {
            identifier: name,
            filename: filename.to_owned(),
            info: fileinfo,
            time_info: (year, month, day),
            title: title,
            contents_raw: contents,
            contents_html: contents_html,
            route: route,
        })
    }

    fn render_pandoc(path: &Path, has_toc: bool) -> Result<String, Box<Error>> {
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

    pub fn timestamp(&self) -> i64 {
        let base_year: i64 = 2010;
        let year = self.time_info.0.to_string().parse::<i64>().unwrap();
        let month = self.time_info.1.to_string().parse::<i64>().unwrap();
        let day = self.time_info.2.to_string().parse::<i64>().unwrap();
        let year = year - base_year;
        return (year * 12 * 31) + (month * 31) + day;
    }
}
