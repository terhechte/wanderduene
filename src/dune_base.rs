use std::rc::Rc;

use dune_post::DunePost;
use configuration::Configuration;

#[derive(Debug)]
pub struct DuneGroup {
    pub identifier: String,
    pub count: usize,
}

#[derive(Debug)]
pub struct DuneProject;

pub struct DuneBase {
    pub posts: Vec<DunePost>,
    pub projects: Vec<DuneProject>,
    pub tags: Vec<DuneGroup>,
    pub keywords: Vec<DuneGroup>,
    pub configuration: Rc<Configuration>,
}

