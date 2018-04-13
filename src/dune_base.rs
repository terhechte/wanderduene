#![feature(conservative_impl_trait)]

/// This is the core data structure that contains the contents of
/// the site.
pub struct DuneBase<Post: DunePost, Page: DunePage> {
    posts: Vec<Post>,
    pages: Vec<Page>,
    tags: Vec<DuneAggregationEntry>,
    keywords: Vec<DuneAggregationEntry>
}

impl<Post: DunePost, Page: DunePage> DuneBase<Post, Page> {

    pub fn new(posts: Vec<Post>) -> DuneBase<Post, Page> {
        DuneBase {
            posts: posts,
            pages: Vec::new(),
            tags: Vec::new(),
            keywords: Vec::new()
        }
    }

    pub fn posts(&self) -> impl Iterator<Item = &Post> {
        self.posts.iter()
    }
}

/// Each blog post has to conform to this Trait
pub trait DunePost {
    /// The identifier for this post. i.e. `blogstrapped-first`
    fn identifier(&self) -> &str;
    /// The original path on disk. i.e. `/posts/2016-03-04-blogstrapped-first`
    fn path(&self) -> &str;
    /// The title of the post
    fn title(&self) -> &str;
    /// The time when the post was released
    fn released(&self) -> &DunePostTime;
    /// The parsed contents of the post
    fn contents(&self) -> &str;
    /// The tags that this post belongs to
    fn tags(&self) -> &[String];
    /// The keywords for this post
    fn keywords(&self) -> &[String];
    /// The description for this post
    fn description(&self) -> &str;
    /// Is this post enabled?
    fn enabled(&self) -> bool;
}

/// Each custom page has to conform to this Trait
pub trait DunePage {
}

pub struct DunePostTime {
    pub year: String,
    pub month: String,
    pub day: String,
    pub values: (i32, i32, i32),
    pub timestamp: i64
}

/// Aggregation entries
pub struct DuneAggregationEntry {
    identifier: String,
    count: i32
}

/// TEMP start figuring out the structure for the builder
// main archive:
// let byear = dunebase.all_posts.sort.groupby(time.year)
// let builder = Builder("html", router)
// builder.enter_route(time.year)
// builder.write_overview(byear)
//
// let bymonth = byear.groupby(time.month)
// builder.enter_route(time.month)
// builder.write_overview(bymonth)
//
// let byday = byear.groupby(time.day)
// builder.enter_route(time.byday)
// builder.write_overview(byday)
//
// builder.write_posts(byday.posts())

/// END TEMP

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
