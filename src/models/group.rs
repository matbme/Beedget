use serde::{Deserialize, Serialize};
use uuid::Uuid;
use anyhow::Result;

use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::gdk::RGBA;
use gtk::{gio, glib};
use glib::{ParamFlags, ParamSpec, ParamSpecString};

use once_cell::sync::Lazy;

use std::cell::RefCell;
use std::path::Path;

use crate::models::*;
use crate::widgets::*;

mod imp {
    use super::*;

    #[derive(Debug, Default, Serialize, Deserialize)]
    pub struct GroupInner {
        pub id: Uuid,
        pub name: String,
        pub emoji: String,
        pub color: Vec<f32>,
        pub transactions: RefCell<Vec<Transaction>>,
    }

    impl DataObject for GroupInner {
        fn filename(&self) -> String {
            self.id.to_string() + &String::from(".json")
        }
    }

    #[derive(Default)]
    pub struct Group {
        pub inner: RefCell<GroupInner>
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Group {
        const NAME: &'static str = "Group";
        type Type = super::Group;
        type ParentType = glib::Object;
    }

    impl ObjectImpl for Group {
        fn properties() -> &'static [ParamSpec] {
            static PROPERTIES: Lazy<Vec<ParamSpec>> = Lazy::new(|| {
                vec![ParamSpecString::new(
                    "uid",
                    "uid",
                    "uid",
                    None,
                    ParamFlags::READWRITE
                ),
                ParamSpecString::new(
                    "emoji",
                    "emoji",
                    "emoji",
                    None,
                    ParamFlags::READWRITE
                ),
                ParamSpecString::new(
                    "color",
                    "color",
                    "color",
                    None,
                    ParamFlags::READWRITE
                ),
                ParamSpecString::new(
                    "name",
                    "name",
                    "name",
                    None,
                    ParamFlags::READWRITE
                )]
            });

            PROPERTIES.as_ref()
        }

        fn set_property(&self, _obj: &Self::Type, _id: usize, value: &glib::Value, pspec: &ParamSpec) {
            match pspec.name() {
                "uid" => {
                    self.inner.borrow_mut().id = Uuid::parse_str(value.get().unwrap()).unwrap();
                }
                "emoji" => {
                    self.inner.borrow_mut().emoji = value.get().unwrap();
                }
                "color" => {
                    let color = RGBA::parse(value.get().unwrap()).unwrap();
                    self.inner.borrow_mut().color = vec![
                        color.red(),
                        color.green(),
                        color.blue(),
                        color.alpha()
                    ];
                }
                "name" => {
                    self.inner.borrow_mut().name = value.get().unwrap();
                }
                _ => unimplemented!()
            }
        }

        fn property(&self, obj: &Self::Type, _id: usize, pspec: &ParamSpec) -> glib::Value {
            match pspec.name() {
                "uid" => self.inner.borrow().id.to_string().to_value(),
                "emoji" => self.inner.borrow().emoji.to_value(),
                "color" => obj.rgba_color().to_str().to_value(),
                "name" => self.inner.borrow().name.to_value(),
                _ => unimplemented!()
            }
        }
    }
}

glib::wrapper! {
    pub struct Group(ObjectSubclass<imp::Group>);
}

impl Default for Group {
    fn default() -> Self {
        Self::empty()
    }
}

impl Group {
    pub fn new(emoji: &str, color: RGBA, name: &str) -> Self {
        glib::Object::new(&[
            ("uid", &Uuid::new_v4().to_string()),
            ("emoji", &emoji),
            ("color", &color.to_str()),
            ("name", &name),
        ]).expect("Failed to create Group")
    }

    pub fn empty() -> Self {
        glib::Object::new(&[]).expect("Failed to create Group")
    }

    pub fn load_from_file(path: &Path) -> Result<Self> {
        let group: Self = glib::Object::new(&[])
            .expect("Failed to create group");

        group.imp().inner.replace(imp::GroupInner::load_from_file(path)?);

        Ok(group)
    }

    pub fn save_to_file(&self, path: &Path) -> Result<()> {
        self.imp().inner.borrow().save_to_file(path)?;

        Ok(())
    }

    pub fn id(&self) -> Uuid {
        self.imp().inner.borrow().id
    }

    pub fn rgba_color(&self) -> RGBA {
        RGBA::new(
            self.imp().inner.borrow().color[0],
            self.imp().inner.borrow().color[1],
            self.imp().inner.borrow().color[2],
            self.imp().inner.borrow().color[3]
        )
    }

    pub fn new_transaction(&self, transaction: Transaction) {
        self.imp().inner.borrow().transactions.borrow_mut().push(transaction);
    }

    pub fn delete_transaction(&self, transaction_id: Uuid) {
        let mut idx = 0;

        for transaction in self.imp().inner.borrow().transactions.borrow().iter() {
            if transaction.id() == transaction_id {
                break;
            } else {
                idx += 1;
            }
        }

        self.imp().inner.borrow().transactions.borrow_mut().remove(idx);
    }

    pub fn transaction_model(&self) -> gio::ListStore {
        let ls = gio::ListStore::new(TransactionRow::static_type());

        for transaction in self.imp().inner.borrow().transactions.borrow().iter() {
            let row = TransactionRow::new(transaction);
            ls.append(&row);
        }

        ls
    }
}
