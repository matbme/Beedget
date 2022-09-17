use serde::{Deserialize, Serialize};
use uuid::Uuid;
use anyhow::Result;

use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::gdk::RGBA;
use gtk::{gio, glib, ClosureExpression, SignalListItemFactory};
use glib::{ParamSpec, ParamSpecString, subclass::Signal};

use once_cell::sync::{Lazy, OnceCell};

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
        pub inner: RefCell<GroupInner>,
        pub transaction_list_store: OnceCell<gio::ListStore>,
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
                vec![
                    ParamSpecString::builder("uid").build(),
                    ParamSpecString::builder("emoji").build(),
                    ParamSpecString::builder("color").build(),
                    ParamSpecString::builder("name").build()
                ]
            });

            PROPERTIES.as_ref()
        }

        fn set_property(&self, _obj: &Self::Type, _id: usize, value: &glib::Value, pspec: &ParamSpec) {
            match pspec.name() {
                "uid" => self.inner.borrow_mut().id = Uuid::parse_str(value.get().unwrap()).unwrap(),
                "emoji" => self.inner.borrow_mut().emoji = value.get().unwrap(),
                "color" => {
                    let color = RGBA::parse(value.get().unwrap()).unwrap();
                    self.inner.borrow_mut().color = vec![
                        color.red(),
                        color.green(),
                        color.blue(),
                        color.alpha()
                    ];
                }
                "name" => self.inner.borrow_mut().name = value.get().unwrap(),
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

        fn signals() -> &'static [Signal] {
            static SIGNALS: Lazy<Vec<Signal>> = Lazy::new(|| {
                vec![
                    Signal::builder(
                        "transaction-added",
                        &[Transaction::static_type().into()],
                        <()>::static_type().into()
                    ).build(),
                    Signal::builder(
                        "transaction-removed",
                        &[Transaction::static_type().into()],
                        <()>::static_type().into()
                    ).build()
                ]
            });
            SIGNALS.as_ref()
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

    pub fn factory() -> SignalListItemFactory {
        let factory = SignalListItemFactory::new();

        factory.connect_setup(move |_, list_item| {
            let row = GroupRow::empty();
            list_item.set_child(Some(&row));
        });

        factory.connect_bind(move |_, list_item| {
            let group_row = list_item
                .child()
                .unwrap()
                .downcast::<GroupRow>()
                .unwrap();

            let group_item = list_item
                .item()
                .unwrap()
                .downcast::<Group>()
                .unwrap();

            group_row.bind(&group_item);
        });

        factory.connect_unbind(move |_, list_item| {
            let group_row = list_item
                .child()
                .unwrap()
                .downcast::<GroupRow>()
                .unwrap();

            group_row.unbind();
        });

        factory
    }

    pub fn search_expression() -> ClosureExpression {
        ClosureExpression::with_callback(gtk::Expression::NONE, |v| {
            let group = v[0].get::<Group>()
                .expect("Value is not a `Group`");

            group.name()
        })
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

    pub fn delete_file(&self, base_path: &Path) -> Result<()> {
        self.imp().inner.borrow().delete_file(base_path)?;

        Ok(())
    }

    pub fn id(&self) -> Uuid {
        self.imp().inner.borrow().id
    }

    pub fn name(&self) -> String {
        self.imp().inner.borrow().name.to_string()
    }

    pub fn emoji(&self) -> String {
        self.imp().inner.borrow().emoji.to_string()
    }

    pub fn color(&self) -> String {
        self.rgba_color().to_str().to_string()
    }

    pub fn set_name(&self, name: &str) {
        self.imp().inner.borrow_mut().name = name.to_string();
    }

    pub fn set_color(&self, color: &RGBA) {
        self.imp().inner.borrow_mut().color = vec![
            color.red(),
            color.green(),
            color.blue(),
            color.alpha()
        ];
    }

    pub fn set_emoji(&self, emoji: &str) {
        self.imp().inner.borrow_mut().emoji = emoji.to_string();
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
        self.emit_by_name::<()>(
            "transaction-added",
            &[&self.imp().inner.borrow().transactions.borrow().last().unwrap()]
        );
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

        let removed = self.imp().inner.borrow().transactions.borrow_mut().remove(idx);
        self.emit_by_name::<()>("transaction-removed", &[&removed]);
    }

    pub fn transaction_model(&self) -> &gio::ListStore {
        self.imp().transaction_list_store.get_or_init(|| {
            let ls = gio::ListStore::new(TransactionRow::static_type());

            for transaction in self.imp().inner.borrow().transactions.borrow().iter() {
                let row = TransactionRow::new(transaction);
                ls.append(&row);
            }

            self.connect_closure("transaction-added", false, glib::closure_local!(move |group: Group, transaction: &Transaction| {
                let row = TransactionRow::new(&transaction);

                let list_store = group.imp().transaction_list_store.get().unwrap();
                list_store.append(&row);
            }));

            self.connect_closure("transaction-removed", false, glib::closure_local!(move |group: Group, transaction: &Transaction| {
                let list_store = group.imp().transaction_list_store.get().unwrap();

                for i in 0..list_store.n_items() {
                    let transaction_id = list_store.item(i)
                        .expect(&format!("No item at position {}", i))
                        .downcast_ref::<TransactionRow>()
                        .expect("Item is not a TransactionRow")
                        .transaction()
                        .expect("No transaction set for TransactionRow")
                        .id();

                    if transaction_id == transaction.id() {
                        list_store.remove(i);
                        break;
                    }
                }
            }));

            ls
        })
    }
}
