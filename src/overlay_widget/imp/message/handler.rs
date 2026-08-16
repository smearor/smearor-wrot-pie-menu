use crate::PieMenuOverlayWidgetImpl;
use crate::overlay_widget::message::PieMenuMessage;
use crate::overlay_widget::message::handler::PieMenuMessageSender;
use std::sync::mpsc::Sender;

impl PieMenuMessageSender for PieMenuOverlayWidgetImpl {
    fn set_message_sender(&self, sender: Sender<PieMenuMessage>) {
        self.message_sender.replace(Some(sender));
    }

    fn send_message(&self, message: PieMenuMessage) {
        if let Some(sender) = self.message_sender.borrow().as_ref() {
            let _ = sender.send(message);
        }
    }
}
