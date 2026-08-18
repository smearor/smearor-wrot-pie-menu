#[cfg(test)]
use std::sync::Once;

#[cfg(test)]
static GTK_INIT: Once = Once::new();

/// Initializes GTK4 for tests. Safe to call multiple times — only
/// the first call actually runs `gtk4::init()`.
#[cfg(test)]
pub fn ensure_gtk_init() {
    GTK_INIT.call_once(|| {
        gtk4::init().expect("Failed to initialize GTK4 for tests");
    });
}
