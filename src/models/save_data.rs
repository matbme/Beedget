use std::cell::RefCell;
use std::fs;
use std::io::ErrorKind;
use std::path::PathBuf;

use anyhow::{Error, Result};
use once_cell::sync::OnceCell;

use gtk::gio;
use gtk::prelude::*;

use crate::models::Group;

#[derive(Default, Debug)]
pub struct SaveData {
    pub groups: RefCell<Vec<Group>>,
    save_path: PathBuf,

    pub group_model: OnceCell<gio::ListStore>,
}

impl SaveData {
    pub fn new(pb: &PathBuf) -> Self {
        match SaveData::load_groups(pb) {
            Ok(groups) => Self {
                groups: RefCell::new(groups),
                save_path: pb.to_owned(),
                group_model: OnceCell::new(),
            },
            Err(e) => {
                panic!("Error loading save data: {}", e);
            }
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
                _ => Err(Error::new(error)),
            },
        }
    }

    /// Get groups model based on data loaded from storage
    pub fn group_model(&self) -> &gio::ListStore {
        self.group_model.get_or_init(|| {
            let ls = gio::ListStore::new(Group::static_type());

            for group in self.groups.borrow().iter() {
                ls.append(group);
            }

            ls
        })
    }

    /// Add new group to groups list.
    pub fn new_group(&self, group: Group) -> Result<()> {
        self.groups.borrow_mut().push(group);

        let stored_groups = self.groups.borrow();
        stored_groups
            .last()
            .unwrap()
            .save_to_file(self.save_path.as_path().join(r"groups").as_path())?;

        self.group_model().append(stored_groups.last().unwrap());

        Ok(())
    }

    /// Save group file after changes
    pub fn save_group(&self, group: &Group) {
        group
            .save_to_file(self.save_path.as_path().join(r"groups").as_path())
            .expect("Could not save group into file");
    }

    /// Delete group and all transactions
    pub fn delete_group(&self, group: &Group) {
        group
            .delete_file(self.save_path.as_path().join(r"groups").as_path())
            .expect("Could not delete group file");

        let list_store = self.group_model();
        for i in 0..list_store.n_items() {
            let group_id = list_store
                .item(i)
                .expect(&format!("No item at position {}", i))
                .downcast_ref::<Group>()
                .expect("Item is not a Group")
                .id();

            if group_id == group.id() {
                list_store.remove(i);
                break;
            }
        }
    }
}
