use self::super::super::dune_post::{DunePost, DunePostTime};
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

// Include the pandoc template
const PANDOC_HTML: &'static str = include_str!("pandoc.html");

fn slurp<T: AsRef<Path>>(path: T) -> String {
    let mut buf = String::new();
    let mut file = File::open(path).unwrap();
    file.read_to_string(&mut buf).unwrap();
    buf
}

fn spit<T: AsRef<Path>>(path: T, contents: &str)  {
    let mut file = File::create(path).unwrap();
    file.write(contents.as_bytes());
}

impl DunePost {

    pub fn new(
        filename: &str,
        path: &PathBuf,
        cache: &CacheDB)
        -> Result<DunePost, OrgError> {
        let name = filename.clone().replace(".org", "");
        let (title, route, year, month, day) = match DunePost::parse_filename(&name) {
            Some(n) => n,
            None => {
                return Err(OrgError {
                    message: format!("Could not parse filename {}", &name),
                })
            }
        };
        // we will read the file twice, which is ok for this simple, awful, blog project
        //let contents =
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
            contents_html = DunePost::render_pandoc(&path, fileinfo.has_toc())
                .map_err(|e|OrgError { message: format!("Error: {}", e) })?;
            cache.set_cache_entry(identifier, &contents_html);
        }

        let tags = fileinfo.tags();
        let keywords = fileinfo.keywords();
        let description = fileinfo.desc();
        let enabled = fileinfo.is_enabled();

        let year_number = year.to_string().parse::<i32>().unwrap();
        let month_number = month.to_string().parse::<i32>().unwrap();
        let day_number = day.to_string().parse::<i32>().unwrap();
        Ok(DunePost {
            identifier: name.clone(),
            path: name.clone(),
            title: title,
            released: DunePostTime {
                year: year,
                month: month,
                day: day,
                values: (year_number, month_number, day_number),
            },
            contents: contents_html,
            tags: tags,
            keywords: keywords,
            description: description,
            enabled: enabled
        })
    }

    fn render_pandoc(path: &Path, has_toc: bool) -> Result<String, Box<Error>> {
        spit("/tmp/htmltemplate.html", PANDOC_HTML);
        let mut args: Vec<&str> = vec!["--template", "/tmp/htmltemplate.html", "-s"];
        if has_toc {
            args.push("--toc");
        }
        let mut output = Command::new("pandoc")
            .args(&args)
            .arg(path.to_str().unwrap())
            .output()?;
        if &output.stderr.len() > &0 {
            println!("errrr: {:?}", str::from_utf8(&output.stderr));
        }
        Ok(str::from_utf8(&output.stdout)?.to_owned())
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
}
