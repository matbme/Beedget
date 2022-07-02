use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{glib, CompositeTemplate};
use gtk::gdk::RGBA;
use gtk::cairo::{LineJoin, LinearGradient};
use glib::{ParamFlags, ParamSpec, ParamSpecString};

use once_cell::sync::{Lazy, OnceCell};

use std::f64::consts::PI;

use crate::force;

mod imp {
    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/com/github/matbme/beedget/ui/group-list-row-content.ui")]
    pub struct GroupListRowContent {
        #[template_child]
        pub overlay: TemplateChild<gtk::Overlay>,

        #[template_child]
        pub icon: TemplateChild<gtk::DrawingArea>,

        #[template_child]
        pub icon_emoji: TemplateChild<gtk::Label>,

        #[template_child]
        pub name: TemplateChild<gtk::Label>,

        pub emoji: OnceCell<String>,
        pub color: OnceCell<RGBA>,
        pub label: OnceCell<String>
    }

    #[glib::object_subclass]
    impl ObjectSubclass for GroupListRowContent {
        const NAME: &'static str = "GroupListRowContent";
        type Type = super::GroupListRowContent;
        type ParentType = gtk::ListBoxRow;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for GroupListRowContent {
        fn properties() -> &'static [ParamSpec] {
            static PROPERTIES: Lazy<Vec<ParamSpec>> = Lazy::new(|| {
                vec![ParamSpecString::new(
                    "emoji",
                    "emoji",
                    "emoji",
                    None,
                    ParamFlags::CONSTRUCT_ONLY | ParamFlags::READWRITE
                ),
                ParamSpecString::new(
                    "color",
                    "color",
                    "color",
                    None,
                    ParamFlags::CONSTRUCT_ONLY | ParamFlags::READWRITE
                ),
                ParamSpecString::new(
                    "label",
                    "label",
                    "label",
                    None,
                    ParamFlags::CONSTRUCT_ONLY | ParamFlags::READWRITE
                )]
            });

            PROPERTIES.as_ref()
        }

        fn set_property(&self, _obj: &Self::Type, _id: usize, value: &glib::Value, pspec: &ParamSpec) {
            match pspec.name() {
                "emoji" => {
                    let input = value.get().expect("Invalid emoji for group");
                    force!(self.emoji.set(input));
                }
                "color" => {
                    let input = value.get().expect("Invalid color for group");
                    force!(self.color.set(RGBA::parse(input).unwrap()));
                }
                "label" => {
                    let input = value.get().expect("Invalid label for group");
                    force!(self.label.set(input));
                }
                _ => unimplemented!()
            }
        }

        fn property(&self, _obj: &Self::Type, _id: usize, pspec: &ParamSpec) -> glib::Value {
            match pspec.name() {
                "emoji" => self.emoji.get().to_value(),
                "color" => self.color.get().to_value(),
                "label" => self.label.get().to_value(),
                _ => unimplemented!()
            }
        }

        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);

            obj.set_draw_func();
            obj.imp().icon.queue_draw();

            obj.set_label();
            obj.set_icon_emoji();
        }
    }

    impl WidgetImpl for GroupListRowContent {}
    impl ListBoxRowImpl for GroupListRowContent {}
}

glib::wrapper! {
    pub struct GroupListRowContent(ObjectSubclass<imp::GroupListRowContent>)
        @extends gtk::Widget,
        @implements gtk::Accessible, gtk::Actionable, gtk::Buildable, gtk::ConstraintTarget;
}

impl GroupListRowContent {
    pub fn new(emoji: &str, color: &RGBA, label: &str) -> Self {
        glib::Object::new(&[
            ("emoji", &String::from(emoji)),
            ("color", &color.to_str()),
            ("label", &String::from(label)),
        ]).expect("Failed to create `GroupListRowContent`.")
    }

    fn set_draw_func(&self) {
        self.imp().icon.set_draw_func(glib::clone!(@weak self as parent => move |_, ctx, w, h| {
            let allocation = parent.imp().icon.parent().unwrap().allocation();

            // Cairo expects values as f64
            let allocation_x = allocation.x() as f64;
            let width = w as f64;
            let height = h as f64;

            ctx.set_tolerance(0.1);
            ctx.set_line_join(LineJoin::Bevel);

            let color = parent.imp().color.get().unwrap();
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

            ctx.arc(width / 2.0,    // x
                    height / 2.0,    // y
                    height / 2.0,    // radius
                    0.0,
                    2.0 * PI);
            force!(ctx.fill());

            force!(ctx.restore());
        }));
    }

    fn set_label(&self) {
        let label = self.imp().label.get().unwrap();
        self.imp().name.set_label(&label);
    }

    fn set_icon_emoji(&self) {
        let icon = self.imp().emoji.get().unwrap();
        self.imp().icon_emoji.set_label(&icon);

        self.imp().overlay.add_overlay(self.imp().icon_emoji.upcast_ref::<gtk::Label>());
    }
}
