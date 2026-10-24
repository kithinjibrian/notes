//! Notes: a Rapidkoi app that works fully offline (PRP-001). Notes are kept
//! in the app's own database on this device, newest first; each has its own
//! link (`?note=7`); search reads titles and text. Nothing is ever sent
//! anywhere.
//!
//! The window keeps a copy of what it read: the list, and the open note.
//! The database stays the truth: the list is read again after every write
//! and whenever another window changes the notes.

extern crate alloc;

mod store;
mod view;

use rapidkoi::{App, Event, Message, Next, State, View};
use store::{cut, Note, Row, MAX_BODY, MAX_TITLE};

/// The whole window.
#[derive(Debug, Default, State)]
pub struct Notes {
    /// The open note's ID, in the window's address; 0 when none is open.
    note: u32,
    /// What's typed in search.
    query: String,
    /// The list as last read.
    list: Vec<Row>,
    /// The open note, as last read and edited here.
    open: Note,
    /// Deleting the open note is being asked about.
    confirming: bool,
    /// The list has been read at least once.
    read: bool,
    /// Why the last call to the database didn't work, if it didn't.
    problem: Option<String>,
}

/// What Notes' screen sends.
#[derive(Debug, Clone, PartialEq, Message)]
pub enum Msg {
    /// Search's text changed: it comes as the event's `value`.
    Search,
    /// A note in the list was clicked.
    Pick(u32),
    /// +, or New note on the empty page.
    New,
    /// The title changed: its text is the event's `value`.
    Title,
    /// The text changed: it is the event's `value`.
    Body,
    AskDelete,
    Keep,
    Delete,
}

impl Notes {
    /// The note `id` open, in the link, and read whole.
    fn open_note(self, id: u32) -> Next<Self> {
        Next::state(Notes { note: id, confirming: false, ..self }).link(Notes::note).ask(store::note(id))
    }

    /// No note open, and none in the address. `note` stays a number (0),
    /// not an `Option`: a link can only set a field whose kind the fresh
    /// state shows (contract 4), and `None` shows none.
    fn close_note(self) -> Next<Self> {
        Next::state(Notes { note: 0, open: Note::default(), confirming: false, ..self }).no_link()
    }

    fn on_message(mut self, msg: Msg, value: Option<String>) -> Next<Self> {
        let id = self.open.id;
        match msg {
            Msg::Search => {
                self.query = value.unwrap_or_default();
                let read = store::list("list", &self.query);
                Next::state(self).ask(read)
            }
            Msg::Pick(id) => self.open_note(id),
            Msg::New => {
                self.query = String::new();
                Next::state(self).ask(store::create())
            }
            // The window's copy changes at once; the database follows, and
            // only takes this edit if no later one got there first.
            Msg::Title if id != 0 => {
                self.open.title = cut(&value.unwrap_or_default(), MAX_TITLE);
                self.open.rev += 1;
                let save = store::save_title(id, &self.open.title, self.open.rev);
                Next::state(self).ask(save)
            }
            Msg::Body if id != 0 => {
                self.open.body = cut(&value.unwrap_or_default(), MAX_BODY);
                self.open.rev += 1;
                let save = store::save_body(id, &self.open.body, self.open.rev);
                Next::state(self).ask(save)
            }
            Msg::AskDelete if id != 0 => Next::state(Notes { confirming: true, ..self }),
            Msg::Keep => Next::state(Notes { confirming: false, ..self }),
            Msg::Delete if id != 0 => self.close_note().ask(store::delete(id)),
            _ => Next::state(self),
        }
    }

    /// A note read by its ID: opened, unless this window has edits the
    /// database hasn't caught up with; gone, the link closes.
    fn arrived(self, note: Option<Note>) -> Next<Self> {
        match note {
            Some(n) if n.id == self.note => {
                let open = if self.open.id == n.id && self.open.rev > n.rev { self.open.clone() } else { n };
                Next::state(Notes { open, problem: None, ..self })
            }
            // An answer for a note no longer asked for.
            Some(_) => Next::state(self),
            None => self.close_note(),
        }
    }
}

impl App for Notes {
    type Msg = Msg;

    fn handle(self, event: Event<Msg>) -> Next<Self> {
        match event {
            // A fresh window has no note open; a link's `note` is set after this.
            Event::Init => Next::state(self).no_link(),
            // Every open: the tables made if they aren't yet.
            Event::Opened => Next::state(self).ask(store::setup()),
            Event::Done { msg, .. } if msg == "setup" => {
                let read = store::list("list", &self.query);
                let note = self.note;
                if note != 0 {
                    self.open_note(note).ask(read)
                } else {
                    Next::state(self).ask(read)
                }
            }
            Event::Done { msg, last_rowid, .. } if msg == "created" => {
                let read = store::list("list", "");
                self.open_note(last_rowid as u32).ask(read)
            }
            Event::Done { msg, .. } if msg == "saved" => {
                let read = store::list("list", &self.query);
                Next::state(self).ask(read)
            }
            // After a delete, the newest note left opens.
            Event::Done { msg, .. } if msg == "deleted" => {
                let read = store::list("after_delete", &self.query);
                Next::state(self).ask(read)
            }
            Event::Rows { msg, rows, .. } if msg == "list" || msg == "after_delete" => {
                let list = store::rows(&rows);
                let first = list.first().map(|r| r.id);
                let got = Notes { list, read: true, problem: None, ..self };
                match first {
                    Some(id) if msg == "after_delete" => got.open_note(id),
                    _ => Next::state(got),
                }
            }
            Event::Rows { msg, rows, .. } if msg == "note" => self.arrived(store::read_note(&rows)),
            // Another window changed them: read them again.
            Event::Change { tables } if tables.iter().any(|t| t == "notes") => {
                let read = store::list("list", &self.query);
                let note = self.note;
                if note != 0 {
                    Next::state(self).ask(read).ask(store::note(note))
                } else {
                    Next::state(self).ask(read)
                }
            }
            Event::Failed { error, .. } => Next::state(Notes { problem: Some(error), ..self }),
            Event::Message { msg, value } => self.on_message(msg, value),
            _ => Next::state(self),
        }
    }

    fn view(&self) -> View<Msg> {
        view::page(self)
    }
}

rapidkoi::export!(Notes);

#[cfg(test)]
mod tests {
    use super::*;
    use rapidkoi::{Ask, Value};

    fn message(n: Notes, msg: Msg, value: Option<&str>) -> Next<Notes> {
        n.handle(Event::Message { msg, value: value.map(String::from) })
    }

    fn done(n: Notes, msg: &str, last_rowid: i64) -> Next<Notes> {
        n.handle(Event::Done { msg: msg.into(), changes: 1, last_rowid })
    }

    fn rows_of(n: Notes, msg: &str, rows: Vec<Vec<Value>>) -> Next<Notes> {
        n.handle(Event::Rows { msg: msg.into(), columns: vec![], rows, cut: false })
    }

    fn t(s: &str) -> Value {
        Value::Text(s.into())
    }

    fn list_row(id: i64, title: &str) -> Vec<Value> {
        vec![Value::Integer(id), t(title), t("A line"), Value::Integer(0), t("10:42"), t("5"), t("24"), t("10")]
    }

    fn note_row(id: i64, title: &str, body: &str, rev: i64) -> Vec<Value> {
        vec![
            Value::Integer(id),
            t(title),
            t(body),
            Value::Integer(rev),
            Value::Integer(0),
            t("10:42"),
            t("5"),
            t("24"),
            t("10"),
        ]
    }

    fn kinds(asks: &[Ask]) -> Vec<String> {
        asks.iter()
            .map(|a| match a {
                Ask::Sql(msg, _, _) | Ask::SqlBatch(msg, _) => msg.clone(),
                _ => String::from("other"),
            })
            .collect()
    }

    /// A window with notes 7 and 5 listed and note 7 open, at edit `rev`.
    fn with_open(rev: i64) -> Notes {
        let listed = rows_of(Notes::default(), "list", vec![list_row(7, "Garden"), list_row(5, "Books")]).into_state();
        let picked = message(listed, Msg::Pick(7), None).into_state();
        rows_of(picked, "note", vec![note_row(7, "Garden", "Sukuma first.", rev)]).into_state()
    }

    #[test]
    fn opening_makes_the_tables_then_reads_the_list() {
        let opened = Notes::default().handle(Event::Opened);
        assert_eq!(kinds(opened.asks()), vec!["setup"]);
        assert_eq!(kinds(done(opened.into_state(), "setup", 0).asks()), vec!["list"]);
    }

    #[test]
    fn a_links_note_is_read_on_opening() {
        let next = done(Notes { note: 7, ..Notes::default() }, "setup", 0);
        assert_eq!(kinds(next.asks()), vec!["note", "list"]);
        assert_eq!(next.linked(), Some(vec![String::from("note")]));
    }

    #[test]
    fn the_list_arrives() {
        let n = rows_of(Notes::default(), "list", vec![list_row(7, "Garden")]).into_state();
        assert_eq!((n.read, n.list.len(), n.list[0].title.as_str()), (true, 1, "Garden"));
    }

    #[test]
    fn picking_a_note_links_it_and_reads_it() {
        let next = message(Notes::default(), Msg::Pick(7), None);
        assert_eq!(next.linked(), Some(vec![String::from("note")]));
        assert_eq!(kinds(next.asks()), vec!["note"]);
        let n = rows_of(next.into_state(), "note", vec![note_row(7, "Garden", "Sukuma first.", 3)]).into_state();
        assert_eq!((n.open.id, n.open.body.as_str(), n.open.rev), (7, "Sukuma first.", 3));
    }

    #[test]
    fn a_link_to_a_note_thats_gone_opens_none() {
        let next = rows_of(Notes { note: 9, ..Notes::default() }, "note", vec![]);
        // Nothing open: the address names no note (never `note=0`).
        assert_eq!(next.linked(), Some(vec![]));
        assert_eq!(next.into_state().note, 0);
    }

    #[test]
    fn a_fresh_window_names_no_note() {
        assert_eq!(Notes::default().handle(Event::Init).linked(), Some(vec![]));
    }

    #[test]
    fn searching_reads_the_list_with_the_text() {
        let next = message(Notes::default(), Msg::Search, Some("garden"));
        assert_eq!(kinds(next.asks()), vec!["list"]);
        assert_eq!(next.into_state().query, "garden");
    }

    #[test]
    fn each_edit_saves_with_a_higher_rev() {
        let a = message(with_open(3), Msg::Body, Some("Sukuma"));
        assert_eq!(kinds(a.asks()), vec!["saved"]);
        let b = message(a.into_state(), Msg::Title, Some("Garden plan")).into_state();
        assert_eq!((b.open.rev, b.open.title.as_str(), b.open.body.as_str()), (5, "Garden plan", "Sukuma"));
    }

    #[test]
    fn an_older_read_never_undoes_edits_still_saving() {
        let edited = message(with_open(3), Msg::Body, Some("Newer text")).into_state();
        let n = rows_of(edited, "note", vec![note_row(7, "Garden", "Sukuma first.", 3)]).into_state();
        assert_eq!(n.open.body, "Newer text");
    }

    #[test]
    fn new_inserts_then_opens_the_new_note() {
        let next = message(Notes { query: "x".into(), ..Notes::default() }, Msg::New, None);
        assert_eq!(kinds(next.asks()), vec!["created"]);
        let made = done(next.into_state(), "created", 12);
        assert_eq!(kinds(made.asks()), vec!["note", "list"]);
        let n = made.into_state();
        assert_eq!((n.note, n.query.as_str()), (12, ""));
    }

    #[test]
    fn delete_asks_first_and_then_opens_the_newest_left() {
        let asked = message(with_open(0), Msg::AskDelete, None).into_state();
        assert!(asked.confirming);
        assert!(!message(asked, Msg::Keep, None).into_state().confirming);
        let gone = message(message(with_open(0), Msg::AskDelete, None).into_state(), Msg::Delete, None);
        assert_eq!(kinds(gone.asks()), vec!["deleted"]);
        let reread = done(gone.into_state(), "deleted", 0);
        assert_eq!(kinds(reread.asks()), vec!["after_delete"]);
        let next = rows_of(reread.into_state(), "after_delete", vec![list_row(5, "Books")]);
        assert_eq!(kinds(next.asks()), vec!["note"]);
        assert_eq!(next.into_state().note, 5);
    }

    #[test]
    fn another_window_s_change_reads_again() {
        let next = with_open(0).handle(Event::Change { tables: vec!["notes".into()] });
        assert_eq!(kinds(next.asks()), vec!["list", "note"]);
        assert!(with_open(0).handle(Event::Change { tables: vec!["other".into()] }).asks().is_empty());
    }

    #[test]
    fn a_failure_is_shown_and_notes_runs_on() {
        let n = with_open(0)
            .handle(Event::Failed { msg: "saved".into(), error: "This app has no room left (100 MB)".into() })
            .into_state();
        assert_eq!((n.problem.as_deref(), n.open.id), (Some("This app has no room left (100 MB)"), 7));
    }
}
