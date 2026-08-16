use crate::PieMenuOverlayWidget;
use crate::overlay_widget::message::PieMenuMessage;
use glib::subclass::prelude::ObjectSubclassIsExt;
use std::sync::mpsc::Sender;

pub trait PieMenuMessageSender {
    /// Set the message sender for communicating with main application
    fn set_message_sender(&self, sender: Sender<PieMenuMessage>);

    /// Send a message to the main application
    fn send_message(&self, message: PieMenuMessage);
}

impl PieMenuMessageSender for PieMenuOverlayWidget {
    fn set_message_sender(&self, sender: Sender<PieMenuMessage>) {
        self.imp().set_message_sender(sender);
    }

    fn send_message(&self, message: PieMenuMessage) {
        self.imp().send_message(message);
    }
}
