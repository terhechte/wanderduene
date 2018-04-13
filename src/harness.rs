use dune_base::{DuneBase};
use org_post::OrgPost;



#[test]
fn testing() {
    let posts = vec![OrgPost::new("test1".to_owned(), "2017".to_owned(), "05".to_owned(), "04".to_owned()),
                     OrgPost::new("test1".to_owned(), "2016".to_owned(), "05".to_owned(), "04".to_owned()),
                     OrgPost::new("test1".to_owned(), "2015".to_owned(), "05".to_owned(), "04".to_owned()),
    ];
    let dune = DuneBase::new(posts);
    dune.posts().map(|post|println!("post: {}", post.title()));
}
