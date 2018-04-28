use askama::Template;

use std::io;
use std::fs;
use std::io::prelude::*;
use std::rc::Rc;
use std::path::{Path, PathBuf};
use std::marker::PhantomData;

// FIXME:
use std::error::Error;

use configuration::Configuration;
use dune_writer::*;
use dune_base::DuneBase;
use dune_post::DunePost;
use dune_router::DuneRouter;

use std::ops::Deref;

pub struct HTMLWriter {
    configuration: Rc<Configuration>
}

/// A better wrapper around the pagination for usage within a template
/*enum Pagination {
    None,

}*/

/// The templates that define the HTML.
/// All the shared structures are part of this base html.
#[derive(Template)]
#[template(path = "base.html")]
struct BaseTemplate<'a, Router> where Router: 'a + DuneRouter {
    base: &'a Rc<DuneBase>,
    config: &'a Rc<Configuration>,
    router: RouterWraper<Router>
}

/// This template is used for rendering an Index
/// I.e. a list of posts where each post contains the full post
#[derive(Template)]
#[template(path = "index.html", escape = "none")]
struct IndexTemplate<'a, Router> where Router: 'a + DuneRouter {
    pagination: &'a Option<DunePagination>,
    posts: &'a Vec<DunePost>,
    _parent: BaseTemplate<'a, Router>
}

/*
impl<'a, Router> IndexTemplate<'a, Router> where Router: 'a + DuneRouter {
    pub fn next(&self) -> Option<i32> {
        self.pagination.clone().and_then(|p|p.next).map(|p|p.0)
    }
}*/

/// This template is used for rendering an Overview
/// I.e. a list of posts where each post is only the headline
#[derive(Template)]
#[template(path = "overview.html", escape = "none")]
struct OverviewTemplate<'a, Router> where Router: 'a + DuneRouter {
    pagination: &'a Option<DunePagination>,
    posts: &'a Vec<DunePost>,
    _parent: BaseTemplate<'a, Router>
}

/// This template is used for rendering a Post
/// I.e. a single blog post
#[derive(Template)]
#[template(path = "post.html", escape = "none")]
struct PostTemplate<'a, Router> where Router: 'a + DuneRouter {
    pagination: &'a Option<DunePagination>,
    post: &'a DunePost,
    _parent: BaseTemplate<'a, Router>
}

// MOVE ROUTER INTO A NEWTYPE WRAPPRE?
struct RouterWraper<T: DuneRouter> {
    __router: PhantomData<T>
}

impl<T> RouterWraper<T> where T: DuneRouter {
    fn post(&self, post: &DunePost) -> String {
        T::post(post)
    }
    fn tag(&self, tag: &str) -> String {
        T::tag(tag)
    }
    fn page(&self, folder: &str, page: &i32) -> String {
        T::page(folder, page)
    }
    fn keyword(&self, keyword: &str) -> String {
        T::keyword(keyword)
    }
}

impl HTMLWriter {
    pub fn new(configuration: Rc<Configuration>) -> HTMLWriter {
        HTMLWriter {
            configuration
        }
    }

    fn base_template<'a, Router: DuneRouter>(&'a self, base: &'a Rc<DuneBase>, router: &'a Router) -> BaseTemplate<'a, Router> {
        BaseTemplate {
            base: base,
            config: &self.configuration,
            router: RouterWraper {
                __router: PhantomData::<Router>
            }
        }
    }

    fn create_file(&self, path: &Path, contents: &str) -> Result<(), Box<Error>> {
        let mut file = fs::File::create(&path)?;
        file.write_all(contents.as_bytes())?;
        Ok(())
    }
}

impl DuneWriter for HTMLWriter {
    fn write<Router: DuneRouter>(&self, database: &Rc<DuneBase>, action: &DuneAction, router: &Router) -> io::Result<()> {
        match action {
            &DuneAction::Post(ref path, ref pagination, ref title, ref post) => {
                let structure = PostTemplate {
                    pagination: pagination,
                    post: post,
                    _parent: self.base_template(database, router)
                };
                // FIXME: Remove unwrap
                let rendered = structure.render().unwrap();
                fs::create_dir_all(path.parent().unwrap());
                self.create_file(path, &rendered);
            },
            &DuneAction::List(ref path, ref pagination, ref title, ref posts, overview) => {
                println!("path: {:?}, pag: {}", &path, &pagination.is_some());
                let base = self.base_template(database, router);
                let rendered = match overview {
                    false => IndexTemplate {
                        pagination: pagination,
                        posts: posts,
                        _parent: base
                    }.render().unwrap(),
                    true => OverviewTemplate {
                        pagination: pagination,
                        posts: posts,
                        _parent: base
                    }.render().unwrap()
                };
                fs::create_dir_all(path.parent().unwrap());
                self.create_file(path, &rendered);
            },
        };
        Ok(())
    }

}
