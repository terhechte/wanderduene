extern crate sha2;
extern crate askama;

mod org_parser;

//mod harness;
//mod org_parser;
//mod configuration;

pub fn make() {
    let parser = org_parser::OrgParser::new("/home/terhechte/Development/Rust/rusttest1/posts", 1);
    let posts = parser.parse();
    println!("{:?}", posts);
}
