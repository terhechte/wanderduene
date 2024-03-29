extern crate sha2;
#[macro_use] extern crate askama;
extern crate ammonia;

mod org_parser;

mod harness;
mod configuration;
mod dune_writer;
mod html_writer;
mod dune_base;
mod traits;
mod dune_router;
mod utils;
pub mod dune_post;

pub fn make() {
    //let parser = org_parser::OrgParser::new("/home/terhechte/Development/Rust/rusttest1/posts", 1);
    let parser = org_parser::OrgParser::new("/Users/terhechte/Development/Rust/rusttest1/posts", 1);
    let posts = parser.parse();
    //println!("{:?}", posts);
}
