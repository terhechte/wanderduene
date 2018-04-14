#![feature(nll)]

use std::error::Error;
use std::io;
use std::path::PathBuf;
use std::collections::HashMap;
use std::rc::Rc;
use std::cell::Cell;

// Useful to have
trait VecCellAppendable {
    type Item;
    fn append(&self, mut items: Vec<Self::Item>);
}
impl<T> VecCellAppendable for Cell<Option<Vec<T>>> {
    type Item = T;
    fn append(&self, mut items: Vec<Self::Item>) {
        if let Some(mut current) = self.take() {
            current.append(&mut items);
            self.set(Some(current));
        }
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

trait Configuration {}

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

    fn execute(&self) {
        for action in self.receiver.actions() {
            println!("{:?}", action);
        }
    }
}

// Traits

trait DuneWriter {
    /// A list of items displayed as a list
    fn overview(&self, path: &PathBuf, posts: &[&BlogPost]) -> io::Result<()>;
    /// A list of possibly paged items. Displayed as a blog of contents
    fn index<'a>(&self, path: &PathBuf, page: &DunePage<'a>) -> io::Result<()>;
    /// A single post
    fn post(&self, path: &PathBuf, post: &BlogPost) -> io::Result<()>;
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
    fn write_overview(self) -> Self;
    fn write_index(self) -> Self;
    fn write_posts(self) -> Self;
}

trait DuneBuildCollector<'a> {
    //fn with_posts<'b, F>(self, action: F) -> Self where 'a: 'b, F: (Fn(&'b BlogPost, i32, i32, &PathBuf) -> ());
    fn collected(&self) -> Vec<&BlogPost>;
    fn into_collected(&self) -> Vec<BlogPost> {
        self.collected().into_iter().map(|post|post.clone()).collect()
    }
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

    fn receive<'a>(&self, mut actions: Vec<DuneAction>) {
        if let Some(mut current) = self.actions.replace(None) {
            current.append(&mut actions);
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
    fn collected(&self) -> Vec<&BlogPost> {
        self.payload.clone()
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
        let mut path = self.path.clone();
        path.push("overview.html");
        //println!("writing overview at {:?}", &path);
        let mut items: Vec<DuneAction> = vec![DuneAction::WriteOverview(path, self.into_collected())];
        self.parent.receive(items);
        self
    }

    fn write_index(self) -> Self {
        //println!("writing index at {:?}", &self.path);
        let mut path = self.path.clone();
        path.push("index.html");
        let mut items: Vec<DuneAction> = vec![DuneAction::WriteIndex(path, self.into_collected())];
        self.parent.receive(items);
        self
    }

    fn write_posts(self) -> Self {
        let count = self.payload.len();
        for (current, post) in self.payload.iter().enumerate() {
            let mut path = self.path.clone();
            path.push(&post.identifier);
            path.push("index.html");
            let mut items: Vec<DuneAction> = vec![DuneAction::WritePost(path, post.clone().clone(), current as i32, count as i32)];
            self.parent.receive(items);
        }
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
    fn collected(&self) -> Vec<&BlogPost> {
        let mut result: Vec<&BlogPost> = Vec::new();
        for &(_, ref posts) in self.payload.iter() {
            for post in posts {
                result.push(post);
            }
        }
        result
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
        let mut path = self.path.clone();
        path.push("overview.html");
        {
            let collected = self.into_collected();
            let mut items: Vec<DuneAction> = vec![DuneAction::WriteOverview(self.path.clone(), collected.clone())];
            self.parent.receive(items);
        }
        self
    }

    fn write_index(self) -> Self {
        {
            let collected = self.into_collected();
            let mut items: Vec<DuneAction> = vec![DuneAction::WriteIndex(self.path.clone(), collected.clone())];
            self.parent.receive(items);
        }
        self
    }

    fn write_posts(self) -> Self {
        println!("not implemented yet..");
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
        let mut path = self.path.clone();
        path.push("overview.html");
        {
            let collected = self.into_collected();
            let mut items: Vec<DuneAction> = vec![DuneAction::WriteOverview(self.path.clone(), collected.clone())];
            self.parent.receive(items);
        }
        self
    }

    fn write_index(self) -> Self {
        {
            let collected = self.into_collected();
            let mut items: Vec<DuneAction> = vec![DuneAction::WriteIndex(self.path.clone(), collected.clone())];
            self.parent.receive(items);
        }
        self
    }

    fn write_posts(self) -> Self {
        println!("not implemented yet..");
        self
    }
}

impl<'a> DuneBuildCollector<'a> for PagedDuneBuilder<'a> {
    fn collected(&self) -> Vec<&BlogPost> {
        let mut result: Vec<&BlogPost> = Vec::new();
        for page in self.payload.iter() {
            for post in &page.posts {
                result.push(post);
            }
        }
        result
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
            "".to_owned()
        }
        fn route_tag(tag: &str) -> String {
            "".to_owned()
        }
        fn route_keyword(keyword: &str) -> String {
            "".to_owned()
        }
    }

    struct AppventureConfig {}
    impl Configuration for AppventureConfig {}

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
                            builder.write_posts().write_overview();
                            // FIXME:
                            // allow
                            // .builder.write_overview()
                            // i.e. do not return result for writes
                        }).write_overview();
                }).write_overview();
        }).write_overview();


    let builder = db.builder();
    builder.push("latest-posts")
        .paged(1)
        .with(|builder, page| {
            builder.write_index();
        });

    db.execute();
}
