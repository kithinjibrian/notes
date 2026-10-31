//! Notes: a Rapidkoi app that works fully offline (PRP-001). Notes are kept
//! in the app's own database on this device, newest first; each has its own
//! link (`?note=7`); search reads titles and text. Nothing is ever sent
//! anywhere.
//!
//! The window keeps a copy of what it read: the list, and the open note.
//! The database stays the truth: the list is read again after every write
//! and whenever another window changes the notes.

extern crate alloc;

mod attachments;
mod store;
mod view;

use attachments::Attachment;
use rapidkoi::{App, Event, Message, Next, State, View};
use store::{cut, Note, Row, MAX_BODY, MAX_TITLE};

/// The whole window.
#[derive(Debug, Default, State)]
pub struct Notes {
    /// The open note's ID, in the window's address; `None` when none is open.
    note: Option<u32>,
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
    /// The attachments last read (PRP-003), and the note they're for: kept
    /// apart from `open`, so a note read again never drops them.
    files: Vec<Attachment>,
    files_of: u32,
    /// The key a file is being chosen for, while Rapidkoi's dialog is open.
    attaching: Option<String>,
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
    /// Attach: a key from SQLite, then Rapidkoi's dialog.
    Attach,
    /// Save a copy of this attachment (its key) where the person chooses.
    SaveCopy(String),
    /// Take this attachment (its key) away.
    Remove(String),
}

impl Notes {
    /// The note `id` open, in the link, and read whole.
    fn open_note(self, id: u32) -> Next<Self> {
        Next::state(Notes { note: Some(id), confirming: false, ..self })
            .link(Notes::note)
            .ask(store::note(id))
            .ask(attachments::read(id))
    }

    /// No note open, and none in the address. A link can still set `note`:
    /// contract 7's `kinds` say it's a number while it's `None`.
    fn close_note(self) -> Next<Self> {
        Next::state(Notes { note: None, open: Note::default(), confirming: false, ..self }).no_link()
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
            // One dialog at a time: a press while one is open does nothing.
            Msg::Attach if id != 0 && self.attaching.is_none() => {
                Next::state(Notes { attaching: Some(String::new()), ..self }).ask(attachments::new_key())
            }
            Msg::SaveCopy(key) => match self.files.iter().find(|f| f.key == key) {
                Some(f) => {
                    let copy = attachments::copy(&f.key, &f.name);
                    Next::state(self).ask(copy)
                }
                None => Next::state(self),
            },
            Msg::Remove(key) => Next::state(self).ask(attachments::remove(&key)),
            _ => Next::state(self),
        }
    }

    /// A note read by its ID: opened, unless this window has edits the
    /// database hasn't caught up with; gone, the link closes.
    fn arrived(self, note: Option<Note>) -> Next<Self> {
        match note {
            Some(n) if Some(n.id) == self.note => {
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
                match self.note {
                    Some(note) => self.open_note(note).ask(read),
                    None => Next::state(self).ask(read),
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
            Event::Rows { msg, rows, .. } if msg == "files" => {
                let (note, files) = attachments::rows(&rows);
                let files_of = note.unwrap_or(self.open.id);
                Next::state(Notes { files, files_of, ..self })
            }
            Event::Rows { msg, rows, .. } if msg == "new_key" => match attachments::key_of(&rows) {
                Some(key) => {
                    let choose = attachments::choose(&key);
                    Next::state(Notes { attaching: Some(key), ..self }).ask(choose)
                }
                None => Next::state(Notes { attaching: None, ..self }),
            },
            Event::FileOpened { msg, key, name, size, .. } if msg == "attached" => {
                let keep = attachments::keep(self.open.id, &key, &name, size);
                Next::state(self).ask(keep)
            }
            Event::FileCancelled { msg } if msg == "attached" => Next::state(Notes { attaching: None, ..self }),
            Event::Done { msg, .. } if msg == "kept" || msg == "removed" || msg == "discarded" => {
                let read = attachments::read(self.open.id);
                Next::state(Notes { attaching: None, ..self }).ask(read)
            }
            // The file was taken but its row wasn't: its blob goes too.
            Event::Failed { msg, error } if msg == "kept" => {
                let gone = self.attaching.clone().unwrap_or_default();
                let next = Next::state(Notes { attaching: None, problem: Some(error), ..self });
                if gone.is_empty() {
                    next
                } else {
                    next.ask(attachments::discard(&gone))
                }
            }
            Event::Failed { msg, error } if msg == "attached" || msg == "new_key" => {
                Next::state(Notes { attaching: None, problem: Some(error), ..self })
            }
            Event::Change { tables } if tables.iter().any(|t| t == "attachments") && self.open.id != 0 => {
                let read = attachments::read(self.open.id);
                Next::state(self).ask(read)
            }
            // Another window changed them: read them again.
            Event::Change { tables } if tables.iter().any(|t| t == "notes") => {
                let read = store::list("list", &self.query);
                match self.note {
                    Some(note) => Next::state(self).ask(read).ask(store::note(note)),
                    None => Next::state(self).ask(read),
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
                Ask::FileOpen(f) => alloc::format!("file_open {}", f.key),
                Ask::FileSaveAs(f) => alloc::format!("file_save_as {}", f.key),
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
        let next = done(Notes { note: Some(7), ..Notes::default() }, "setup", 0);
        assert_eq!(kinds(next.asks()), vec!["note", "files", "list"]);
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
        assert_eq!(kinds(next.asks()), vec!["note", "files"]);
        let n = rows_of(next.into_state(), "note", vec![note_row(7, "Garden", "Sukuma first.", 3)]).into_state();
        assert_eq!((n.open.id, n.open.body.as_str(), n.open.rev), (7, "Sukuma first.", 3));
    }

    #[test]
    fn a_link_to_a_note_thats_gone_opens_none() {
        let next = rows_of(Notes { note: Some(9), ..Notes::default() }, "note", vec![]);
        // Nothing open: the address names no note (never `note=0`).
        assert_eq!(next.linked(), Some(vec![]));
        assert_eq!(next.into_state().note, None);
    }

    #[test]
    fn a_fresh_window_names_no_note() {
        assert_eq!(Notes::default().note, None);
        assert_eq!(Notes::default().handle(Event::Init).linked(), Some(vec![]));
    }

    #[test]
    fn a_link_can_set_the_note_though_it_starts_empty() {
        // Contract 7's kinds: `note` is a number even while it's None.
        assert!(<Notes as rapidkoi::State>::kinds().contains(&(String::from("note"), "number")));
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
        assert_eq!(kinds(made.asks()), vec!["note", "files", "list"]);
        let n = made.into_state();
        assert_eq!((n.note, n.query.as_str()), (Some(12), ""));
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
        assert_eq!(kinds(next.asks()), vec!["note", "files"]);
        assert_eq!(next.into_state().note, Some(5));
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

    #[test]
    fn attaching_asks_for_a_key_then_the_dialog_then_keeps_the_file() {
        let pressed = message(with_open(0), Msg::Attach, None);
        assert_eq!(kinds(pressed.asks()), vec!["new_key"]);
        // A second press while it's under way asks nothing.
        assert!(message(pressed.into_state(), Msg::Attach, None).asks().is_empty());
        let keyed = rows_of(message(with_open(0), Msg::Attach, None).into_state(), "new_key", vec![vec![t("file-0f")]]);
        assert_eq!(kinds(keyed.asks()), vec!["file_open file-0f"]);
        let opened = keyed.into_state().handle(Event::FileOpened {
            msg: "attached".into(),
            key: "file-0f".into(),
            name: "photo.jpg".into(),
            size: 2048,
            handle: None,
        });
        assert_eq!(kinds(opened.asks()), vec!["kept"]);
        let kept = done(opened.into_state(), "kept", 0);
        assert_eq!(kinds(kept.asks()), vec!["files"]);
        assert_eq!(kept.into_state().attaching, None);
    }

    #[test]
    fn cancelling_the_dialog_asks_nothing() {
        let keyed = rows_of(message(with_open(0), Msg::Attach, None).into_state(), "new_key", vec![vec![t("file-0f")]]);
        let n = keyed.into_state().handle(Event::FileCancelled { msg: "attached".into() });
        assert!(n.asks().is_empty());
        assert_eq!(n.into_state().attaching, None);
    }

    #[test]
    fn a_keep_that_fails_removes_its_blob_and_says_why() {
        let keyed = rows_of(message(with_open(0), Msg::Attach, None).into_state(), "new_key", vec![vec![t("file-0f")]]);
        let failed = keyed
            .into_state()
            .handle(Event::Failed { msg: "kept".into(), error: "This app's files are full (1 GB)".into() });
        assert_eq!(kinds(failed.asks()), vec!["discarded"]);
        assert_eq!(failed.into_state().problem.as_deref(), Some("This app's files are full (1 GB)"));
        let refused =
            rows_of(message(with_open(0), Msg::Attach, None).into_state(), "new_key", vec![vec![t("file-1")]])
                .into_state()
                .handle(Event::Failed { msg: "attached".into(), error: "This file is over 16 MB".into() })
                .into_state();
        assert_eq!((refused.problem.as_deref(), refused.attaching), (Some("This file is over 16 MB"), None));
    }

    #[test]
    fn attachments_arrive_save_a_copy_and_go() {
        let rows = vec![vec![Value::Integer(7), t("file-b"), t("report.pdf"), Value::Integer(10)]];
        let n = rows_of(with_open(0), "files", rows).into_state();
        assert_eq!((n.files_of, n.files.len()), (7, 1));
        let copy = message(n, Msg::SaveCopy("file-b".into()), None);
        assert_eq!(kinds(copy.asks()), vec!["file_save_as file-b"]);
        let gone = message(copy.into_state(), Msg::Remove("file-b".into()), None);
        assert_eq!(kinds(gone.asks()), vec!["removed"]);
        assert_eq!(kinds(done(gone.into_state(), "removed", 0).asks()), vec!["files"]);
    }

    #[test]
    fn another_window_s_attachment_reads_them_again() {
        let next = with_open(0).handle(Event::Change { tables: vec!["attachments".into()] });
        assert_eq!(kinds(next.asks()), vec!["files"]);
    }
}
