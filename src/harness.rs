#![feature(nll)]

use std::error::Error;
use std::io;
use std::path::{PathBuf, Path};
use std::collections::HashMap;
use std::rc::Rc;
use std::cell::Cell;

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
    fn overview_pagename(&self) -> &'static str;
    fn index_pagename(&self) -> &'static str;
    fn post_pagename(&self) -> &'static str;
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
enum DuneAction {
    WriteOverview(PathBuf, Vec<BlogPost>),
    WriteIndex(PathBuf, Vec<BlogPost>),
    WritePost(PathBuf, BlogPost, i32, i32),
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

    fn execute<Writer: DuneWriter>(&self, writer: &Writer) {
        for action in self.receiver.actions() {
            match action {
                DuneAction::WritePost(path, post, current, total) => writer.post(&path, &post, current, total),
                // DuneAction::WriteIndex(path, posts) => writer.index(&path, &posts),
                DuneAction::WriteOverview(path, posts) => writer.overview(&path, &posts),
                _ => Ok(())
            };
        }
    }
}

// Traits

trait DuneWriter {
    /// A list of items displayed as a list
    fn overview(&self, path: &PathBuf, posts: &[BlogPost]) -> io::Result<()>;
    /// A list of possibly paged items. Displayed as a blog of contents
    fn index<'a>(&self, path: &PathBuf, page: &DunePage<'a>) -> io::Result<()>;
    /// A single post
    fn post(&self, path: &PathBuf, post: &BlogPost, current: i32, total: i32) -> io::Result<()>;
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
    /// Will iterate over the contents of this collection. For each entry, a new
    /// `Builder` will be created and moved into the grouped `Category`.
    /// I.e. iterating over a structure such as `Vec<("2016", Vec<BlogPost>), ("2017", Vec<BlogPost>)>`
    /// will first create a builder for the sub-route `current/2016` and then for `current/2017`.
    fn with<F>(self, action: F) -> Self where F: (Fn(Builder<'a>, Self::CategoryType) -> ());
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
    fn write_overview(self) -> Self;
    fn write_index(self) -> Self;
}


trait DunePathBuilder {
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
}

impl<'a> DuneBuildWriter for Builder<'a> {
    fn write_overview(self) -> Self {
        let path = self.path.appending(self.database.configuration.overview_pagename());
        let posts = self.into_collected();
        self.receive(DuneAction::WriteOverview(path, posts))
    }

    fn write_index(self) -> Self {
        let path = self.path.appending(self.database.configuration.index_pagename());
        let posts = self.into_collected();
        self.receive(DuneAction::WriteIndex(path, posts))
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
    fn write_post<Router: DuneRouter>(self, router: &Router) -> Self {
        // FIXME: Move the routing from the configuration into the router trait object
        println!("router says: {}", &Router::route_tag("testtest"));
        let path = self.path.appending(self.database.configuration.index_pagename());
        let action = DuneAction::WritePost(path, self.payload.0.clone(), self.payload.1, self.payload.2);
        self.receive(action)
    }
}

impl<'a> DunePathBuilder for PostBuilder<'a> {
    fn push<T: AsRef<str>>(mut self, path: T) -> Self {
        self.path.push(path.as_ref());
        self
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
    fn write_overview(self) -> Self {
        let path = self.path.appending(self.database.configuration.overview_pagename());
        let posts = self.into_collected();
        self.receive(DuneAction::WriteOverview(path, posts))
    }

    fn write_index(self) -> Self {
        let path = self.path.appending(self.database.configuration.index_pagename());
        let posts = self.into_collected();
        self.receive(DuneAction::WriteIndex(path, posts))
    }
}

impl<'a> DunePathBuilder for GroupedDuneBuilder<'a> {
    fn push<T: AsRef<str>>(mut self, path: T) -> Self {
        self.path.push(path.as_ref());
        self
    }
}

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
    type CategoryType = i32;
    fn with<F>(self, action: F) -> Self where F: (Fn(Builder<'a>, Self::CategoryType) -> ()) {
        for page in self.payload.iter() {
            let mut path = self.path.clone();
            let key = &format!("{}", page.page);
            path.push(&key);
            let inner_builder = Builder::new(Rc::clone(&self.database), path, page.posts.clone(), Rc::clone(&self.parent));
            action(inner_builder, page.page);
        }
        self
    }
}

impl<'a> DuneBuildWriter for PagedDuneBuilder<'a> {
    fn write_overview(self) -> Self {
        let path = self.path.appending(self.database.configuration.overview_pagename());
        let posts = self.into_collected();
        self.receive(DuneAction::WriteOverview(path, posts))
    }

    fn write_index(self) -> Self {
        let path = self.path.appending(self.database.configuration.index_pagename());
        let posts = self.into_collected();
        self.receive(DuneAction::WriteIndex(path, posts))
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

    fn index<'b>(&self, path: &PathBuf, page: &DunePage<'b>) -> io::Result<()> {
        println!("writing index-page with {} posts to {:?}", page.posts.len(), &path);
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
            "post/".to_owned()
        }
        fn route_tag(tag: &str) -> String {
            "tag/".to_owned()
        }
        fn route_keyword(keyword: &str) -> String {
            "keyword/".to_owned()
        }
    }

    struct AppventureConfig {}
    impl Configuration for AppventureConfig {
        fn overview_pagename(&self) -> &'static str {
            "overview.html"
        }
        fn index_pagename(&self) -> &'static str {
            "index.html"
        }
        fn post_pagename(&self) -> &'static str {
            "index.html"
        }
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
                                postbuilder.push(title).write_post(&TestingRouter);
                            }).write_overview();
                        }).write_overview();
                }).write_overview();
        }).write_overview();


    let builder = db.builder();
    builder.push("latest-posts")
        .paged(1)
        .with(|builder, page| {
            builder.write_index();
        });

    let cloned = Rc::clone(&configuration);
    let writer = HTMLWriter::new(cloned);

    db.execute(&writer);
}
