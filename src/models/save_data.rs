use std::path::PathBuf;
use std::cell::RefCell;
use std::fs;
use std::io::ErrorKind;

use anyhow::{Error, Result};
use once_cell::sync::OnceCell;

use gtk::prelude::*;
use gtk::gio;

use crate::models::{DataObject, Group};
use crate::widgets::*;

#[derive(Default, Debug)]
pub struct SaveData {
    pub groups: RefCell<Vec<Group>>,
    save_path: PathBuf,

    pub group_model: OnceCell<gio::ListStore>
}

impl SaveData {
    pub fn new(pb: &PathBuf) -> Self {
        match SaveData::load_groups(pb) {
            Ok(groups) => {
                Self {
                    groups: RefCell::new(groups),
                    save_path: pb.to_owned(),
                    group_model: OnceCell::from(gio::ListStore::new(GroupListRowContent::static_type()))
                }
            }
            Err(_) => { panic!("Could not access save data directory"); }
        }
    }

    /// Loads all groups from path
    fn load_groups(pb: &PathBuf) -> Result<Vec<Group>> {
        let mut loaded_groups: Vec<Group> = vec![];

        match fs::read_dir(pb.as_path().join(r"groups")) {
            Ok(group_files) => {
                for file in group_files {
                    let group = Group::load_from_file(&file?.path())?;
                    loaded_groups.push(group);
                }

                Ok(loaded_groups)
            }
            Err(error) => match error.kind() {
                ErrorKind::NotFound => {
                    fs::create_dir_all(pb.as_path().join(r"groups"))?;
                    Ok(vec![])
                }
                _ => { Err(Error::new(error)) }
            }
        }
    }

    /// Initialize groups model based on data loaded from storage
    pub fn init_group_model(&self) {
        for group in self.groups.borrow().iter() {
            let row = GroupListRowContent::new(group);
            self.group_model.get().unwrap().append(&row);
        }
    }

    /// Add new group to groups list.
    pub fn new_group(&self, group: Group) -> Result<()> {
        self.groups.borrow_mut().push(group);

        let stored_group = self.groups.borrow();
        stored_group.last().unwrap()
            .save_to_file(self.save_path.as_path().join(r"groups").as_path())?;

        let row = GroupListRowContent::new(stored_group.last().unwrap());
        self.group_model.get().unwrap().append(&row);

        Ok(())
    }

    /// Save group file after changes
    pub fn save_group(&self, group: &Group) {
        group.save_to_file(self.save_path.as_path().join(r"groups").as_path())
            .expect("Could not save group into file");
    }
}
