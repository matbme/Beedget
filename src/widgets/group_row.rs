use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{glib, CompositeTemplate, SignalListItemFactory};
use glib::{ParamFlags, ParamSpec, ParamSpecObject};

use gtk::cairo::{LineJoin, LinearGradient};

use once_cell::sync::Lazy;

use std::cell::RefCell;
use std::f64::consts::PI;

use crate::force;
use crate::models::*;

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

        pub group: RefCell<Group>,
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
                    ParamSpecObject::builder("group", Group::static_type())
                        .flags(ParamFlags::CONSTRUCT | ParamFlags::READWRITE)
                        .build()
                ]
            });

            PROPERTIES.as_ref()
        }

        fn set_property(&self, obj: &Self::Type, _id: usize, value: &glib::Value, pspec: &ParamSpec) {
            match pspec.name() {
                "group" => {
                    if let Ok(input) = value.get::<Group>() {
                        self.group.replace(input);
                        obj.set_icon_emoji();
                        obj.set_draw_func();
                        obj.set_label();
                    }
                }
                _ => unimplemented!()
            }
        }

        fn property(&self, _obj: &Self::Type, _id: usize, pspec: &ParamSpec) -> glib::Value {
            match pspec.name() {
                "group" => self.group.borrow().to_value(),
                _ => unimplemented!()
            }
        }

        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);

            obj.connect_property_changes();
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
            ("group", &group)
        ]).expect("Failed to create `GroupRow`.")
    }

    pub fn empty() -> Self {
        glib::Object::new(&[])
            .expect("Failed to create `GroupRow`.")
    }

    pub fn factory() -> SignalListItemFactory {
        let group_factory = gtk::SignalListItemFactory::new();

        group_factory.connect_setup(move |_, list_item| {
            let row = GroupRow::empty();

            list_item.set_child(Some(&row));

            list_item
                .property_expression("item")
                .chain_property::<GroupRow>("group")
                .bind(&row, "group", gtk::Widget::NONE);
        });

        group_factory
    }

    pub fn search_expression() -> gtk::ClosureExpression {
        gtk::ClosureExpression::with_callback(gtk::Expression::NONE, |v| {
            let row = v[0].get::<GroupRow>()
                .expect("Value is not a `GroupRow`");

            let group = row.imp().group.borrow();
            group.property::<glib::GString>("name").to_string()
        })
    }

    fn connect_property_changes(&self) {
        self.connect_notify_local(Some("group"), move |instance, _| {
            instance.set_icon_emoji();
            instance.set_draw_func();
            instance.set_label();
        });
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

            let color = parent.imp().group.borrow().rgba_color();
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

    fn set_label(&self) {
        let label = self.imp().group
            .borrow()
            .property::<glib::GString>("name")
            .to_string();

        self.imp().name.set_label(&label);
    }

    fn set_icon_emoji(&self) {
        let icon = self.imp().group.borrow().property::<glib::GString>("emoji").to_string();

        self.imp().icon_emoji.set_label(&icon);
        self.imp().overlay.add_overlay(self.imp().icon_emoji.upcast_ref::<gtk::Label>());
    }
}
