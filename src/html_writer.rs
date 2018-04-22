use std::io;
use std::rc::Rc;

use configuration::Configuration;
use dune_writer::*;

pub struct HTMLWriter {
    configuration: Rc<Configuration>
}

impl HTMLWriter {
    pub fn new(configuration: Rc<Configuration>) -> HTMLWriter {
        HTMLWriter {
            configuration
        }
    }
}

impl DuneWriter for HTMLWriter {
    fn write(&self, action: &DuneAction) -> io::Result<()> {
        match action {
            &DuneAction::Post(ref path, ref pagination, ref title, ref post) => {
                println!("Post: {:?}: {:?}", path, title);
            },
            &DuneAction::List(ref path, ref pagination, ref title, ref posts, true) => {
                println!("Overview: {:?}: {:?}", path, title);
            },
            &DuneAction::List(ref path, ref pagination, ref title, ref posts, false) => {
                println!("Index: {:?}: {:?}", path, title);
            }
        };
        Ok(())
    }
}
