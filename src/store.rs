//! The database's side of Notes: its shape, the reads and writes Notes asks
//! for, and turning rows into the window's copy. Values only ever go in as
//! parameters. The SDK has no clock, so times come from SQLite
//! (`unixepoch()`, `date(…, 'localtime')`).

use alloc::string::String;
use alloc::vec::Vec;
use rapidkoi::{params, Ask, Value};

/// The database's shape, made on every open if it isn't there yet.
/// `AUTOINCREMENT` keeps an ID, and the link to it, from ever being reused.
/// `rev` is how far a note's edits got: a write that lands late never
/// overwrites a newer one.
pub const SETUP: [&str; 3] = [
    "CREATE TABLE IF NOT EXISTS notes (id INTEGER PRIMARY KEY AUTOINCREMENT, title TEXT NOT NULL DEFAULT '', body TEXT NOT NULL DEFAULT '', created INTEGER NOT NULL, updated INTEGER NOT NULL, rev INTEGER NOT NULL DEFAULT 0)",
    "CREATE INDEX IF NOT EXISTS notes_updated ON notes (updated)",
    "PRAGMA user_version = 1",
];

/// When a note was changed, as the list groups it: 0 today, 1 this week, 2 before.
const AGE: &str = "CASE WHEN date(updated, 'unixepoch', 'localtime') = date('now', 'localtime') THEN 0 WHEN updated > unixepoch('now', '-7 days') THEN 1 ELSE 2 END";

/// The pieces of when it was changed: the time, the weekday (0 is Sunday), the day and month.
const WHEN: &str = "strftime('%H:%M', updated, 'unixepoch', 'localtime'), strftime('%w', updated, 'unixepoch', 'localtime'), strftime('%d', updated, 'unixepoch', 'localtime'), strftime('%m', updated, 'unixepoch', 'localtime')";

/// The longest title kept, in characters.
pub const MAX_TITLE: usize = 300;

/// The longest text kept, in characters: well under a value's 4 MB.
pub const MAX_BODY: usize = 200_000;

/// How long a row's preview is, in characters.
const PREVIEW: usize = 120;

/// One note in the list: what the list shows.
#[derive(Debug, Clone, Default, PartialEq, rapidkoi::State)]
pub struct Row {
    pub id: u32,
    pub title: String,
    /// Its first line of text, cut short.
    pub preview: String,
    /// `Today`, `This week` or `Earlier`.
    pub group: String,
    /// "10:42", "Wed" or "12 Oct".
    pub when: String,
}

/// The note that's open: its whole title and text.
#[derive(Debug, Clone, Default, PartialEq, rapidkoi::State)]
pub struct Note {
    /// 0: no note open.
    pub id: u32,
    pub title: String,
    pub body: String,
    /// How many edits this window has made to it, counted from the database's.
    pub rev: u32,
    pub when: String,
}

/// Making the database's tables, if they aren't there yet.
pub fn setup() -> Ask {
    Ask::sql_batch("setup", SETUP.iter().map(|s| (String::from(*s), Vec::new())).collect())
}

/// The list, newest first: every note, or those whose title or text holds `query`.
pub fn list(msg: &str, query: &str) -> Ask {
    let sql = alloc::format!(
        "SELECT id, title, substr(body, 1, 400), {AGE}, {WHEN} FROM notes WHERE ? = '' OR instr(lower(title), lower(?)) > 0 OR instr(lower(body), lower(?)) > 0 ORDER BY updated DESC, id DESC"
    );
    let q = query.trim();
    Ask::sql(msg, &sql, params![q, q, q])
}

/// One note, whole, by its ID.
pub fn note(id: u32) -> Ask {
    let sql = alloc::format!("SELECT id, title, body, rev, {AGE}, {WHEN} FROM notes WHERE id = ?");
    Ask::sql("note", &sql, params![id])
}

/// A new, empty note.
pub fn create() -> Ask {
    Ask::sql_batch(
        "created",
        alloc::vec![("INSERT INTO notes (created, updated) VALUES (unixepoch(), unixepoch())".into(), Vec::new())],
    )
}

/// A note's title, as of edit `rev`: kept only if no later edit got there first.
pub fn save_title(id: u32, title: &str, rev: u32) -> Ask {
    save("title", id, title, rev)
}

/// A note's text, as of edit `rev`: kept only if no later edit got there first.
pub fn save_body(id: u32, body: &str, rev: u32) -> Ask {
    save("body", id, body, rev)
}

fn save(column: &str, id: u32, value: &str, rev: u32) -> Ask {
    // `column` is one of two names written here, never the person's text.
    let sql = alloc::format!("UPDATE notes SET {column} = ?, updated = unixepoch(), rev = ? WHERE id = ? AND rev < ?");
    Ask::sql_batch("saved", alloc::vec![(sql, params![value, rev, id, rev])])
}

/// Removing a note.
pub fn delete(id: u32) -> Ask {
    Ask::sql_batch("deleted", alloc::vec![("DELETE FROM notes WHERE id = ?".into(), params![id])])
}

/// The list's rows, as read by `list`.
pub fn rows(rows: &[Vec<Value>]) -> Vec<Row> {
    rows.iter()
        .filter(|r| r.len() >= 8)
        .map(|r| Row {
            id: r[0].int() as u32,
            title: r[1].text(),
            preview: first_line(&r[2].text()),
            group: String::from(group(r[3].int())),
            when: when(r[3].int(), &r[4..8]),
        })
        .collect()
}

/// The open note, as read by `note`; `None` when it's gone.
pub fn read_note(rows: &[Vec<Value>]) -> Option<Note> {
    let r = rows.first().filter(|r| r.len() >= 9)?;
    Some(Note {
        id: r[0].int() as u32,
        title: r[1].text(),
        body: r[2].text(),
        rev: r[3].int() as u32,
        when: when(r[4].int(), &r[5..9]),
    })
}

/// Text cut to `most` characters.
pub fn cut(s: &str, most: usize) -> String {
    s.chars().take(most).collect()
}

/// The first line with something on it, cut to the preview's length.
fn first_line(body: &str) -> String {
    cut(body.lines().map(str::trim).find(|l| !l.is_empty()).unwrap_or(""), PREVIEW)
}

fn group(age: i64) -> &'static str {
    match age {
        0 => "Today",
        1 => "This week",
        _ => "Earlier",
    }
}

/// "10:42" today, "Wed" this week, "12 Oct" before.
fn when(age: i64, parts: &[Value]) -> String {
    const DAYS: [&str; 7] = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];
    const MONTHS: [&str; 12] = ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"];
    let number = |v: &Value| v.text().parse::<usize>().unwrap_or(0);
    match age {
        0 => parts[0].text(),
        1 => String::from(DAYS[number(&parts[1]) % 7]),
        _ => alloc::format!("{} {}", number(&parts[2]), MONTHS[(number(&parts[3]) + 11) % 12]),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn t(s: &str) -> Value {
        Value::Text(s.into())
    }

    #[test]
    fn rows_read_their_group_time_and_first_line() {
        let got = rows(&[
            alloc::vec![
                Value::Integer(7),
                t("Garden"),
                t("\n  \nSukuma first.\nThen tomatoes."),
                Value::Integer(0),
                t("10:42"),
                t("5"),
                t("24"),
                t("10")
            ],
            alloc::vec![Value::Integer(5), t("Books"), t(""), Value::Integer(1), t("09:00"), t("3"), t("22"), t("10")],
            alloc::vec![
                Value::Integer(3),
                t("Chapati"),
                t("Warm water"),
                Value::Integer(2),
                t("08:00"),
                t("0"),
                t("03"),
                t("10")
            ],
        ]);
        assert_eq!(
            got.iter().map(|r| (r.id, r.preview.as_str(), r.group.as_str(), r.when.as_str())).collect::<Vec<_>>(),
            alloc::vec![
                (7, "Sukuma first.", "Today", "10:42"),
                (5, "", "This week", "Wed"),
                (3, "Warm water", "Earlier", "3 Oct")
            ]
        );
    }

    #[test]
    fn a_short_row_is_skipped_and_a_missing_note_is_none() {
        assert!(rows(&[alloc::vec![Value::Integer(1)]]).is_empty());
        assert_eq!(read_note(&[]), None);
    }

    #[test]
    fn searching_passes_the_text_only_as_parameters() {
        match list("list", "  It's 100%  ") {
            Ask::Sql(_, sql, params) => {
                assert!(!sql.contains("It's"));
                assert_eq!(
                    params,
                    params![String::from("It's 100%"), String::from("It's 100%"), String::from("It's 100%")]
                );
            }
            _ => panic!("expected a read"),
        }
    }

    #[test]
    fn a_save_only_lands_on_an_older_edit() {
        match save_body(4, "Hello", 9) {
            Ask::SqlBatch(_, steps) => {
                assert!(steps[0].0.ends_with("WHERE id = ? AND rev < ?"));
                assert_eq!(steps[0].1, params![String::from("Hello"), 9u32, 4u32, 9u32]);
            }
            _ => panic!("expected a write"),
        }
    }
}
