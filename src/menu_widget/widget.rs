use crate::menu_widget::imp::widget::PieMenuWidgetImpl;
use gtk4::Accessible;
use gtk4::Buildable;
use gtk4::ConstraintTarget;
use gtk4::Widget;
use gtk4::glib;
use gtk4::subclass::prelude::*;

glib::wrapper! {
    pub struct PieMenuWidget(ObjectSubclass<PieMenuWidgetImpl>)
        @extends Widget,
        @implements Accessible, Buildable, ConstraintTarget;
}

impl PieMenuWidget {
    pub fn new() -> Self {
        glib::Object::builder().build()
    }

    pub fn set_close_callback<F: Fn() + 'static>(&self, callback: F) {
        self.imp().close_callback.replace(Some(Box::new(callback)));
    }
}
