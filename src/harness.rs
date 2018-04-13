use std::error::Error;
use std::io;
use std::path::PathBuf;
use std::collections::HashMap;

pub struct BlogPost {
    pub identifier: String,
    pub path: String,
    pub title: String,
    pub released: DunePostTime,
    pub contents: String,
    pub tags: Vec<String>,
    pub keywords: Vec<String>,
    pub description: String,
    pub enabled: bool
}

impl BlogPost {
    pub fn new(title: String, year: String, month: String, day: String) -> BlogPost {
        BlogPost {
            identifier: title.clone(),
            path: title.clone(),
            title: title.clone(),
            released: DunePostTime {
                year: year,
                month: month,
                day: day,
                values: (0, 0, 0),
                timestamp: 0
            },
            contents: title.clone(),
            tags: vec![title.clone()],
            keywords: Vec::new(),
            description: title.clone(),
            enabled: true
        }
    }
}


#[derive(Copy, Clone)]
pub enum DuneBaseAggType {
    Year, Month, Day, Tag, Keyword, Enabled
}

pub struct DunePostTime {
    pub year: String,
    pub month: String,
    pub day: String,
    pub values: (i32, i32, i32),
    pub timestamp: i64
}



struct Database {
    posts: Vec<BlogPost>
}

impl Database {
    fn new() -> Database {
        let posts = vec![BlogPost::new("test1".to_owned(), "2017".to_owned(), "01".to_owned(), "14".to_owned()),
                         BlogPost::new("test2".to_owned(), "2016".to_owned(), "02".to_owned(), "24".to_owned()),
                         BlogPost::new("test3".to_owned(), "2015".to_owned(), "03".to_owned(), "31".to_owned()),
        ];
        Database {
            posts: posts,
        }
    }
    fn builder(&self) -> Builder {
        let path = PathBuf::from("html");
        let v8: Vec<&BlogPost> = self.posts.iter().collect();
        Builder::new(path, v8)
    }
}

// Traits

trait DuneBuildMapper<'a> {
    fn group_by(self, key: DuneBaseAggType) -> GroupedDuneBuilder<'a>;
    fn paged(self) -> PagedDuneBuilder;
}

trait DuneBuildFlatter<'a> {
    type CategoryType;
    fn with<F>(self, action: F) -> Self where F: (Fn(Builder<'a>, Self::CategoryType) -> ());
}

trait DuneBuildWriter {
    fn write_overview(self) -> io::Result<()>;
    fn write_index(self) -> io::Result<()>;
}

trait DuneBuildCollector<'a> {
    fn with_posts<F>(self, action: F) -> Self where F: (Fn(&'a BlogPost, i32, i32, &PathBuf) -> ());
}

// Types

struct Builder<'a> {
    payload: Vec<&'a BlogPost>,
    path: PathBuf
}

impl<'a> Builder<'a> {
    fn new(path: PathBuf, posts: Vec<&'a BlogPost>) -> Builder {
        Builder {
            payload: posts,
            path: path
        }
    }
}

impl<'a> DuneBuildMapper<'a> for Builder<'a> {
    fn group_by(self, key: DuneBaseAggType) -> GroupedDuneBuilder<'a> {
        //let grouped = self.payload.iter().fold(HashMap::<String, Vec<&BlogPost>>::new(), |mut acc: HashMap<String, Vec<&BlogPost>>, elm: &BlogPost| {
        let grouped = self.payload.iter().fold(HashMap::<String, Vec<&BlogPost>>::new(), |mut acc, elm| {
            {
                // FIXME: Move this logic as an internal impl on DuneBaseAggType?
                let key = match key {
                    DuneBaseAggType::Year => elm.released.year.to_string(),
                    DuneBaseAggType::Month => elm.released.month.to_string(),
                    DuneBaseAggType::Day => elm.released.day.to_string(),
                    _ => "fail".to_string()
                };
                let mut entry = acc.entry(key).or_insert(
                    Vec::new(),
                );
                entry.push(elm);
            }
            acc
        });
        // FIXME: Can I map this?
        let mut payload: Vec<(String, Vec<&'a BlogPost>)> = Vec::new();
        for (key, posts) in grouped {
            payload.push((key, posts));
        }
        GroupedDuneBuilder {
            payload: payload,
            path: self.path.clone()
        }
    }
    fn paged(self) -> PagedDuneBuilder {
        PagedDuneBuilder {}
    }
}

impl<'a> DuneBuildCollector<'a> for Builder<'a> {
    fn with_posts<F>(self, action: F) -> Self where F: (Fn(&'a BlogPost, i32, i32, &PathBuf) -> ()) {
        let count = self.payload.len();
        for (current, post) in self.payload.iter().enumerate() {
            action(post, current as i32, count as i32, &self.path);
        }
        self
    }
}

impl<'a> DuneBuildWriter for Builder<'a> {
    fn write_overview(self) -> io::Result<()> {
        let mut path = self.path;
        path.push("overview.html");
        println!("writing overview at {:?}", &path);
        Ok(())
    }

    fn write_index(self) -> io::Result<()> {
        println!("writing index at {:?}", &self.path);
        Ok(())
    }
}

struct GroupedDuneBuilder<'a> {
    payload: Vec<(String, Vec<&'a BlogPost>)>,
    path: PathBuf,
}

impl<'a> DuneBuildFlatter<'a> for GroupedDuneBuilder<'a> {
    type CategoryType = String;
    fn with<F>(self, action: F) -> Self where F: (Fn(Builder<'a>, Self::CategoryType) -> ()) {
        for (key, posts) in self.payload.iter() {
            let mut path = self.path.clone();
            path.push(&key);
            let inner_builder = Builder::new(path, posts.clone());
            action(inner_builder, key.to_owned());
        }
        self
    }
}

impl<'a> DuneBuildWriter for GroupedDuneBuilder<'a> {
    fn write_overview(self) -> io::Result<()> {
        let mut path = self.path;
        path.push("overview.html");
        println!("writing overview at {:?}", &path);
        Ok(())
    }
    fn write_index(self) -> io::Result<()> {
        println!("writing index at {:?}", &self.path);
        Ok(())
    }
}

struct PagedDuneBuilder;

#[test]
fn testing() {
    let db = Database::new();
    let builder = db.builder();
    builder.group_by(DuneBaseAggType::Year)
        .with(|builder, group| {
            builder.group_by(DuneBaseAggType::Month)
                .with(|builder, group| {
                    builder.group_by(DuneBaseAggType::Day)
                        .with(|builder, group| {
                            builder.with_posts(|post, current, total, path| {
                                // FIXME: have a writer here that can be used to write here
                                let mut path = path.clone();
                                path.push(&post.identifier);
                                path.push("index.html");
                                println!("writing posts at {:?}", &path);
                            }).write_overview().unwrap();
                        }).write_overview().unwrap();
                }).write_overview().unwrap();
        }).write_overview().unwrap();
}
