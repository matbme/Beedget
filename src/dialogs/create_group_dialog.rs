use std::cell::RefCell;

use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{glib, CompositeTemplate};
use gtk::gdk::RGBA;

use adw::subclass::window::AdwWindowImpl;

use emojis;
use rand::prelude::*;

use crate::models::*;

mod imp {
    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/com/github/matbme/beedget/ui/create-group-dialog.ui")]
    pub struct CreateGroupDialog {
        #[template_child]
        pub add_button: TemplateChild<gtk::Button>,

        #[template_child]
        pub cancel_button: TemplateChild<gtk::Button>,

        #[template_child]
        pub group_name: TemplateChild<gtk::Entry>,

        #[template_child]
        pub group_color: TemplateChild<gtk::ColorButton>,

        #[template_child]
        pub group_icon_picker_button: TemplateChild<gtk::Button>,

        pub current_emoji : RefCell<String>
    }

    #[glib::object_subclass]
    impl ObjectSubclass for CreateGroupDialog {
        const NAME: &'static str = "CreateGroupDialog";
        type Type = super::CreateGroupDialog;
        type ParentType = adw::Window;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
            Self::Type::bind_template_callbacks(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for CreateGroupDialog {
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);

            let mut rng = rand::thread_rng();
            self.group_color.set_rgba(&RGBA::new(rng.gen(), rng.gen(), rng.gen(), 1.0));

            let random_emoji = emojis::iter().choose(&mut rng).unwrap();
            self.group_icon_picker_button.set_label(random_emoji.as_str());
        }
    }
    impl WidgetImpl for CreateGroupDialog {}
    impl WindowImpl for CreateGroupDialog {}
    impl AdwWindowImpl for CreateGroupDialog {}
}

glib::wrapper! {
    pub struct CreateGroupDialog(ObjectSubclass<imp::CreateGroupDialog>)
        @extends gtk::Window, gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::Native, gtk::Root, gtk::ShortcutManager;
}

#[gtk::template_callbacks]
impl CreateGroupDialog {
    pub fn new(parent: &gtk::Window) -> Self {
        glib::Object::new(&[
            ("transient-for", &Some(parent))
        ]).expect("Failed to create `CreateGroupDialog`.")
    }

    #[template_callback]
    fn close_window(&self) {
        self.destroy();
    }

    #[template_callback]
    fn create_group(&self) {
        let window = self.transient_for().unwrap();
        let application = window.application().unwrap();
        let beedget_application = application.downcast_ref::<crate::BeedgetApplication>().unwrap();
        let app_data = beedget_application.imp().data.get().unwrap();

        match app_data.new_group(Group::new(&self.imp().current_emoji.borrow(), self.imp().group_color.rgba(), &self.imp().group_name.text())) {
            Ok(()) => { self.destroy(); }
            Err(error) => { panic!("{}", error); }
        }
    }

    #[template_callback]
    fn present_group_icon_picker(&self) {
        let mut current_emoji = self.imp().current_emoji.borrow_mut();
        current_emoji.clear();
        current_emoji.push_str(self.imp().group_icon_picker_button.label().unwrap().as_str());

        let emoji_picker = gtk::EmojiChooser::new();
        self.imp().group_icon_picker_button.set_child(Some(&emoji_picker));

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
}
