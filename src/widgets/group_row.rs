use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{gdk, gio, glib, CompositeTemplate};
use gdk::RGBA;
use glib::{ParamSpec, ParamSpecString, ParamSpecObject};

use gtk::cairo::{LineJoin, LinearGradient};

use once_cell::sync::{Lazy, OnceCell};

use std::f64::consts::PI;

use crate::force;
use crate::dialogs::*;
use crate::models::*;

use beedget::app_data;

mod imp {
    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/com/github/matbme/beedget/ui/group-row.ui")]
    pub struct GroupRow {
        #[template_child]
        pub overlay: TemplateChild<gtk::Overlay>,

        #[template_child]
        pub icon: TemplateChild<gtk::DrawingArea>,

        #[template_child]
        pub icon_emoji: TemplateChild<gtk::Label>,

        #[template_child]
        pub name: TemplateChild<gtk::Label>,

        #[template_child]
        pub options_menu: TemplateChild<gtk::PopoverMenu>,

        pub group: OnceCell<Group>
    }

    #[glib::object_subclass]
    impl ObjectSubclass for GroupRow {
        const NAME: &'static str = "GroupRow";
        type Type = super::GroupRow;
        type ParentType = gtk::Box;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for GroupRow {
        fn properties() -> &'static [ParamSpec] {
            static PROPERTIES: Lazy<Vec<ParamSpec>> = Lazy::new(|| {
                vec![
                    ParamSpecString::builder("group-name").build(),
                    ParamSpecString::builder("group-color").build(),
                    ParamSpecString::builder("group-emoji").build(),
                    ParamSpecObject::builder("group", Group::static_type()).build()
                ]
            });

            PROPERTIES.as_ref()
        }

        fn set_property(&self, obj: &Self::Type, _id: usize, value: &glib::Value, pspec: &ParamSpec) {
            match pspec.name() {
                "group-name" => {
                    if let Ok(input) = value.get() {
                        obj.set_label(input);
                    }
                }
                "group-color" => {
                    if let Ok(input) = value.get() {
                        obj.set_draw_func(RGBA::parse(input).unwrap());
                    }
                }
                "group-emoji" => {
                    if let Ok(input) = value.get() {
                        obj.set_icon_emoji(input);
                    }
                }
                "group" => {
                    if let Ok(input) = value.get() {
                        self.group.set(input)
                            .expect("Failed to set group row's associated group");
                    }
                }
                _ => unimplemented!()
            }
        }

        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);

            let click_event_controller = gtk::GestureClick::builder()
                .button(3) // Right mouse button
                .build();

            click_event_controller.connect_pressed(glib::clone!(@weak obj as parent => move |_, _, _, _| {
                parent.imp().options_menu.popup();
            }));

            obj.add_controller(&click_event_controller);

            obj.setup_gactions();
        }
    }

    impl WidgetImpl for GroupRow {}
    impl BoxImpl for GroupRow {}
}

glib::wrapper! {
    pub struct GroupRow(ObjectSubclass<imp::GroupRow>)
        @extends gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::Orientable;
}

impl GroupRow {
    pub fn new(group: &Group) -> Self {
        glib::Object::new(&[
            ("group-name", &group.name()),
            ("group-color", &group.color()),
            ("group-emoji", &group.emoji())
        ]).expect("Failed to create `GroupRow`.")
    }

    pub fn empty() -> Self {
        glib::Object::new(&[])
            .expect("Failed to create `GroupRow`.")
    }

    fn setup_gactions(&self) {
        let group_action_group = gio::SimpleActionGroup::new();

        let edit_action = gio::SimpleAction::new("edit", None);
        edit_action.connect_activate(glib::clone!(@weak self as parent => move |_, _| {
            let dialog = GroupDialog::edit(
                parent.root().unwrap().downcast_ref::<gtk::Window>().unwrap(),
                parent.imp().group.get().unwrap()
            );
            dialog.present();
        }));
        group_action_group.add_action(&edit_action);

        let delete_action = gio::SimpleAction::new("delete", None);
        delete_action.connect_activate(glib::clone!(@weak self as parent => move |_, _| {
            parent.delete_group();
        }));
        group_action_group.add_action(&delete_action);

        self.insert_action_group("group", Some(&group_action_group));
    }

    fn set_draw_func(&self, color: RGBA) {
        self.imp().icon.set_draw_func(glib::clone!(@weak self as parent => move |_, ctx, w, h| {
            let allocation = parent.imp().icon.parent().unwrap().allocation();

            // Cairo expects values as f64
            let allocation_x = allocation.x() as f64;
            let width = w as f64;
            let height = h as f64;

            ctx.set_tolerance(0.1);
            ctx.set_line_join(LineJoin::Bevel);

            ctx.set_source_rgb(color.red() as f64, color.green() as f64, color.blue() as f64);

            let gradient = LinearGradient::new(allocation_x,
                height / 2.0 - 15.0,
                allocation_x + 30.0,
                (height / 2.0 - 15.0) + 30.0);
            // Gradient starts darker
            gradient.add_color_stop_rgb(
                0.0,
                (color.red() - (color.red() * 0.3)) as f64,
                (color.green() - (color.green() * 0.3)) as f64,
                (color.blue() - (color.blue() * 0.3)) as f64
            );
            // Goes to actual color
            gradient.add_color_stop_rgb(
                0.33,
                color.red() as f64,
                color.green() as f64,
                color.blue() as f64
            );
            // And ends brighter
            gradient.add_color_stop_rgb(
                0.66,
                (color.red() + (color.red() * 0.2)) as f64,
                (color.green() + (color.green() * 0.2)) as f64,
                (color.blue() + (color.blue() * 0.2)) as f64
            );
            force!(ctx.set_source(&gradient));

            force!(ctx.save());

            ctx.arc(width / 2.0,     // x
                    height / 2.0,    // y
                    height / 2.0,    // radius
                    0.0,
                    2.0 * PI);
            force!(ctx.fill());

            force!(ctx.restore());
        }));

        self.imp().icon.queue_draw();
    }

    fn set_label(&self, label: &str) {
        self.imp().name.set_label(&label);
    }

    fn set_icon_emoji(&self, icon: &str) {
        self.imp().icon_emoji.set_label(&icon);
        self.imp().overlay.add_overlay(self.imp().icon_emoji.upcast_ref::<gtk::Label>());
    }

    fn delete_group(&self) {
        app_data!(|data| data.delete_group(self.imp().group.get().unwrap()));
    }
}
