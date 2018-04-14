use std::error::Error;
use std::io;
use std::path::PathBuf;
use std::collections::HashMap;


/**
FIXME:
- Error Handling
- Keywords
- Tags
- Unit Tests
- htmlwriter
- config
- router: a trait that returns the absolute path for keyword, tag, post, year, year/month, year/month/day
- the collection & parsing of items should happen on multiple threads possible with the rust go channels so that
  even for large post bases, the parsing is fast
- the writing of items could also happen on multiple threads, if the write actions are stored in a dependency tree,
i.e. each folder for a year would be a new node in the tree, each month for each year a subnode of the respective tree node
  then, we could paraellilize all years, and thend for each year all months
*/

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
}

struct Dune<Config, Writer, Router>
where Config: DuneConfigurator, Writer: DuneWriter<Config, Router>, Router: DuneRouter {
    database: Database,
    configuration: Config,
    writer: Writer,
    router: Router
}

impl<Config: DuneConfigurator, Writer: DuneWriter<Config, Router>, Router: DuneRouter> Dune<Config, Writer, Router>
{
    fn builder(&self) -> Builder {
        let path = PathBuf::from("html");
        let posts: Vec<&BlogPost> = self.database.posts.iter().collect();
        Builder::new(path, posts)
    }
}

// Traits

/// Configuration is a trait with required fields and a base implementation.
/// Individual users can, however, create their own implementation to have access
/// to additional properties in the `DuneWriter` or `DuneRouter` implementations.
trait DuneConfigurator {
}

trait DuneWriter<Config: DuneConfigurator, Router: DuneRouter> {
    // A list of items displayed as a list
    fn overview(&self, config: &Config, router: &Router, path: &PathBuf, posts: &[&BlogPost]) -> io::Result<()>;
    // A list of possibly paged items. Displayed as a blog of contents
    //fn index<'a>(&self, path: &PathBuf, page: &DunePage<'a>) -> io::Result<()>;
    // A single post
    //fn post(&self, path: &PathBuf, post: &BlogPost) -> io::Result<()>;
}

use std::marker::PhantomData;
struct HTMLWriter<C, R> where C: DuneConfigurator, R: DuneRouter {
    _phantom1: PhantomData<C>,
    _phantom2: PhantomData<R>,
}

impl<C, R> HTMLWriter<C, R> where C: DuneConfigurator, R: DuneRouter {
    fn new() -> HTMLWriter<C, R> {
        HTMLWriter {
            _phantom1: PhantomData,
            _phantom2: PhantomData
        }
    }
}

impl<C, R> DuneWriter<C, R> for HTMLWriter<C, R>  where C: DuneConfigurator, R: DuneRouter {
    fn overview(&self, config: &C, router: &R, path: &PathBuf, posts: &[&BlogPost]) -> io::Result<()> {
        println!("writing overview with {} posts to {:?}", posts.len(), &path);
        Ok(())
    }

    /*fn index<'b>(&self, path: &PathBuf, page: &DunePage<'b>) -> io::Result<()> {
        println!("writing index-page with {} posts to {:?}", page.posts.len(), &path);
        Ok(())
    }
    fn post(&self, path: &PathBuf, post: &BlogPost) -> io::Result<()> {
        println!("writing post {} to {:?}", post.title, &path);
        Ok(())
    }*/
}

trait DuneRouter {
    fn route_post(post: &BlogPost) -> String;
    fn route_tag(tag: &str) -> String;
    fn route_keyword(keyword: &str) -> String;
}


trait DuneBuildMapper<'a> {
    fn group_by(self, key: DuneBaseAggType) -> GroupedDuneBuilder<'a>;
    fn paged(self, i32) -> PagedDuneBuilder<'a>;
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

trait DunePathBuilder {
    fn push<T: AsRef<str>>(mut self, path: T) -> Self;
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

    fn paged(self, per_page: i32) -> PagedDuneBuilder<'a> {
        let mut result: Vec<DunePage> = Vec::new();
        let mut counter: i32 = 0;
        loop {
            let cloned = self.payload.clone();
            let entries: Vec<&BlogPost> = cloned
                .into_iter()
                .skip((counter * per_page) as usize)
                .take(per_page as usize)
                .collect();
            if entries.is_empty() {
                break;
            }
            let page = DunePage {
                page: counter + 1,
                previous_page: match counter {
                    0 => None,
                    _ => Some(counter),
                },
                next_page: match (((counter + 1) * per_page) as i32) < ((self.payload.len()) as i32) {
                    true => Some(counter + 2),
                    false => None,
                },
                posts: entries
            };
            result.push(page);
            counter += 1;
        }
        // for entry in &result {
        //     println!("page {}, {:?} {:?}", entry.number, entry.previous_page, entry.next_page);
        // }
        PagedDuneBuilder::new(self.path.clone(), result)
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

impl<'a> DunePathBuilder for Builder<'a> {
    fn push<T: AsRef<str>>(mut self, path: T) -> Self {
        self.path.push(path.as_ref());
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

struct DunePage<'a> {
    page: i32,
    previous_page: Option<i32>,
    next_page: Option<i32>,
    posts: Vec<&'a BlogPost>
}

struct PagedDuneBuilder<'a> {
    payload: Vec<DunePage<'a>>,
    path: PathBuf,
}

impl<'a> PagedDuneBuilder<'a> {
    fn new(path: PathBuf, payload: Vec<DunePage<'a>>) -> PagedDuneBuilder {
        PagedDuneBuilder {
            payload,
            path
        }
    }
}

impl<'a> DuneBuildFlatter<'a> for PagedDuneBuilder<'a> {
    type CategoryType = i32;
    fn with<F>(self, action: F) -> Self where F: (Fn(Builder<'a>, Self::CategoryType) -> ()) {
        for page in self.payload.iter() {
            let mut path = self.path.clone();
            let key = &format!("{}", page.page);
            path.push(&key);
            let inner_builder = Builder::new(path, page.posts.clone());
            action(inner_builder, page.page);
        }
        self
    }
}

impl<'a> DuneBuildWriter for PagedDuneBuilder<'a> {
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

//#[test]
fn testing() {

    struct AppventureConfig;

    impl DuneConfigurator for AppventureConfig {}

    struct TestingRouter;
    impl DuneRouter for TestingRouter {
        fn route_post(post: &BlogPost) -> String {
            "".to_owned()
        }
        fn route_tag(tag: &str) -> String {
            "".to_owned()
        }
        fn route_keyword(keyword: &str) -> String {
            "".to_owned()
        }
    }
    let database = Database::new();
    let writer: HTMLWriter<AppventureConfig, TestingRouter> = HTMLWriter::new();
    let dune: Dune<AppventureConfig, HTMLWriter<AppventureConfig, TestingRouter>, TestingRouter> = Dune {
        database: database,
        configuration: AppventureConfig {},
        writer: writer,
        router: TestingRouter {}
    };
    let builder = dune.builder();
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


    let builder = dune.builder();
    builder.push("latest-posts")
        .paged(1)
        .with(|builder, page| {
            builder.write_index().unwrap();
        });
}
