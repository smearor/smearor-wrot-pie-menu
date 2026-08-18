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

    /// Sets or removes the center widget.
    ///
    /// When `Some(widget)`, the widget is registered as a child of
    /// `PieMenuWidget` via `set_parent`, made visible via `show()`,
    /// and a resize is triggered via `queue_resize()`.
    ///
    /// When `None`, any existing center widget is unparented and
    /// the default center-click-to-close behavior is restored.
    ///
    /// The center widget rotates with the ring. The consumer is
    /// responsible for attaching event controllers (e.g.,
    /// `GestureClick`) to handle close-menu / close-submenu
    /// interactions.
    pub fn set_center_widget(&self, widget: Option<&Widget>) {
        let imp = self.imp();

        // Take old widget out of RefCell — borrow guard drops immediately
        let old_widget = imp.center_widget.borrow_mut().take();

        // Unparent OUTSIDE the borrow — safe even if GTK signals
        // trigger reentrant access to center_widget
        if let Some(existing) = old_widget {
            existing.unparent();
        }

        // Parent new widget BEFORE storing it in RefCell —
        // set_parent emits hierarchy-changed which could trigger
        // consumer callbacks; the RefCell must not be borrowed
        // when those fire
        if let Some(new_widget) = widget {
            new_widget.set_parent(self);
            new_widget.set_visible(true);
            *imp.center_widget.borrow_mut() = Some(new_widget.clone());
        }

        self.queue_resize();
        self.queue_draw();
    }

    /// Returns the current center widget, if any.
    pub fn center_widget(&self) -> Option<Widget> {
        self.imp().center_widget.borrow().clone()
    }
}

impl Default for PieMenuWidget {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gtk4::prelude::Cast;

    #[test]
    fn test_center_widget_default_none() {
        let imp = PieMenuWidgetImpl::default();
        assert!(imp.center_widget.borrow().is_none());
    }

    #[test]
    #[ignore = "requires GTK display environment"]
    fn test_set_center_widget_some() {
        crate::test_util::ensure_gtk_init();
        let widget = PieMenuWidget::new();
        let label = gtk4::Label::new(Some("test"));
        widget.set_center_widget(Some(label.upcast_ref()));
        assert!(widget.center_widget().is_some());
    }

    #[test]
    #[ignore = "requires GTK display environment"]
    fn test_set_center_widget_none_after_some() {
        crate::test_util::ensure_gtk_init();
        let widget = PieMenuWidget::new();
        let label = gtk4::Label::new(Some("test"));
        widget.set_center_widget(Some(label.upcast_ref()));
        assert!(widget.center_widget().is_some());
        widget.set_center_widget(None);
        assert!(widget.center_widget().is_none());
    }

    #[test]
    #[ignore = "requires GTK display environment"]
    fn test_set_center_widget_replaces_existing() {
        crate::test_util::ensure_gtk_init();
        let widget = PieMenuWidget::new();
        let label1 = gtk4::Label::new(Some("first"));
        widget.set_center_widget(Some(label1.upcast_ref()));
        let first = widget.center_widget();
        assert!(first.is_some());

        let label2 = gtk4::Label::new(Some("second"));
        widget.set_center_widget(Some(label2.upcast_ref()));
        let second = widget.center_widget();
        assert!(second.is_some());
        // The new widget should be different from the first
        let first_label = first.unwrap().downcast_ref::<gtk4::Label>().unwrap().label();
        let second_label = second.unwrap().downcast_ref::<gtk4::Label>().unwrap().label();
        assert_ne!(first_label, second_label);
    }
}
