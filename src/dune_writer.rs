use std::path::PathBuf;
use std::io;
use std::rc::Rc;

use dune_post::DunePost;

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

pub trait DuneWriter {
    fn write(&self, action: &DuneAction) -> io::Result<()>;
}
