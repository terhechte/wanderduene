use std::path::Path;
use std::error::Error;
use std::io::prelude::*;
use std::io::BufReader;
use std::fs::File;

#[derive(Debug)]
pub struct FileInfo {
    pub meta: Vec<FileMeta>,
    pub sources: Vec<FileSource>,
}

#[derive(Debug)]
pub struct FileMeta {
    pub key_name: String,
    pub value: String,
}

#[derive(Debug)]
pub struct FileSource {
    pub contents: String,
    pub properties: Vec<(String, String)>,
}

#[allow(dead_code)]
impl FileInfo {
    pub fn new(path: &Path) -> FileInfo {
        match FileInfo::meta_contents_of_file(path) {
            Ok(x) => {
                return FileInfo {
                    meta: x.0,
                    sources: x.1,
                }
            }
            Err(_) => {
                return FileInfo {
                    meta: Vec::new(),
                    sources: Vec::new(),
                }
            }
        };
    }

    pub fn describe(&self) {
        for meta in self.meta.iter() {
            println!("{} => {}", meta.key_name, meta.value);
        }
        for source in self.sources.iter() {
            println!("Begin:");
            for pair in source.properties.iter() {
                println!(":: {} => {}", pair.0, pair.1);
            }
            println!("::code::");
            println!("{}", source.contents);
            println!("End:");
        }
    }
}

impl FileInfo {
    pub fn tags<'a>(&'a self) -> Vec<&'a str> {
        match self.meta_contents("+tags:") {
            Some(n) => n.trim().split(" ").collect(),
            None => Vec::new(),
        }
    }

    pub fn keywords<'a>(&'a self) -> Vec<&'a str> {
        match self.meta_contents("+keywords:") {
            Some(n) => n.trim().split(" ").collect(),
            None => Vec::new(),
        }
    }

    pub fn is_disabled(&self) -> bool {
        match self.meta_contents("+inactive:") {
            Some(n) => n.contains("true"),
            _ => false
        }
    }

    pub fn has_toc(&self) -> bool {
        match self.meta_contents("+OPTIONS:") {
            Some(n) => !n.contains("toc:nil"),
            _ => true
        }
    }

    pub fn meta_contents<'a>(&'a self, field: &'static str) -> Option<&'a str> {
        match self.meta
            .iter()
            .filter(|entry| entry.key_name == field)
            .nth(0) {
            Some(n) => Some(&n.value),
            None => None,
        }
    }
}

impl FileInfo {
    fn meta_contents_of_file(path: &Path) -> Result<(Vec<FileMeta>, Vec<FileSource>), Box<Error>> {
        let f = File::open(path)?;
        let reader = BufReader::new(f);
        let mut result: Vec<FileMeta> = Vec::new();
        let mut sources: Vec<FileSource> = Vec::new();

        let mut is_in_source_block = false;
        let mut current_source: Vec<String> = Vec::new();
        let mut current_properties: Option<Vec<(String, String)>> = None;
        for line in reader.lines() {
            let line = match line {
                Ok(l) => l,
                Err(_) => continue,
            };
            if is_in_source_block {
                if FileInfo::is_source_end(&line) {
                    is_in_source_block = false;
                    match current_properties {
                        Some(x) => {
                            sources.push(FileSource {
                                contents: current_source.join("\n"),
                                properties: x,
                            })
                        }
                        None => (),
                    };
                    current_properties = None;
                } else {
                    current_source.push(line);
                }
                continue;
            }
            if let Some(x) = FileInfo::is_source_begin(&line) {
                is_in_source_block = true;
                current_source = Vec::new();
                current_properties = Some(x);
                continue;
            }
            match FileInfo::find_line_meta(line) {
                Some(x) => {
                    result.push(FileMeta {
                        key_name: x.0,
                        value: x.1,
                    })
                }
                None => continue,
            };
        }
        return Ok((result, sources));
    }

    fn is_source_begin(begin_line: &String) -> Option<Vec<(String, String)>> {
        let pattern = "+BEGIN_SRC";
        if !begin_line.to_uppercase().starts_with(pattern) {
            return None;
        }
        let mut properties: Vec<(String, String)> = Vec::new();
        let mut previous_entry: Option<String> = None;
        for entry in begin_line.split(" ") {
            let entry = entry.to_owned();
            if entry.to_uppercase() == pattern {
                continue;
            };
            if let Some(previous) = previous_entry {
                properties.push((previous, entry));
                previous_entry = None;
                continue;
            }
            if entry.starts_with(":") {
                previous_entry = Some(entry);
            } else {
                properties.push((entry, "".to_owned()));
            }
        }
        return Some(properties);
    }

    fn is_source_end(line: &String) -> bool {
        return line.to_uppercase().starts_with("+END_SRC");
    }

    fn find_line_meta(line: String) -> Option<(String, String)> {
        let pattern = "#+";
        if line.starts_with(pattern) {
            let position = match line.find(":") {
                Some(n) => n,
                None => return None,
            };
            if (position + 1) == line.len() {
                return Some((line, "".to_owned()));
            }
            let components = line[(pattern.len() - 1)..].split_at(position);
            let mut key = components.0.trim().to_owned();
            let value = components.1.to_owned();
            return Some((key, value));
        }
        None
    }
}

// TESTING

#[test]
fn test_meta_contents_of_file() {
    let filename = "/tmp/a1.org".to_owned();
    let contents = "#+option abc: klaus
#+title: Benedikt
#+tags: this is a list of tags
this is example text1.
this is example text2.
this is example text3.
+begin_src swift :abc a1 :cba a2
func test(abc: Int) -> Bool {
  return abc >= 5 && abc <= 20
}
+end_src

more text
more text

+BEGIN_SRC javascript
console.log(undefined);
+END_SRC

final end.
";
    {
        let mut output = File::create(filename.clone()).unwrap();
        output.write(contents.as_bytes());
    }

    let info = FileInfo::new(Path::new(&filename));
    info.describe();
}
