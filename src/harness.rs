#![feature(nll)]

use std::error::Error;
use std::io;
use std::path::{PathBuf, Path};
use std::collections::HashMap;
use std::rc::Rc;
use std::cell::Cell;
use std::mem;

// Useful to have
trait VecCellAppendable {
    type Item;
    fn append(&self, mut items: Vec<Self::Item>);
    fn push(&self, item: Self::Item);
}
impl<T> VecCellAppendable for Cell<Option<Vec<T>>> {
    type Item = T;
    fn append(&self, mut items: Vec<Self::Item>) {
        if let Some(mut current) = self.take() {
            current.append(&mut items);
            self.set(Some(current));
        }
    }

    fn push(&self, item: Self::Item) {
        if let Some(mut current) = self.take() {
            current.push(item);
            self.set(Some(current));
        }
    }
}

trait PathAppending {
    fn appending<T: AsRef<Path>>(&self, element: T) -> Self;
}

impl PathAppending for PathBuf {
    fn appending<T: AsRef<Path>>(&self, element: T) -> Self {
        let mut clone = self.clone();
        clone.push(&element);
        clone
    }
}


#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
pub struct DunePostTime {
    pub year: String,
    pub month: String,
    pub day: String,
    pub values: (i32, i32, i32),
    pub timestamp: i64
}

#[derive(Debug)]
pub struct DuneAggregationEntry {
    identifier: String,
    count: i32
}

trait Configuration {
}

struct DuneProject;

trait BlogProvider {
    fn posts(&self) -> Vec<BlogPost>;
    fn projects(&self) -> Vec<DuneProject>;
    fn tags(&self) -> Vec<DuneAggregationEntry>;
    fn keywords(&self) -> Vec<DuneAggregationEntry>;
}

struct Database {
    posts: Vec<BlogPost>,
    projects: Vec<DuneProject>,
    tags: Vec<DuneAggregationEntry>,
    keywords: Vec<DuneAggregationEntry>,
    configuration: Rc<Configuration>,
}

#[derive(Debug)]
struct DunePagination {
    /// Name, Title, Route
    current: (i32, String, String),
    /// Name, Title, Route
    next: Option<(i32, String, String)>,
    /// Name, Title, Route
    previous: Option<(i32, String, String)>
}

#[derive(Debug)]
enum DuneAction {
    /// Path, Pagination, Title, Blogpost
    Post(PathBuf, Option<DunePagination>, String, BlogPost),
    /// Path, Paginationi, Title, Posts, Overview?
    List(PathBuf, Option<DunePagination>, String, Vec<BlogPost>, bool),
}


struct Dune {
    database: Rc<Database>,
    receiver: Rc<ActionReceiver>
}

impl Dune {
    fn new<Provider: BlogProvider>(configuration: Rc<Configuration>, provider: Provider) -> Dune {
        Dune {
            database: Rc::new(Database {
                posts: provider.posts(),
                projects: provider.projects(),
                tags: provider.tags(),
                keywords: provider.keywords(),
                configuration: configuration
            }),
            receiver: Rc::new(ActionReceiver::new())
        }
    }

    fn builder(&self) -> Builder {
        let path = PathBuf::from("html");
        let posts: Vec<&BlogPost> = self.database.posts.iter().collect();
        Builder::new(Rc::clone(&self.database), path, posts, Rc::clone(&self.receiver))
    }

    // FIXME: Maybe move this into the database?
    fn post_by_identifier<'a>(&'a self, identifier: &str) -> &'a BlogPost {
        panic!();
    }

    fn execute<Writer: DuneWriter>(&self, writer: &Writer) {
        for action in self.receiver.actions() {
            match action {
                DuneAction::Post(_, _, _, _) => {
                    println!("post action {:?}", action);
                },
                DuneAction::List(_, _, _, _, _) => {
                    println!("list action {:?}", action);
                }
            };
        }
    }
}

// Traits

trait DuneWriter {
    /// A list of items displayed as a list
    fn overview(&self, path: &PathBuf, posts: &[BlogPost]) -> io::Result<()>;
    /// A list of possibly paged items. Displayed as a blog of contents
    fn index(&self, path: &PathBuf, posts: &[BlogPost], page: i32, previous: Option<i32>, next: Option<i32>) -> io::Result<()>;
    /// A single post
    fn post(&self, path: &PathBuf, post: &BlogPost, current: i32, total: i32) -> io::Result<()>;
}

trait DuneRouter {
    fn route_post(post: &BlogPost) -> String;
    fn route_tag(tag: &str) -> String;
    fn route_keyword(keyword: &str) -> String;
    fn overview_pagename<PathBuilder: DunePathBuilder>(builder: &PathBuilder) -> String;
    fn index_pagename<PathBuilder: DunePathBuilder>(builder: &PathBuilder) -> String;
    fn post_pagename<PathBuilder: DunePathBuilder>(builder: &PathBuilder, post: &BlogPost) -> String;

    fn is_overview<PathBuilder: DunePathBuilder>(builder: &PathBuilder, overview: bool) -> String {
        match overview {
            true => Self::overview_pagename(builder),
            false => Self::index_pagename(builder)
        }
    }
}


trait DuneBuildMapper<'a> {
    fn group_by(self, key: DuneBaseAggType) -> GroupedDuneBuilder<'a>;
    fn paged(self, i32) -> PagedDuneBuilder<'a>;
}

trait DuneBuildFlatter<'a> where Self::BuilderType: DuneBuildWriter {
    type CategoryType;
    type BuilderType;
    /// Will iterate over the contents of this collection. For each entry, a new
    /// `Builder` will be created and moved into the grouped `Category`.
    /// I.e. iterating over a structure such as `Vec<("2016", Vec<BlogPost>), ("2017", Vec<BlogPost>)>`
    /// will first create a builder for the sub-route `current/2016` and then for `current/2017`.
    fn with<F>(self, action: F) -> Self where F: (Fn(Self::BuilderType, Self::CategoryType) -> ());
}

trait DuneBuildCollector<'a> {
    fn receive(self, action: DuneAction) -> Self;

    fn collected(&self) -> Vec<&'a BlogPost>;
    fn into_collected(&self) -> Vec<BlogPost> {
        self.collected().into_iter().map(|post|post.clone()).collect()
    }

    fn with_posts<F>(self, action: F) -> Self where F: (Fn(PostBuilder<'a>) -> ());
}

trait DuneBuildWriter {
    fn write<Router: DuneRouter>(self, router: &Router, title: String, overview: bool) -> Self;
    fn clone_to<T: AsRef<Path>>(self, path: T, title: String, overview: bool) -> Self;
}

trait DunePathBuilder {
    fn path(&self) -> &Path;
    fn push<T: AsRef<str>>(mut self, path: T) -> Self;
}

// Types

struct ActionReceiver {
    actions: Cell<Option<Vec<DuneAction>>>,
}

impl ActionReceiver {

    fn new() -> ActionReceiver {
        ActionReceiver {
            actions: Cell::new(Some(Vec::new())),
        }
    }

    fn receive<'a>(&self, mut action: DuneAction) {
        if let Some(mut current) = self.actions.replace(None) {
            current.push(action);
            self.actions.set(Some(current));
        }
    }

    fn actions(&self) -> Vec<DuneAction> {
        if let Some(mut current) = self.actions.replace(None) {
            current.reverse();
            return current;
        }
        Vec::new()
    }
}

struct Builder<'a> {
    payload: Vec<&'a BlogPost>,
    path: PathBuf,
    database: Rc<Database>,
    parent: Rc<ActionReceiver>
}

impl<'a> Builder<'a> {
    fn new(database: Rc<Database>, path: PathBuf, posts: Vec<&'a BlogPost>, parent: Rc<ActionReceiver>) -> Builder<'a> {
        Builder {
            payload: posts,
            path: path,
            database: database,
            parent: parent
        }
    }
}

impl<'a> DuneBuildMapper<'a> for Builder<'a> {
    fn group_by(self, key: DuneBaseAggType) -> GroupedDuneBuilder<'a> {
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
        let builder = GroupedDuneBuilder::new(Rc::clone(&self.database), self.path.clone(), payload, Rc::clone(&self.parent));
        builder
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
        PagedDuneBuilder::new(Rc::clone(&self.database), self.path.clone(), result, Rc::clone(&self.parent))
    }
}

impl<'a> DuneBuildCollector<'a> for Builder<'a> {
    fn receive(self, action: DuneAction) -> Self {
        self.parent.receive(action);
        self
    }
    fn collected(&self) -> Vec<&'a BlogPost> {
        self.payload.clone()
    }
    fn with_posts<F>(self, action: F) -> Self where F: (Fn(PostBuilder<'a>) -> ()) {
        let count = self.payload.len() as i32;
        self.payload.iter().enumerate()
            .map(|(pos, post)|PostBuilder::new(Rc::clone(&self.database), self.path.clone(), (post, pos as i32, count), Rc::clone(&self.parent)))
            .for_each(|builder| action(builder));
        self
    }
}

impl<'a> DunePathBuilder for Builder<'a> {
    fn push<T: AsRef<str>>(mut self, path: T) -> Self {
        self.path.push(path.as_ref());
        self
    }
    fn path(&self) -> &Path {
        &self.path
    }
}

impl<'a> DuneBuildWriter for Builder<'a> {
    fn write<Router: DuneRouter>(self, router: &Router, title: String, overview: bool) -> Self {
        let path = self.path.appending(&Router::is_overview(&self, overview));
        let posts = self.into_collected();
        self.receive(DuneAction::List(path, None, title, posts, true))
    }

    fn clone_to<T: AsRef<Path>>(self, path: T, title: String, overview: bool) -> Self {
        let posts = self.into_collected();
        self.receive(DuneAction::List(path.as_ref().to_path_buf(), None, title, posts, true))
    }
}

struct PostBuilder<'a> {
    payload: (&'a BlogPost, i32, i32),
    path: PathBuf,
    database: Rc<Database>,
    parent: Rc<ActionReceiver>
}

impl<'a> PostBuilder<'a> {
    fn new(database: Rc<Database>, path: PathBuf, payload: (&'a BlogPost, i32, i32), parent: Rc<ActionReceiver>) -> PostBuilder<'a> {
        PostBuilder {
            payload,
            path,
            database,
            parent
        }
    }

    fn post(&self) -> &'a BlogPost {
        self.payload.0
    }

    /// Write the post at the current path using the given filename
    fn write_post<Router: DuneRouter>(self, router: &Router, title: String) -> Self {
        let path = self.path.appending(&Router::post_pagename(&self, &self.payload.0));
        let pagination = DunePagination {
            current: (self.payload.1, format!("fake"), format!("fake")),
            next: None,
            previous: None
        };
        let action = DuneAction::Post(path, Some(pagination), title, self.payload.0.clone());
        self.receive(action)
    }
}

impl<'a> DunePathBuilder for PostBuilder<'a> {
    fn push<T: AsRef<str>>(mut self, path: T) -> Self {
        self.path.push(path.as_ref());
        self
    }
    fn path(&self) -> &Path {
        &self.path
    }
}

impl<'a> DuneBuildCollector<'a> for PostBuilder<'a> {
    fn receive(self, action: DuneAction) -> Self {
        self.parent.receive(action);
        self
    }

    fn collected(&self) -> Vec<&'a BlogPost> {
        vec![self.payload.0]
    }

    fn with_posts<F>(self, action: F) -> Self where F: (Fn(PostBuilder<'a>) -> ()) {
        /// This implementation literally does nothing
        self
    }
}



struct GroupedDuneBuilder<'a> {
    database: Rc<Database>,
    payload: Vec<(String, Vec<&'a BlogPost>)>,
    path: PathBuf,
    parent: Rc<ActionReceiver>
}

impl<'a> GroupedDuneBuilder<'a> {
    fn new(database: Rc<Database>, path: PathBuf, payload: Vec<(String, Vec<&'a BlogPost>)>, parent: Rc<ActionReceiver>) -> GroupedDuneBuilder<'a> {
        GroupedDuneBuilder {
            database,
            payload,
            path,
            parent: parent
        }
    }
}

impl<'a> DuneBuildCollector<'a> for GroupedDuneBuilder<'a> {
    fn receive(self, action: DuneAction) -> Self {
        self.parent.receive(action);
        self
    }

    fn collected(&self) -> Vec<&'a BlogPost> {
        let mut result: Vec<&BlogPost> = Vec::new();
        for &(_, ref posts) in self.payload.iter() {
            for post in posts {
                result.push(post);
            }
        }
        result
    }
    fn with_posts<F>(self, action: F) -> Self where F: (Fn(PostBuilder<'a>) -> ()) {
        let count = self.collected().len() as i32;
        self.collected().iter().enumerate()
            .map(|(pos, post)|PostBuilder::new(Rc::clone(&self.database), self.path.clone(), (post, pos as i32, count), Rc::clone(&self.parent)))
            .for_each(|builder| action(builder));
        self
    }
}

impl<'a> DuneBuildFlatter<'a> for GroupedDuneBuilder<'a> {
    type CategoryType = String;
    type BuilderType = Builder<'a>;
    fn with<F>(self, action: F) -> Self where F: (Fn(Builder<'a>, Self::CategoryType) -> ()) {
        for &(ref key, ref posts) in self.payload.iter() {
            let mut path = self.path.clone();
            path.push(&key);
            let inner_builder = Builder::new(Rc::clone(&self.database), path, posts.clone(), Rc::clone(&self.parent));
            action(inner_builder, key.to_owned());
        }
        self
    }
}

impl<'a> DuneBuildWriter for GroupedDuneBuilder<'a> {
    fn write<Router: DuneRouter>(self, router: &Router, title: String, overview: bool) -> Self {
        let path = self.path.appending(&Router::is_overview(&self, overview));
        let posts = self.into_collected();
        self.receive(DuneAction::List(path, None, title, posts, true))
    }

    fn clone_to<T: AsRef<Path>>(self, path: T, title: String, overview: bool) -> Self {
        let posts = self.into_collected();
        self.receive(DuneAction::List(path.as_ref().to_path_buf(), None, title, posts, true))
    }
}

impl<'a> DunePathBuilder for GroupedDuneBuilder<'a> {
    fn push<T: AsRef<str>>(mut self, path: T) -> Self {
        self.path.push(path.as_ref());
        self
    }
    fn path(&self) -> &Path {
        &self.path
    }
}

#[derive(Debug, Clone)]
struct DunePage<'a> {
    page: i32,
    previous_page: Option<i32>,
    next_page: Option<i32>,
    posts: Vec<&'a BlogPost>
}

struct PagedDuneBuilder<'a> {
    database: Rc<Database>,
    payload: Vec<DunePage<'a>>,
    path: PathBuf,
    parent: Rc<ActionReceiver>
}

impl<'a> PagedDuneBuilder<'a> {
    fn new(database: Rc<Database>, path: PathBuf, payload: Vec<DunePage<'a>>, parent: Rc<ActionReceiver>) -> PagedDuneBuilder {
        PagedDuneBuilder {
            database,
            payload,
            path,
            parent
        }
    }
}

impl<'a> DuneBuildFlatter<'a> for PagedDuneBuilder<'a> {
    type CategoryType = (i32, Option<i32>, Option<i32>);
    type BuilderType = PageDuneBuilder<'a>;
    fn with<F>(self, action: F) -> Self where F: (Fn(PageDuneBuilder<'a>, Self::CategoryType) -> ())
    {
        for (idx, ref page) in self.payload.iter().enumerate() {
            let mut path = self.path.clone();
            let key = &format!("{}", page.page);
            path.push(&key);
            let inner_builder = PageDuneBuilder {
                database: Rc::clone(&self.database),
                payload: self.payload.clone(),
                index: idx,
                path: path,
                parent: Rc::clone(&self.parent)
            };
            action(inner_builder, (0, None, None));
        }
        self
    }

}

impl<'a> DuneBuildWriter for PagedDuneBuilder<'a> {
    fn write<Router: DuneRouter>(self, router: &Router, title: String, overview: bool) -> Self {
        let path = self.path.appending(&Router::is_overview(&self, overview));
        let posts = self.into_collected();
        self.receive(DuneAction::List(path, None, title, posts, true))
    }

    fn clone_to<T: AsRef<Path>>(self, path: T, title: String, overview: bool) -> Self {
        let posts = self.into_collected();
        self.receive(DuneAction::List(path.as_ref().to_path_buf(), None, title, posts, true))
    }
}

impl<'a> DuneBuildCollector<'a> for PagedDuneBuilder<'a> {
    fn receive(self, action: DuneAction) -> Self {
        self.parent.receive(action);
        self
    }

    fn collected(&self) -> Vec<&'a BlogPost> {
        let mut result: Vec<&BlogPost> = Vec::new();
        for page in self.payload.iter() {
            for post in &page.posts {
                result.push(post);
            }
        }
        result
    }

    fn with_posts<F>(self, action: F) -> Self where F: (Fn(PostBuilder<'a>) -> ()) {
        let count = self.collected().len() as i32;
        self.collected().iter().enumerate()
            .map(|(pos, post)|PostBuilder::new(Rc::clone(&self.database), self.path.clone(), (post, pos as i32, count), Rc::clone(&self.parent)))
            .for_each(|builder| action(builder));
        self
    }
}

impl<'a> DunePathBuilder for PagedDuneBuilder<'a> {
    fn push<T: AsRef<str>>(mut self, path: T) -> Self {
        self.path.push(path.as_ref());
        self
    }
    fn path(&self) -> &Path {
        &self.path
    }
}

struct PageDuneBuilder<'a> {
    database: Rc<Database>,
    payload: Vec<DunePage<'a>>,
    index: usize,
    path: PathBuf,
    parent: Rc<ActionReceiver>
}

impl<'a> DuneBuildWriter for PageDuneBuilder<'a> {
    fn write<Router: DuneRouter>(self, router: &Router, title: String, overview: bool) -> Self {
        let path = self.path.appending(&Router::is_overview(&self, overview));
        let posts = self.into_collected();
        let pagination = DunePagination { current: (self.payload[self.index].page, "fake1".to_string(), "fake2".to_string()),
                                          next: None,
                                          previous: None
        };
        self.receive(DuneAction::List(path, Some(pagination), title, posts, overview))
    }

    // FIXME: This, I might be able to make a trait function
    fn clone_to<T: AsRef<Path>>(self, path: T, title: String, overview: bool) -> Self {
        let posts = self.into_collected();
        self.receive(DuneAction::List(path.as_ref().to_path_buf(), None, title, posts, true))
    }
}

impl<'a> DuneBuildCollector<'a> for PageDuneBuilder<'a> {
    fn receive(self, action: DuneAction) -> Self {
        self.parent.receive(action);
        self
    }

    fn collected(&self) -> Vec<&'a BlogPost> {
        self.payload[self.index].posts.clone()
    }

    fn with_posts<F>(self, action: F) -> Self where F: (Fn(PostBuilder<'a>) -> ()) {
        let count = self.collected().len() as i32;
        self.collected().iter().enumerate()
            .map(|(pos, post)|PostBuilder::new(Rc::clone(&self.database), self.path.clone(), (post, pos as i32, count), Rc::clone(&self.parent)))
            .for_each(|builder| action(builder));
        self
    }
}

impl<'a> DunePathBuilder for PageDuneBuilder<'a> {
    fn push<T: AsRef<str>>(mut self, path: T) -> Self {
        self.path.push(path.as_ref());
        self
    }
    fn path(&self) -> &Path {
        &self.path
    }
}


struct HTMLWriter {
    configuration: Rc<Configuration>
}

impl HTMLWriter {
    fn new(configuration: Rc<Configuration>) -> HTMLWriter {
        HTMLWriter {
            configuration
        }
    }
}


impl DuneWriter for HTMLWriter {
    fn overview(&self, path: &PathBuf, posts: &[BlogPost]) -> io::Result<()> {
        println!("writing overview with {} posts to {:?}", posts.len(), &path);
        Ok(())
    }

    fn index(&self, path: &PathBuf, posts: &[BlogPost], page: i32, previous: Option<i32>, next: Option<i32>) -> io::Result<()> {
        println!("writing index-page with {} posts to {:?}", posts.len(), &path);
        Ok(())
    }
    fn post(&self, path: &PathBuf, post: &BlogPost, current: i32, total: i32) -> io::Result<()> {
        println!("writing post {} to {:?}", post.title, &path);
        Ok(())
    }
}

#[test]
fn testing() {

    struct TestProvider {}

    impl BlogProvider for TestProvider {
        fn posts(&self) -> Vec<BlogPost> {
            let posts = vec![BlogPost::new("test1".to_owned(), "2017".to_owned(), "01".to_owned(), "14".to_owned()),
                             BlogPost::new("test2".to_owned(), "2016".to_owned(), "02".to_owned(), "24".to_owned()),
                             BlogPost::new("test3".to_owned(), "2015".to_owned(), "03".to_owned(), "31".to_owned()),
            ];
            posts
        }

        fn projects(&self) -> Vec<DuneProject> {
            Vec::new()
        }

        fn tags(&self) -> Vec<DuneAggregationEntry> {
            Vec::new()
        }

        fn keywords(&self) -> Vec<DuneAggregationEntry> {
            Vec::new()
        }
    }

    struct TestingRouter;
    impl DuneRouter for TestingRouter {
        fn route_post(post: &BlogPost) -> String {
            format!("{}/{}/{}/{}", post.released.year, post.released.month, post.released.day, post.path)
        }
        fn route_tag(tag: &str) -> String {
            format!("tag/{}", tag)
        }
        fn route_keyword(keyword: &str) -> String {
            format!("keyword/{}", keyword)
        }
        fn overview_pagename<PathBuilder: DunePathBuilder>(builder: &PathBuilder) -> String {
            "overview.html".to_owned()
        }
        fn index_pagename<PathBuilder: DunePathBuilder>(builder: &PathBuilder) -> String {
            "index.html".to_owned()
        }
        fn post_pagename<PathBuilder: DunePathBuilder>(builder: &PathBuilder, post: &BlogPost) -> String {
            "index.html".to_owned()
        }
    }

    struct AppventureConfig {}
    impl Configuration for AppventureConfig {
    }

    let configuration = Rc::new(AppventureConfig {});
    // This is so confusing. Doing `Database::new(Rc::clone(&configuration))`
    // will fail. Putting it into its own line, works fine.
    let cloned = Rc::clone(&configuration);
    let provider = TestProvider {};


    let db = Dune::new(cloned, provider);
    let builder = db.builder();

    builder.group_by(DuneBaseAggType::Year)
        .with(|builder, group| {
            builder.group_by(DuneBaseAggType::Month)
                .with(|builder, group| {
                    builder.group_by(DuneBaseAggType::Day)
                        .with(|builder, group| {
                            builder.with_posts(|postbuilder| {
                                let title = &postbuilder.post().path;
                                postbuilder.push(title).write_post(&TestingRouter, title.clone());
                            }).write(&TestingRouter, format!("{}", group), true);
                        }).write(&TestingRouter, format!("{}", group), true);
                }).write(&TestingRouter, format!("{}", group), true);
        });


    let builder = db.builder();
    builder.push("latest-posts")
        .paged(1)
        .with(|builder, (page, previous, next)| {
            let builder = builder.write(&TestingRouter, format!("Page {}", page), false);
            println!("building..");
            if page == 1 {
                builder.clone_to("index.html", format!("Welcome"), false);
            }
        });

    let cloned = Rc::clone(&configuration);
    let writer = HTMLWriter::new(cloned);

    db.execute(&writer);
}
