use crate::menu::widget_factory_erased::MenuItemWidgetFactoryErased;
use crate::menu_widget::imp::widget::PieMenuWidgetImpl;
use gtk4::Accessible;
use gtk4::Buildable;
use gtk4::ConstraintTarget;
use gtk4::Widget;
use gtk4::glib;
use gtk4::prelude::WidgetExt;
use gtk4::subclass::prelude::*;
use std::sync::atomic::Ordering;

glib::wrapper! {
    pub struct PieMenuWidget(ObjectSubclass<PieMenuWidgetImpl>)
        @extends Widget,
        @implements Accessible, Buildable, ConstraintTarget;
}

impl PieMenuWidget {
    pub fn new() -> Self {
        let widget: Self = glib::Object::builder().build();

        // Register CSS for keyboard-selected highlight on standard widgets
        let css = "button.selected { outline: 2px solid white; outline-offset: 2px; }";
        let provider = gtk4::CssProvider::new();
        provider.load_from_string(css);
        if let Some(display) = gtk4::gdk::Display::default() {
            gtk4::style_context_add_provider_for_display(&display, &provider, gtk4::STYLE_PROVIDER_PRIORITY_USER);
        }

        widget
    }

    pub fn set_close_callback<F: Fn() + 'static>(&self, callback: F) {
        self.imp().close_callback.replace(Some(Box::new(callback)));
    }

    /// Enables or disables drawing of inner and outer ring markings.
    /// Default: `true`.
    pub fn set_markings_enabled(&self, enabled: bool) {
        self.imp().markings_enabled.store(enabled, Ordering::Relaxed);
        self.queue_draw();
    }

    /// Returns whether ring markings are currently enabled.
    pub fn markings_enabled(&self) -> bool {
        self.imp().markings_enabled.load(Ordering::Relaxed)
    }

    /// Registers a custom widget factory under its type name.
    ///
    /// If a factory with the same type name already exists, it is
    /// replaced. Call `refresh_widgets` after registering new
    /// factories to rebuild existing item widgets.
    pub fn register_widget_factory(&self, factory: Box<dyn MenuItemWidgetFactoryErased>) {
        self.imp().widget_registry.borrow_mut().register(factory);
    }
}

impl Default for PieMenuWidget {
    fn default() -> Self {
        Self::new()
    }
}
