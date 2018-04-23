use std::path::PathBuf;
use std::io;
use std::rc::Rc;
use std::fmt;
use std::io::prelude::*;

use dune_post::DunePost;
use dune_base::DuneBase;
use dune_router::DuneRouter;

#[derive(Debug, Clone)]
pub struct DunePagination {
    /// Page Number, Identifier
    pub current: (i32, Option<String>),
    /// Page Number, Identifier
    pub next: Option<(i32, Option<String>)>,
    /// Page Number, Title, Route
    pub previous: Option<(i32, Option<String>)>
}

#[derive(Debug)]
pub enum DuneAction {
    /// Path, Pagination, Title, Blogpost
    Post(PathBuf, Option<DunePagination>, String, DunePost),
    /// Path, Paginationi, Title, Posts, Overview?
    List(PathBuf, Option<DunePagination>, String, Vec<DunePost>, bool),
}

impl fmt::Display for DuneAction {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), ::std::fmt::Error> {
        match self {
            &DuneAction::Post(ref path, _, _, _) => write!(f, "{:?}", &path),
            &DuneAction::List(ref path, _, _, _, _) => write!(f, "{:?}", &path)
        };
        Ok(())
    }
}

pub trait DuneWriter {
    fn write<Router: DuneRouter>(&self, database: &Rc<DuneBase>, action: &DuneAction, router: &Router) -> io::Result<()>;
}
