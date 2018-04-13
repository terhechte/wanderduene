use std::error::Error;
use std::io;
use std::path::PathBuf;
use std::collections::HashMap;

/*struct TestPage;

impl DunePage for TestPage {
}


#[test]
fn testing() {
    let posts = vec![OrgPost::new("test1".to_owned(), "2017".to_owned(), "05".to_owned(), "04".to_owned()),
                     OrgPost::new("test1".to_owned(), "2016".to_owned(), "05".to_owned(), "04".to_owned()),
                     OrgPost::new("test1".to_owned(), "2015".to_owned(), "05".to_owned(), "04".to_owned()),
    ];
    let pages = vec![TestPage {}];
    let dune = DuneBase::new(posts, pages);
    dune.posts().map(|post|println!("post: {}", post.title()));
}


tasks:
- should I slice everything into the groups? i.e. instead of &Vec<BlogPost> I do &[BlogPost] ?
*/

/*struct BlogPost {
    title: String,
}

impl BlogPost {
    fn new() -> BlogPost {
        BlogPost {
            title: "yeah".to_string(),
        }
    }
}*/

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
    /*fn write_posts(self) -> io::Result<()> {
        for post in self.payload {
            let mut path = self.path.clone();
            path.push(&post.identifier);
            path.push("index.html");
            println!("writing posts at {:?}", &path);
        }
        Ok(())
    }*/
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

/*impl<'a> DuneBuildCollector<'a> for GroupedDuneBuilder<'a> {
    fn with_posts<F>(self, action: F) -> Self where F: (Fn(&'a BlogPost, i32, i32, &PathBuf) -> ()) {
        let count = self.payload.len();
        for (current, post) in self.payload.iter().enumerate() {
            action(post, current, count, &self.path);
        }
        self
    }
}*/

struct PagedDuneBuilder;

/*impl PagedDuneBuilder {
    fn with<F>(self, action: F) -> Self where F: (Fn(Builder, i32) -> Builder) {
        self
    }
}*/

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


/*
trait DuneBuilder {
    fn group_by(self, key: DuneBaseAggType) -> Self;
    fn paged(self) -> PageDuneBuilder;
    // writes index.html files with short one line descs
    fn write_overview(self) -> Self;
    // writes folders with the name of the post and and index.html
    fn write_posts(self) -> Self;
    // writes an index
    fn write_index(self) -> Self;
}

trait DuneBuilderConsumable {
    fn consume(self) -> Result<(), Box<Error>>;
}

struct Builder<'a> {
    payload: Cell<&'a Vec<BlogPost>>
}

impl<'a> Builder<'a> {
    fn new(posts: &'a Vec<BlogPost>) -> Builder {
        Builder {
            payload: Cell::new(posts),
        }
    }
}

impl<'a> DuneBuilder for Builder<'a> {
    fn group_by(self, key: DuneBaseAggType) -> Self {
        self
    }
    fn paged(self) -> PageDuneBuilder {
        PageDuneBuilder {}
    }
    fn overview(self) -> Self {
        self
    }
    fn posts(self) -> Self {
        self
    }
}

impl<'a> DuneBuilderConsumable for Builder<'a> {
    fn consume(self) -> Result<(), Box<Error>> {
        Ok(())
    }
}

struct PageDuneBuilder;
impl DuneBuilder for PageDuneBuilder {
    fn group_by(self, key: DuneBaseAggType) -> Self {
        self
    }
    fn paged(self) -> Self {
        panic!("Cannot page a paged builder");
    }
    fn overview(self) -> Self {
        self
    }
    fn posts(self) -> Self {
        self
    }
}

impl<'a> DuneBuilderConsumable for PageDuneBuilder<'a> {
    fn consume(self) -> Result<(), Box<Error>> {
        Ok(())
    }
}

struct GroupedDuneBuilder;

impl GroupedDuneBuilder {
    fn with<F>(self, action: F) -> Self where F: (Fn(Builder) -> Builder) {
    }
}

impl<'a> DuneBuilderConsumable for GroupedDuneBuilder<'a> {
    fn consume(self) -> Result<(), Box<Error>> {
        Ok(())
    }
}

#[test]
fn testing() {
    let start = Builder {};
    start.group_by(DuneBaseAggType::Day)
        .overview()
        .with(|builder| {
            builder.group_by(DuneBaseAggType::Year)
                .overview()
        })
        .build();
}
*/
