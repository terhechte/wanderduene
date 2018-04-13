#![feature(conservative_impl_trait)]
use itertools::{Itertools, GroupBy};

use std::marker::Sized;
use std::collections::HashMap;

/// This is the core data structure that contains the contents of
/// the site.
pub struct DuneBase<Post: DunePost, Page: DunePage> {
    posts: Vec<Post>,
    pages: Vec<Page>,
    tags: Vec<DuneAggregationEntry>,
    keywords: Vec<DuneAggregationEntry>
}

impl<Post: DunePost, Page: DunePage> DuneBase<Post, Page> {

    pub fn new(posts: Vec<Post>, pages: Vec<Page>) -> DuneBase<Post, Page> {
        DuneBase {
            posts: posts,
            pages: pages,
            tags: Vec::new(),
            keywords: Vec::new()
        }
    }

    pub fn posts(&self) -> impl Iterator<Item = &Post> {
        self.posts.iter()
    }
}

pub trait DuneBaseCompat<Post: DunePost> where Self: Iterator<Item = Post>, Self: Sized {
    fn consume(self) -> Vec<Post> {
        // FIXME: This should replace the posts, the posts should be in a Cell
        let mut output: Vec<Post> = self.collect();
        output.sort_by(|a, b|a.released().timestamp.cmp(&b.released().timestamp));
        output
    }

    // Year, Month, Day, Tag, Keyword, Enabled
    fn agg(self, by: DuneBaseAggType)
        /*GroupBy<String, Self, Post>*/
    {
        /*self.group_by(|a: Post|a.released().timestamp)*/
        self.fold(HashMap::<String, Vec<Post>>::new(), |mut acc: HashMap<String, Vec<Post>>, elm: Post| {
            {
                let key = match by {
                    DuneBaseAggType::Year => elm.released().year.to_string(),
                    _ => elm.title().to_string()
                };
                let mut entry = acc.entry(key).or_insert(
                    Vec::new(),
                );
                entry.push(elm);
            }
            acc
        });
    }
}

/*impl DuneBaseCompat for Iterator<Item = DunePost> {
    type Post = Self::Item;
    fn all() -> Vec<Self::Post> {
        Vec::new()
    }
}*/

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

#[derive(Copy, Clone)]
pub enum DuneBaseAggType {
    Year, Month, Day, Tag, Keyword, Enabled
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

// next test:
// let writer = htmlwriter();
// let builder = dunebase.builder(writer, router, config);
// the year / month / day / title case might need a trickier solution
// builder.groupby(year).sort().with(|builder, group| {
//   builder.groupby(month).sort().with(|builder, group| {
//      builder.posts().build()
//   }).overview().build()
// })

// archive:
// builder.paged().with(|builder, page|) {
//   
// }

// simpler case: tags
// let builder = dunebase.builder(config);
// builder.

// final case. write year/month/day/ and then pages per day!

/**
DuneBuildable:
- group_by
- paged

DuneConsumable:
- consume

DuneWritable:
- write_overview
- write_posts
- write_index

DuneGrouped:
- with

DuneBuilder
DuneBuilderPage
DuneBuilderGroup

DuneBuildAdapter (fn apply)?

*/

/// END TEMP

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
