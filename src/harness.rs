use dune_base::*;
use std::error::Error;
use std::io;
use std::path::PathBuf;
//use org_post::OrgPost;

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

struct BlogPost {
    title: String,
}

impl BlogPost {
    fn new() -> BlogPost {
        BlogPost {
            title: "yeah".to_string(),
        }
    }
}

struct Database {
    posts: Vec<BlogPost>
}

impl Database {
    fn new() -> Database {
        Database {
            posts: vec![BlogPost::new(), BlogPost::new()],
        }
    }
    fn builder(&self) -> Builder {
        let path = PathBuf::from("html");
        Builder::new(path, &self.posts)
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
    fn write_posts(self) -> io::Result<()>;
    fn write_index(self) -> io::Result<()>;
}

// Types

struct Builder<'a> {
    payload: &'a Vec<BlogPost>,
    path: PathBuf
}

impl<'a> Builder<'a> {
    fn new(path: PathBuf, posts: &'a Vec<BlogPost>) -> Builder {
        Builder {
            payload: posts,
            path: path
        }
    }
}

impl<'a> DuneBuildMapper<'a> for Builder<'a> {
    fn group_by(self, key: DuneBaseAggType) -> GroupedDuneBuilder<'a> {
        GroupedDuneBuilder {
            payload: vec![("test".to_string(), self.payload)],
            path: self.path.clone()
        }
    }
    fn paged(self) -> PagedDuneBuilder {
        PagedDuneBuilder {}
    }
}

impl<'a> DuneBuildWriter for Builder<'a> {
    fn write_overview(self) -> io::Result<()> {
        println!("writing overview at {:?}", &self.path);
        Ok(())
    }
    fn write_posts(self) -> io::Result<()> {
        println!("writing posts at {:?}", &self.path);
        Ok(())
    }
    fn write_index(self) -> io::Result<()> {
        println!("writing index at {:?}", &self.path);
        Ok(())
    }
}

struct GroupedDuneBuilder<'a> {
    payload: Vec<(String, &'a Vec<BlogPost>)>,
    path: PathBuf,
}

impl<'a> DuneBuildFlatter<'a> for GroupedDuneBuilder<'a> {
    type CategoryType = String;
    fn with<F>(self, action: F) -> Self where F: (Fn(Builder<'a>, Self::CategoryType) -> ()) {
        for (key, posts) in self.payload.iter() {
            let mut path = self.path.clone();
            path.push(&key);
            let inner_builder = Builder::new(path, posts);
            action(inner_builder, key.to_owned());
        }
        self
    }
}

impl<'a> DuneBuildWriter for GroupedDuneBuilder<'a> {
    fn write_overview(self) -> io::Result<()> {
        println!("writing overview at {:?}", &self.path);
        Ok(())
    }
    fn write_posts(self) -> io::Result<()> {
        println!("writing posts at {:?}", &self.path);
        Ok(())
    }
    fn write_index(self) -> io::Result<()> {
        println!("writing index at {:?}", &self.path);
        Ok(())
    }
}

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
                    builder.write_posts().unwrap();
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
