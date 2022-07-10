use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{glib, CompositeTemplate, SignalListItemFactory, PropertyExpression};
use gtk::gdk::RGBA;
use gtk::cairo::{LineJoin, LinearGradient};
use glib::{ParamFlags, ParamSpec, ParamSpecString};

use once_cell::sync::Lazy;

use std::f64::consts::PI;
use std::cell::RefCell;

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

        pub emoji: RefCell<Option<String>>,
        pub color: RefCell<Option<RGBA>>,
        pub label: RefCell<Option<String>>
    }

    #[glib::object_subclass]
    impl ObjectSubclass for GroupListRowContent {
        const NAME: &'static str = "GroupListRowContent";
        type Type = super::GroupListRowContent;
        type ParentType = gtk::Box;

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
                    ParamFlags::CONSTRUCT | ParamFlags::READWRITE
                ),
                ParamSpecString::new(
                    "color",
                    "color",
                    "color",
                    None,
                    ParamFlags::CONSTRUCT | ParamFlags::READWRITE
                ),
                ParamSpecString::new(
                    "label",
                    "label",
                    "label",
                    None,
                    ParamFlags::CONSTRUCT | ParamFlags::READWRITE
                )]
            });

            PROPERTIES.as_ref()
        }

        fn set_property(&self, _obj: &Self::Type, _id: usize, value: &glib::Value, pspec: &ParamSpec) {
            match pspec.name() {
                "emoji" => {
                    if let Ok(input) = value.get() {
                        self.emoji.replace(input);
                    }
                }
                "color" => {
                    if let Ok(input) = value.get() {
                        self.color.replace(Some(RGBA::parse(input).unwrap()));
                    }
                }
                "label" => {
                    if let Ok(input) = value.get() {
                        self.label.replace(input);
                    }
                }
                _ => unimplemented!()
            }
        }

        fn property(&self, _obj: &Self::Type, _id: usize, pspec: &ParamSpec) -> glib::Value {
            match pspec.name() {
                "emoji" => self.emoji.borrow().to_value(),
                "color" => self.color.borrow().unwrap().to_str().to_value(),
                "label" => self.label.borrow().to_value(),
                _ => unimplemented!()
            }
        }

        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);

            if self.emoji.borrow().is_some() {
                obj.set_icon_emoji();
            }

            if self.color.borrow().is_some() {
                obj.set_draw_func();
            }

            if self.label.borrow().is_some() {
                obj.set_label();
            }

            obj.connect_property_changes();
        }
    }

    impl WidgetImpl for GroupListRowContent {}
    impl BoxImpl for GroupListRowContent {}
}

glib::wrapper! {
    pub struct GroupListRowContent(ObjectSubclass<imp::GroupListRowContent>)
        @extends gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::Orientable;
}

impl GroupListRowContent {
    pub fn new(emoji: &str, color: &RGBA, label: &str) -> Self {
        glib::Object::new(&[
            ("emoji", &String::from(emoji)),
            ("color", &color.to_str()),
            ("label", &String::from(label)),
        ]).expect("Failed to create `GroupListRowContent`.")
    }

    pub fn empty() -> Self {
        glib::Object::new(&[])
            .expect("Failed to create `GroupListRowContent`.")
    }

    pub fn factory() -> SignalListItemFactory {
        let group_factory = gtk::SignalListItemFactory::new();

        group_factory.connect_setup(move |_, list_item| {
            let row = GroupListRowContent::empty();

            list_item.set_child(Some(&row));

            list_item
                .property_expression("item")
                .chain_property::<GroupListRowContent>("emoji")
                .bind(&row, "emoji", gtk::Widget::NONE);

            list_item
                .property_expression("item")
                .chain_property::<GroupListRowContent>("color")
                .bind(&row, "color", gtk::Widget::NONE);

            list_item
                .property_expression("item")
                .chain_property::<GroupListRowContent>("label")
                .bind(&row, "label", gtk::Widget::NONE);
        });

        group_factory
    }

    pub fn search_expression() -> PropertyExpression {
        PropertyExpression::new(
            GroupListRowContent::static_type(),
            gtk::Expression::NONE,
            "label"
        )
    }

    fn connect_property_changes(&self) {
        self.connect_notify_local(Some("emoji"), move |instance, _| {
            instance.set_icon_emoji();
        });
        self.connect_notify_local(Some("color"), move |instance, _| {
            instance.set_draw_func();
        });
        self.connect_notify_local(Some("label"), move |instance, _| {
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

            let color = parent.imp().color.borrow().unwrap();
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
        if let Some(label) = self.imp().label.borrow().as_ref() {
            self.imp().name.set_label(&label);
        }
    }

    fn set_icon_emoji(&self) {
        if let Some(icon) = self.imp().emoji.borrow().as_ref() {
            self.imp().icon_emoji.set_label(&icon);
            self.imp().overlay.add_overlay(self.imp().icon_emoji.upcast_ref::<gtk::Label>());
        }
    }
}
