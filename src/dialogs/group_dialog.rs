use std::cell::RefCell;

use glib::{ParamFlags, ParamSpec, ParamSpecObject};
use gtk::gdk::RGBA;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{gdk, glib, CompositeTemplate};

use once_cell::sync::{Lazy, OnceCell};

use adw::subclass::window::AdwWindowImpl;

use emojis;
use rand::prelude::*;

use crate::application;
use crate::models::*;

mod imp {
    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/com/github/matbme/beedget/ui/group-dialog.ui")]
    pub struct GroupDialog {
        #[template_child]
        pub add_button: TemplateChild<gtk::Button>,

        #[template_child]
        pub cancel_button: TemplateChild<gtk::Button>,

        #[template_child]
        pub group_name: TemplateChild<gtk::Entry>,

        #[template_child]
        pub group_budget: TemplateChild<gtk::Entry>,

        #[template_child]
        pub decrease_budget_button: TemplateChild<gtk::Button>,

        #[template_child]
        pub increase_budget_button: TemplateChild<gtk::Button>,

        #[template_child]
        pub group_color: TemplateChild<gtk::ColorButton>,

        #[template_child]
        pub no_budget_check: TemplateChild<gtk::CheckButton>,

        #[template_child]
        pub group_icon_picker_button: TemplateChild<gtk::Button>,

        pub current_emoji: RefCell<String>,
        pub budget: RefCell<Budget>,
        pub edit_group: OnceCell<Group>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for GroupDialog {
        const NAME: &'static str = "GroupDialog";
        type Type = super::GroupDialog;
        type ParentType = adw::Window;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
            Self::Type::bind_template_callbacks(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for GroupDialog {
        fn properties() -> &'static [ParamSpec] {
            static PROPERTIES: Lazy<Vec<ParamSpec>> = Lazy::new(|| {
                vec![ParamSpecObject::builder("group", Group::static_type())
                    .flags(ParamFlags::CONSTRUCT | ParamFlags::READWRITE)
                    .build()]
            });

            PROPERTIES.as_ref()
        }

        fn set_property(
            &self,
            obj: &Self::Type,
            _id: usize,
            value: &glib::Value,
            pspec: &ParamSpec,
        ) {
            match pspec.name() {
                "group" => {
                    if let Ok(input) = value.get::<Group>() {
                        match self.edit_group.set(input) {
                            Ok(_) => obj.populate_group_values(),
                            Err(_) => panic!("Group pointer was already set!"),
                        }
                    }
                }
                _ => unimplemented!(),
            }
        }

        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);

            if obj.imp().edit_group.get().is_none() {
                // Random color
                let mut rng = rand::thread_rng();
                self.group_color
                    .set_rgba(&RGBA::new(rng.gen(), rng.gen(), rng.gen(), 1.0));

                // Random emoji
                let random_emoji = emojis::iter().choose(&mut rng).unwrap();
                self.group_icon_picker_button
                    .set_label(random_emoji.as_str());
                *obj.imp().current_emoji.borrow_mut() = random_emoji.to_string();
            }

            obj.connect_key_event_controller();
            obj.connect_budget_entry_changes();
            obj.connect_constraints_for_confirm_button();
            obj.listen_no_budget_check_button();
        }
    }

    impl WidgetImpl for GroupDialog {}
    impl WindowImpl for GroupDialog {}
    impl AdwWindowImpl for GroupDialog {}
}

glib::wrapper! {
    pub struct GroupDialog(ObjectSubclass<imp::GroupDialog>)
        @extends gtk::Window, gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget,
                    gtk::Native, gtk::Root, gtk::ShortcutManager;
}

#[gtk::template_callbacks]
impl GroupDialog {
    pub fn new(parent: &gtk::Window) -> Self {
        glib::Object::new(&[("transient-for", &Some(parent))])
            .expect("Failed to create `GroupDialog`.")
    }

    pub fn edit(parent: &gtk::Window, edit_group: &Group) -> Self {
        glib::Object::new(&[("transient-for", &Some(parent)), ("group", &edit_group)])
            .expect("Failed to create `GroupDialog`")
    }

    #[template_callback]
    fn close_window(&self) {
        self.destroy();
    }

    #[template_callback]
    fn confirm_group(&self) {
        if self.imp().edit_group.get().is_some() {
            self.modify_group();
        } else {
            self.create_group();
        }
    }

    #[template_callback]
    fn increase_budget(&self) {
        self.imp().budget.replace_with(|budget| Budget::Some(*budget.get_value().unwrap() + 1.0));

        if !self.imp().decrease_budget_button.is_sensitive() {
            self.imp().decrease_budget_button.set_sensitive(true);
        }

        self.imp()
            .group_budget
            .set_buffer(&gtk::EntryBuffer::new(Some(&format!(
                "{:.2}",
                &self.imp().budget.borrow().get_value().unwrap_or(&0.0)
            ))))
    }

    #[template_callback]
    fn decrease_budget(&self) {
        self.imp().budget.replace_with(|budget| Budget::Some(*budget.get_value().unwrap() - 1.0));

        if self.imp().budget.borrow().get_value().unwrap().le(&0.0) {
            self.imp().budget.replace_with(|_| Budget::Some(0.1));
            self.imp().decrease_budget_button.set_sensitive(false);
        }

        self.imp()
            .group_budget
            .set_buffer(&gtk::EntryBuffer::new(Some(&format!(
                "{:.2}",
                &self.imp().budget.borrow().get_value().unwrap_or(&0.0)
            ))));
    }

    fn create_group(&self) {
        let group = Group::new(
            &self.imp().current_emoji.borrow(),
            self.imp().group_color.rgba(),
            &self.imp().group_name.text(),
            self.imp().budget.borrow().clone(),
        );

        let application = application!(self @as crate::BeedgetApplication);
        match application.data().new_group(group) {
            Ok(()) => {
                self.destroy();
            }
            Err(error) => {
                panic!("{}", error);
            }
        }
    }

    fn modify_group(&self) {
        let group = self.imp().edit_group.get().unwrap();

        let name = self.imp().group_name.text();
        group.set_property("name", name.to_value());

        let color_str = self.imp().group_color.rgba().to_str();
        group.set_property("color", color_str.to_value());

        let emoji = self
            .imp()
            .group_icon_picker_button
            .label()
            .expect("No group emoji selected");
        group.set_property("emoji", emoji.to_value());

        let budget = self.imp().budget.borrow();
        group.set_budget(&budget);

        application!(self @as crate::BeedgetApplication)
            .data()
            .save_group(group);

        self.destroy();
    }

    #[template_callback]
    fn present_group_icon_picker(&self) {
        let mut current_emoji = self.imp().current_emoji.borrow_mut();
        current_emoji.clear();
        current_emoji.push_str(
            self.imp()
                .group_icon_picker_button
                .label()
                .unwrap()
                .as_str(),
        );

        let emoji_picker = gtk::EmojiChooser::new();
        self.imp()
            .group_icon_picker_button
            .set_child(Some(&emoji_picker));

        emoji_picker.connect_emoji_picked(glib::clone!(@weak self as parent => move |_, text| {
            if !text.is_empty() {
                parent.imp().group_icon_picker_button.set_label(text);
                parent.imp().current_emoji.borrow_mut().replace_range(.., text);
            }
        }));

        emoji_picker.connect_closed(glib::clone!(@weak self as parent => move |_| {
            parent.imp().group_icon_picker_button.set_label(parent.imp().current_emoji.borrow().as_str());
        }));

        emoji_picker.popup();
    }

    /// Disables button if name and/or amount entries are empty
    fn connect_constraints_for_confirm_button(&self) {
        // Set initial
        self.imp()
            .add_button
            .set_sensitive(self.imp().group_name.text_length() > 0);

        // Subscribe to changes
        self.imp().group_name.buffer().connect_length_notify(
            glib::clone!(@weak self as parent => move |_| {
                parent.imp().add_button.set_sensitive(parent.imp().group_name.text_length() > 0);
            }),
        );
    }

    fn connect_budget_entry_changes(&self) {
        self.imp().group_budget.connect_text_notify(
            glib::clone!(@weak self as parent => move |_| {
                match parent.imp().group_budget.text().to_string().parse::<f32>() {
                    Ok(val) => {
                        parent.imp().budget.replace(Budget::Some(val));
                    },
                    Err(_) => {
                        let current_val = parent.imp().budget.borrow().get_value().unwrap_or(&0.1).clone();
                        parent.imp().group_budget.set_text(&current_val.to_string());
                    },
                };
            }),
        );
    }

    /// Disable budget widgets if "no budget" is selected
    fn listen_no_budget_check_button(&self) {
        self.imp()
            .no_budget_check
            .connect_toggled(glib::clone!(@weak self as parent => move |_| {
                let checked = parent.imp().no_budget_check.is_active();

                parent.imp().group_budget.set_sensitive(!checked);
                parent.imp().increase_budget_button.set_sensitive(!checked);
                parent.imp().decrease_budget_button.set_sensitive(!checked);
            }));
    }

    /// Handle keyboard events
    fn connect_key_event_controller(&self) {
        let key_controller = gtk::EventControllerKey::new();
        key_controller.connect_key_pressed(
            glib::clone!(@strong self as parent => move |_, keyval, _, _| {
                match keyval {
                    gdk::Key::Escape => { // Esc closes dialog
                        parent.destroy();
                        gtk::Inhibit(true)
                    }
                    _ => { gtk::Inhibit(false) }
                }
            }),
        );

        self.add_controller(&key_controller);
    }

    /// Fill entries with group values for edit
    fn populate_group_values(&self) {
        let group = self
            .imp()
            .edit_group
            .get()
            .expect("Group is not initialized");

        self.imp()
            .group_name
            .set_buffer(&gtk::EntryBuffer::new(Some(&group.name())));

        self.imp().group_color.set_rgba(&group.rgba_color());
        self.imp()
            .group_icon_picker_button
            .set_label(&group.emoji());
    }
}
