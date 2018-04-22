use askama::Template;

use std::io;
use std::rc::Rc;
use std::path::{Path, PathBuf};

// FIXME:
use std::error::Error;

use configuration::Configuration;
use dune_writer::*;
use dune_base::DuneBase;
use dune_post::DunePost;

pub struct HTMLWriter {
    configuration: Rc<Configuration>
}

/// The templates that define the HTML.
/// All the shared structures are part of this base html.
#[derive(Template)]
#[template(path = "base.html")]
struct BaseTemplate<'a> {
    base: &'a Rc<DuneBase>,
    config: &'a Rc<Configuration>,
}

/// This template is used for rendering an Index
/// I.e. a list of posts where each post contains the full post
#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate<'a> {
    pagination: &'a Option<DunePagination>,
    posts: &'a Vec<DunePost>,
    _parent: BaseTemplate<'a>
}
/// This template is used for rendering an Overview
/// I.e. a list of posts where each post is only the headline
#[derive(Template)]
#[template(path = "overview.html")]
struct OverviewTemplate<'a> {
    pagination: &'a Option<DunePagination>,
    posts: &'a Vec<DunePost>,
    _parent: BaseTemplate<'a>
}

/// This template is used for rendering a Post
/// I.e. a single blog post
#[derive(Template)]
#[template(path = "post.html")]
struct PostTemplate<'a> {
    pagination: &'a Option<DunePagination>,
    post: &'a DunePost,
    _parent: BaseTemplate<'a>
}

impl HTMLWriter {
    pub fn new(configuration: Rc<Configuration>) -> HTMLWriter {
        HTMLWriter {
            configuration
        }
    }

    fn base_template<'a>(&'a self, base: &'a Rc<DuneBase>) -> BaseTemplate<'a> {
        BaseTemplate {
            base: base,
            config: &self.configuration
        }
    }

    fn create_directory(&self, path: &PathBuf) -> Result<(), Box<Error>> {
        println!("c: {:?}", &path);
        //fs::create_dir(&path);
        Ok(())
    }

    fn create_file(&self, path: &PathBuf, contents: &str) -> Result<(), Box<Error>> {
        //println!("creating file {}", &path);
        //let mut file = fs::File::create(&path)?;
        //file.write_all(contents.as_bytes())?;
        Ok(())
    }
}

impl DuneWriter for HTMLWriter {
    fn write(&self, database: &Rc<DuneBase>, action: &DuneAction) -> io::Result<()> {
        match action {
            &DuneAction::Post(ref path, ref pagination, ref title, ref post) => {
                let structure = PostTemplate {
                    pagination: pagination,
                    post: post,
                    _parent: self.base_template(database)
                };
                // FIXME: Remove unwrap
                let rendered = structure.render().unwrap();
                self.create_directory(path);
                self.create_file(path, &rendered);
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
