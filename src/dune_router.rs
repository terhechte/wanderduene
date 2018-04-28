use traits::*;
use dune_post::DunePost;

pub trait DuneRouter {
    fn post(post: &DunePost) -> String;
    fn tag(tag: &str) -> String;
    fn keyword(keyword: &str) -> String;
    // FIXME: This setup doesn't fly. too restricted... maybe
    // the routes are build during the building?
    fn page(folder: &str, page: &i32) -> String;
    fn overview_pagename<PathBuilder: DunePathBuilder>(builder: &PathBuilder) -> String;
    fn index_pagename<PathBuilder: DunePathBuilder>(builder: &PathBuilder) -> String;
    fn post_pagename<PathBuilder: DunePathBuilder>(builder: &PathBuilder, post: &DunePost) -> String;

    fn is_overview<PathBuilder: DunePathBuilder>(builder: &PathBuilder, overview: bool) -> String {
        match overview {
            true => Self::overview_pagename(builder),
            false => Self::index_pagename(builder)
        }
    }
}
