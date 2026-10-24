//! Notes: a Rapidkoi app that works fully offline. Every note is kept in
//! the app's own database on this device, and has its own link.
//!
//! The scaffold only: the app itself is built from its PRP.

use rapidkoi::{column, text, App, Event, Next, State, View};

/// The whole window.
#[derive(Debug, Default, State)]
pub struct Notes {}

impl App for Notes {
    // Plain text until the PRP gives Notes its own `Message` enum.
    type Msg = String;

    fn handle(self, _event: Event<String>) -> Next<Self> {
        Next::state(self)
    }

    fn view(&self) -> View<String> {
        column([text("Notes").title()]).screen_title("Notes")
    }
}

rapidkoi::export!(Notes);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_shows_its_name() {
        assert_eq!(Notes::default().view(), column([text("Notes").title()]).screen_title("Notes"));
    }
}
