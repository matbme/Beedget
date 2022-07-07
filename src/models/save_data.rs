use std::path::PathBuf;
use std::cell::RefCell;
use std::fs;
use std::io::ErrorKind;

use anyhow::{Error, Result};
use derivative::*;

use crate::models::{DataObject, Group};

pub enum UpdateType {
    Added,
    Removed,
    Changed
}

#[derive(Derivative)]
#[derivative(Default, Debug)]
pub struct SaveData {
    pub groups: RefCell<Vec<Group>>,
    save_path: PathBuf,

    #[derivative(Debug="ignore")]
    group_update_callbacks: RefCell<Vec<Box<dyn Fn(&Group, &UpdateType)>>>
}

impl SaveData {
    pub fn new(pb: &PathBuf) -> Self {
        match SaveData::load_groups(pb) {
            Ok(groups) => {
                Self {
                    groups: RefCell::new(groups),
                    save_path: pb.to_owned(),
                    group_update_callbacks: RefCell::new(vec![])
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

    /// Add new group to groups list.
    pub fn new_group(&self, group: Group) -> Result<()> {
        self.groups.borrow_mut().push(group);
        self.groups.borrow().last().unwrap()
            .save_to_file(self.save_path.as_path().join(r"groups").as_path())?;

        self.run_group_update_callbacks(&self.groups.borrow().last().unwrap(), &UpdateType::Added);

        Ok(())
    }

    /// Save group file after changes
    pub fn save_group(&self, group: &Group) {
        group.save_to_file(self.save_path.as_path().join(r"groups").as_path())
            .expect("Could not save group into file");
    }

    /// Add a new callback for when any change occurs to groups (e.g. group added, removed, some
    /// value changed).
    ///
    /// Callbacks receive two arguments as input:
    /// - A reference to the modified group
    /// - An `UpdateType` enum representing the change type
    pub fn add_group_update_callback<F: 'static + Fn(&Group, &UpdateType)>(&self, callback: F) {
        self.group_update_callbacks.borrow_mut().push(Box::new(callback));
    }

    /// Run all callbacks registered for changes to groups.
    fn run_group_update_callbacks(&self, updated_group: &Group, update_type: &UpdateType) {
        for callback in self.group_update_callbacks.borrow().iter() {
            callback(updated_group, update_type);
        }
    }
}
