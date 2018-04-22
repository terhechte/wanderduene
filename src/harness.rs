use std::error::Error;
use std::io;
use std::path::{PathBuf, Path};
use std::collections::HashMap;
use std::rc::Rc;
use std::cell::Cell;
use std::marker;

use configuration::Configuration;
use dune_post::DunePost;
use org_parser::OrgParser;


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

#[derive(Copy, Clone)]
pub enum DuneBaseAggType {
    Year, Month, Day, Tag, Keyword, Enabled
}

#[derive(Debug)]
pub struct DuneGroup {
    pub identifier: String,
    pub count: usize,
}

pub struct DuneProject;

struct Database {
    posts: Vec<DunePost>,
    projects: Vec<DuneProject>,
    tags: Vec<DuneGroup>,
    keywords: Vec<DuneGroup>,
    configuration: Rc<Configuration>,
}

#[derive(Debug, Clone)]
struct DunePagination {
    /// Page Number, Identifier
    current: (i32, Option<String>),
    /// Page Number, Identifier
    next: Option<(i32, Option<String>)>,
    /// Page Number, Title, Route
    previous: Option<(i32, Option<String>)>
}

#[derive(Debug)]
enum DuneAction {
    /// Path, Pagination, Title, Blogpost
    Post(PathBuf, Option<DunePagination>, String, DunePost),
    /// Path, Paginationi, Title, Posts, Overview?
    List(PathBuf, Option<DunePagination>, String, Vec<DunePost>, bool),
}

struct Dune {
    database: Rc<Database>,
    receiver: Rc<ActionReceiver>
}

impl Dune {
    fn new(configuration: Rc<Configuration>, posts: Vec<DunePost>, projects: Vec<DuneProject>) -> Dune {
        let tags = Dune::aggregate(&posts, |post| &post.tags);
        let keywords = Dune::aggregate(&posts, |post| &post.keywords);
        Dune {
            database: Rc::new(Database {
                posts: posts,
                projects: projects,
                tags: tags,
                keywords: keywords,
                configuration: configuration
            }),
            receiver: Rc::new(ActionReceiver::new())
        }
    }

    fn aggregate<A>(posts: &[DunePost], with_parser: A) -> Vec<DuneGroup>
    where
        A: Fn(&DunePost) -> &[String]
    {
        let mut tag_map: HashMap<String, Vec<&DunePost>> = HashMap::new();
        for post in posts {
            let entry_tags = with_parser(&post);
            for tag in entry_tags {
                let hash_entry = tag_map.entry(tag.to_string()).or_insert(Vec::new());
                hash_entry.push(&post);
            }
        }
        // build the tags
        let mut result: Vec<DuneGroup> = Vec::new();
        for (key, val) in tag_map.into_iter() {
            result.push(DuneGroup {
                identifier: key,
                count: val.len()
            });
        }
        return result;
    }

    fn builder(&self) -> Builder {
        let path = PathBuf::from(self.database.configuration.html_folder());
        let posts: Vec<&DunePost> = self.database.posts.iter().collect();
        Builder::new(Rc::clone(&self.database), path, posts, Rc::clone(&self.receiver))
    }

    // FIXME: Maybe move this into the database?
    fn post_by_identifier<'a>(&self, identifier: &str) -> &'a DunePost {
        panic!();
    }

    fn execute<Writer: DuneWriter>(&self, writer: &Writer) {
        for action in self.receiver.actions() {
            writer.write(&action);
        }
    }
}

// Traits

trait DuneWriter {
    fn write(&self, action: &DuneAction) -> io::Result<()>;
}

trait DuneRouter {
    fn route_post(post: &DunePost) -> String;
    fn route_tag(tag: &str) -> String;
    fn route_keyword(keyword: &str) -> String;
    fn overview_pagename<PathBuilder: DunePathBuilder>(builder: &PathBuilder) -> String;
    fn index_pagename<PathBuilder: DunePathBuilder>(builder: &PathBuilder) -> String;
    fn post_pagename<PathBuilder: DunePathBuilder>(builder: &PathBuilder, post: &DunePost) -> String;

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

trait DuneBuildFlatter<'a> where Self::BuilderType: DuneBuildWriter<'a> {
    type CategoryType;
    type BuilderType;
    /// Will iterate over the contents of this collection. For each entry, a new
    /// `Builder` will be created and moved into the grouped `Category`.
    /// I.e. iterating over a structure such as `Vec<("2016", Vec<DunePost>), ("2017", Vec<DunePost>)>`
    /// will first create a builder for the sub-route `current/2016` and then for `current/2017`.
    fn with<F>(self, action: F) -> Self where F: (Fn(Self::BuilderType, Self::CategoryType) -> ());
}

trait DuneBuilder {
    fn path(&self) -> &PathBuf;
    fn database(&self) -> &Rc<Database>;
    fn parent(&self) -> &Rc<ActionReceiver>;
}

trait DuneBuildCollector<'a> {
    fn receive(self, action: DuneAction) -> Self;

    fn collected(&self) -> Vec<&'a DunePost>;
    fn into_collected(&self) -> Vec<DunePost> {
        self.collected().into_iter().map(|post|post.clone()).collect()
    }

    fn with_posts<F>(self, action: F) -> Self where F: (Fn(PostBuilder<'a>) -> ()), Self: marker::Sized + DuneBuilder {
        let count = self.collected().len() as i32;
        let collected = self.collected();
        self.collected().iter().enumerate()
            .map(|(pos, post)| {
                PostBuilder {
                    payload: collected.clone(),
                    index: pos,
                    path: self.path().clone(),
                    database: Rc::clone(&self.database()),
                    parent: Rc::clone(&self.parent())
                }
            }).for_each(|builder| action(builder));
        self
    }
}

trait DuneBuildWriter<'a> {
    fn write<Router: DuneRouter>(self, router: &Router, title: String, overview: bool, sorted: bool) -> Self
    where Self: marker:: Sized + DuneBuilder + DuneBuildCollector<'a> + DunePathBuilder
    {
        let path = self.path().appending(&Router::is_overview(&self, overview));
        let posts = self.into_collected();
        self.receive(DuneAction::List(path, None, title, posts, true))
    }

    fn clone_to<T: AsRef<Path>>(self, path: T, title: String, overview: bool, sorted: bool) -> Self
    where Self: marker::Sized + DuneBuilder + DuneBuildCollector<'a> + DunePathBuilder {
        let mut posts = self.into_collected();
        if sorted {
            posts.sort();
        }
        let mut root_path = PathBuf::from(self.database().configuration.html_folder());
        root_path.push(path);
        self.receive(DuneAction::List(root_path, None, title, posts, overview))
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
    payload: Vec<&'a DunePost>,
    path: PathBuf,
    database: Rc<Database>,
    parent: Rc<ActionReceiver>
}

impl<'a> DuneBuilder for Builder<'a> {
    fn path(&self) -> &PathBuf { &self.path }
    fn database(&self) -> &Rc<Database> {&self.database }
    fn parent(&self) -> &Rc<ActionReceiver> { &self.parent }
}

impl<'a> Builder<'a> {
    fn new(database: Rc<Database>, path: PathBuf, posts: Vec<&'a DunePost>, parent: Rc<ActionReceiver>) -> Builder<'a> {
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
        let grouped = self.payload.iter().fold(HashMap::<String, Vec<&DunePost>>::new(), |mut acc, elm| {
            {
                // FIXME: Move this logic as an internal impl on DuneBaseAggType?
                let keys = match key {
                    DuneBaseAggType::Year => vec![elm.released.year.to_string()],
                    DuneBaseAggType::Month => vec![elm.released.month.to_string()],
                    DuneBaseAggType::Day => vec![elm.released.day.to_string()],
                    DuneBaseAggType::Tag => elm.tags.clone(),
                    DuneBaseAggType::Keyword => elm.keywords.clone(),
                    DuneBaseAggType::Enabled => vec![format!("{}", elm.enabled)]
                };
                for key in keys {
                    let mut entry = acc.entry(key).or_insert(
                        Vec::new(),
                    );
                    entry.push(elm);
                }
            }
            acc
        });
        // FIXME: Can I map this?
        let mut payload: Vec<(String, Vec<&'a DunePost>)> = Vec::new();
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
            let entries: Vec<&DunePost> = cloned
                .into_iter()
                .skip((counter * per_page) as usize)
                .take(per_page as usize)
                .collect();
            if entries.is_empty() {
                break;
            }
            let previous = match counter {
                0 => None,
                _ => Some(counter),
            };
            let next = match (((counter + 1) * per_page) as i32) < ((self.payload.len()) as i32) {
                true => Some(counter + 2),
                false => None,
            };
            let current = counter + 1;
            let page = DunePage {
                pagination: DunePagination {
                    current: (current, None),
                    next: next.map(|number|(number, None)),
                    previous: previous.map(|number|(number, None))
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
    fn collected(&self) -> Vec<&'a DunePost> {
        self.payload.clone()
    }
}

impl<'a> DunePathBuilder for Builder<'a> {
    fn push<T: AsRef<str>>(mut self, path: T) -> Self {
        self.path.push(path.as_ref());
        self
    }
}

impl<'a> DuneBuildWriter<'a> for Builder<'a> {}

struct PostBuilder<'a> {
    payload: Vec<&'a DunePost>,
    index: usize,
    path: PathBuf,
    database: Rc<Database>,
    parent: Rc<ActionReceiver>
}

impl<'a> DuneBuilder for PostBuilder<'a> {
    fn path(&self) -> &PathBuf { &self.path }
    fn database(&self) -> &Rc<Database> {&self.database }
    fn parent(&self) -> &Rc<ActionReceiver> { &self.parent }
}

impl<'a> PostBuilder<'a> {
    fn post(&self) -> &'a DunePost {
        self.payload[self.index]
    }

    /// Write the post at the current path using the given filename
    fn write_post<Router: DuneRouter>(self, router: &Router, title: String) -> Self {
        let post = self.payload[self.index];
        let path = self.path.appending(&Router::post_pagename(&self, &post));
        let next = if self.index < (self.payload.len() - 1) { Some(self.index + 1) } else { None };
        let previous = if self.index > 0 { Some(self.index - 1) } else { None };
        let pagination = DunePagination {
            current: (self.index as i32, Some(self.payload[self.index].identifier.clone())),
            next: next.map(|number| (number as i32, Some(self.payload[number].identifier.clone()))),
            previous: previous.map(|number| (number as i32, Some(self.payload[number].identifier.clone()))),
        };
        let action = DuneAction::Post(path, Some(pagination), title, post.clone());
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

    fn collected(&self) -> Vec<&'a DunePost> {
        vec![self.payload[self.index]]
    }

    fn with_posts<F>(self, action: F) -> Self where F: (Fn(PostBuilder<'a>) -> ()) {
        /// This implementation literally does nothing
        self
    }
}



struct GroupedDuneBuilder<'a> {
    database: Rc<Database>,
    payload: Vec<(String, Vec<&'a DunePost>)>,
    path: PathBuf,
    parent: Rc<ActionReceiver>
}

impl<'a> DuneBuilder for GroupedDuneBuilder<'a> {
    fn path(&self) -> &PathBuf { &self.path }
    fn database(&self) -> &Rc<Database> {&self.database }
    fn parent(&self) -> &Rc<ActionReceiver> { &self.parent }
}

impl<'a> GroupedDuneBuilder<'a> {
    fn new(database: Rc<Database>, path: PathBuf, payload: Vec<(String, Vec<&'a DunePost>)>, parent: Rc<ActionReceiver>) -> GroupedDuneBuilder<'a> {
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

    fn collected(&self) -> Vec<&'a DunePost> {
        let mut result: Vec<&DunePost> = Vec::new();
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

impl<'a> DuneBuildWriter<'a> for GroupedDuneBuilder<'a> {}

impl<'a> DunePathBuilder for GroupedDuneBuilder<'a> {
    fn push<T: AsRef<str>>(mut self, path: T) -> Self {
        self.path.push(path.as_ref());
        self
    }
}

#[derive(Debug, Clone)]
struct DunePage<'a> {
    pagination: DunePagination,
    posts: Vec<&'a DunePost>
}

struct PagedDuneBuilder<'a> {
    database: Rc<Database>,
    payload: Vec<DunePage<'a>>,
    path: PathBuf,
    parent: Rc<ActionReceiver>
}

impl<'a> DuneBuilder for PagedDuneBuilder<'a> {
    fn path(&self) -> &PathBuf { &self.path }
    fn database(&self) -> &Rc<Database> {&self.database }
    fn parent(&self) -> &Rc<ActionReceiver> { &self.parent }
}


impl<'a> PagedDuneBuilder<'a> {
    fn new(database: Rc<Database>, path: PathBuf, payload: Vec<DunePage<'a>>, parent: Rc<ActionReceiver>) -> PagedDuneBuilder<'a> {
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
    type BuilderType = PageDuneBuilder<'a>;
    fn with<F>(self, action: F) -> Self where F: (Fn(PageDuneBuilder<'a>, Self::CategoryType) -> ())
    {
        for (idx, ref page) in self.payload.iter().enumerate() {
            let mut path = self.path.clone();
            let key = &format!("{}", page.pagination.current.0);
            path.push(&key);
            let inner_builder = PageDuneBuilder {
                database: Rc::clone(&self.database),
                payload: self.payload.clone(),
                index: idx,
                path: path,
                parent: Rc::clone(&self.parent)
            };
            action(inner_builder, page.pagination.current.0);
        }
        self
    }

}

impl<'a> DuneBuildWriter<'a> for PagedDuneBuilder<'a> {}

impl<'a> DuneBuildCollector<'a> for PagedDuneBuilder<'a> {
    fn receive(self, action: DuneAction) -> Self {
        self.parent.receive(action);
        self
    }

    fn collected(&self) -> Vec<&'a DunePost> {
        let mut result: Vec<&DunePost> = Vec::new();
        for page in self.payload.iter() {
            for post in &page.posts {
                result.push(post);
            }
        }
        result
    }

}

impl<'a> DunePathBuilder for PagedDuneBuilder<'a> {
    fn push<T: AsRef<str>>(mut self, path: T) -> Self {
        self.path.push(path.as_ref());
        self
    }
}

struct PageDuneBuilder<'a> {
    database: Rc<Database>,
    payload: Vec<DunePage<'a>>,
    index: usize,
    path: PathBuf,
    parent: Rc<ActionReceiver>
}

impl<'a> DuneBuilder for PageDuneBuilder<'a> {
    fn path(&self) -> &PathBuf { &self.path }
    fn database(&self) -> &Rc<Database> {&self.database }
    fn parent(&self) -> &Rc<ActionReceiver> { &self.parent }
}

impl<'a> DuneBuildWriter<'a> for PageDuneBuilder<'a> {
    fn write<Router: DuneRouter>(self, router: &Router, title: String, overview: bool, sorted: bool) -> Self {
        let path = self.path.appending(&Router::is_overview(&self, overview));
        let mut posts = self.into_collected();
        if sorted {
            posts.sort();
        }
        let pagination = self.payload[self.index].pagination.clone();
        self.receive(DuneAction::List(path, Some(pagination), title, posts, overview))
    }
}

impl<'a> DuneBuildCollector<'a> for PageDuneBuilder<'a> {
    fn receive(self, action: DuneAction) -> Self {
        self.parent.receive(action);
        self
    }

    fn collected(&self) -> Vec<&'a DunePost> {
        self.payload[self.index].posts.clone()
    }

}

impl<'a> DunePathBuilder for PageDuneBuilder<'a> {
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
    fn write(&self, action: &DuneAction) -> io::Result<()> {
        match action {
            &DuneAction::Post(ref path, ref pagination, ref title, ref post) => {
                println!("Post: {:?}: {:?}", path, title);
            },
            &DuneAction::List(ref path, ref pagination, ref title, ref posts, true) => {
                println!("Overview: {:?}: {:?}", path, title);
            },
            &DuneAction::List(ref path, ref pagination, ref title, ref posts, false) => {
                println!("Index: {:?}: {:?}", path, title);
            }
        };
        Ok(())
    }
}

#[test]
fn testing() {

    struct TestingRouter;
    impl DuneRouter for TestingRouter {
        fn route_post(post: &DunePost) -> String {
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
        fn post_pagename<PathBuilder: DunePathBuilder>(builder: &PathBuilder, post: &DunePost) -> String {
            "index.html".to_owned()
        }
    }

    struct AppventureConfig {}
    impl Configuration for AppventureConfig {
        fn html_folder(&self) -> &str {
            "html"
        }
        fn cache_file(&self) -> &Path {
            Path::new("./cache_file.cache")
        }
        fn post_folder(&self) -> &Path {
            Path::new("/Users/terhechte/Development/Rust/rusttest1/posts")
        }
    }

    let configuration = Rc::new(AppventureConfig {});
    // This is so confusing. Doing `Database::new(Rc::clone(&configuration))`
    // will fail. Putting it into its own line, works fine.
    let cloned = Rc::clone(&configuration);

    let parser = OrgParser::new(&cloned.post_folder(), 2);
    let posts = parser.parse();

    let db = Dune::new(cloned, posts, Vec::new());
    let builder = db.builder();

    builder.group_by(DuneBaseAggType::Year)
        .with(|builder, year| {
            builder.group_by(DuneBaseAggType::Month)
                .with(|builder, month| {
                    builder.group_by(DuneBaseAggType::Day)
                        .with(|builder, day| {
                            builder.with_posts(|postbuilder| {
                                let title = &postbuilder.post().path;
                                postbuilder.push(title).write_post(&TestingRouter, title.clone());
                            }).write(&TestingRouter, format!("{} {} {}", year, month, day), true, true);
                        }).write(&TestingRouter, format!("{} {}", year, month), true, true);
                }).write(&TestingRouter, format!("{}", year), true, true);
        });

    let builder = db.builder();
    builder.push("tags")
        .group_by(DuneBaseAggType::Tag)
        .with(|builder, tag| {
            let count = &builder.collected().len();
            builder.write(&TestingRouter, format!("{} {}", tag, count), true, true);
        });


    let builder = db.builder();
    builder.push("latest-posts")
        .paged(1)
        .with(|builder, page| {
            let builder = builder.write(&TestingRouter, format!("Page {}", page), false, true);
            if page == 1 {
                builder.clone_to("index.html", format!("Welcome"), false, true);
            }
        });

    let cloned = Rc::clone(&configuration);
    let writer = HTMLWriter::new(cloned);

    db.execute(&writer);
}
