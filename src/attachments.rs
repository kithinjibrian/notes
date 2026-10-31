//! A note's attachments (PRP-003): files the person chose in Rapidkoi's
//! dialog, each kept as a blob that belongs to its row in `attachments`,
//! so removing the row (or deleting the note) removes the file. Pictures
//! show in the note; other files show as a row with their size and a way to
//! save a copy out. Notes learns a file's name and size, never where it was.

use alloc::string::String;
use alloc::vec::Vec;
use rapidkoi::style::{Font, Layout, Sizing, TextSize};
use rapidkoi::{button, column, image_blob, params, row, text, Ask, Fit, Space, Step, Value, View};

use crate::Msg;

/// One file kept with the note.
#[derive(Debug, Clone, Default, PartialEq, rapidkoi::State)]
pub struct Attachment {
    /// Its blob's key, and its row's.
    pub key: String,
    /// The file's name when it was attached: all Notes knows of where it came from.
    pub name: String,
    /// In bytes.
    pub size: u64,
}

/// A new key, from SQLite: random, so two windows attaching at once never pick the same.
pub fn new_key() -> Ask {
    Ask::sql("new_key", "SELECT 'file-' || lower(hex(randomblob(8)))", Vec::new())
}

/// The key a `new_key` read answered with.
pub fn key_of(rows: &[Vec<Value>]) -> Option<String> {
    rows.first().and_then(|r| r.first()).map(|v| v.text()).filter(|k| !k.is_empty())
}

/// Rapidkoi's dialog, the chosen file going into blob `key`.
pub fn choose(key: &str) -> Ask {
    Ask::file_open("attached", key).into()
}

/// The note's attachments, in the order they were attached.
pub fn read(note: u32) -> Ask {
    Ask::sql(
        "files",
        "SELECT note, key, name, size FROM attachments WHERE note = ? ORDER BY created, rowid",
        params![note],
    )
}

/// The file now in blob `key` kept with note `note`: its row, and the blob given to the row, in one write.
pub fn keep(note: u32, key: &str, name: &str, size: u64) -> Ask {
    Ask::sql_batch_steps(
        "kept",
        alloc::vec![
            Step::sql(
                "INSERT INTO attachments (key, note, name, size, created) VALUES (?, ?, ?, ?, unixepoch())",
                params![key, note, name, size as i64],
            ),
            Step::blob_own(key, "attachments", key),
        ],
    )
}

/// A blob no row came to name (its keep failed): removed, so nothing is kept for nothing.
pub fn discard(key: &str) -> Ask {
    Ask::sql_batch_steps("discarded", alloc::vec![Step::blob_delete(key)])
}

/// An attachment taken away; Rapidkoi removes its file with its row.
pub fn remove(key: &str) -> Ask {
    Ask::sql_batch_steps("removed", alloc::vec![Step::sql("DELETE FROM attachments WHERE key = ?", params![key])])
}

/// Rapidkoi's save dialog, offering the file's own name.
pub fn copy(key: &str, name: &str) -> Ask {
    Ask::file_save_as("copied", key).name(name).into()
}

/// The note a `read` answered for (`None` when it had none), and its attachments.
pub fn rows(rows: &[Vec<Value>]) -> (Option<u32>, Vec<Attachment>) {
    let note = rows.first().filter(|r| r.len() >= 4).map(|r| r[0].int() as u32);
    let got = rows
        .iter()
        .filter(|r| r.len() >= 4)
        .map(|r| Attachment { key: r[1].text(), name: r[2].text(), size: r[3].int().max(0) as u64 })
        .collect();
    (note, got)
}

/// Whether a file shows as a picture: by its name; Rapidkoi checks the bytes.
pub fn is_picture(name: &str) -> bool {
    let lower = name.to_lowercase();
    lower.ends_with(".png") || lower.ends_with(".jpg") || lower.ends_with(".jpeg")
}

/// A size as a person reads it: "512 bytes", "2 KB", "2.4 MB".
pub fn size_words(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = 1024 * 1024;
    if bytes < KB {
        alloc::format!("{bytes} bytes")
    } else if bytes < MB {
        alloc::format!("{} KB", bytes / KB)
    } else {
        // Rounded to the nearest tenth, as a person reads it.
        let tenths = (bytes * 10 + MB / 2) / MB;
        alloc::format!("{}.{} MB", tenths / 10, tenths % 10)
    }
}

/// The attachments below the note's text: pictures at the note's width, other files as rows.
pub fn view(files: &[Attachment]) -> View<Msg> {
    let small = || [Font::size(TextSize::Rem(0.875))];
    column(files.iter().map(|a| {
        let remove = button("Remove").on_press(Msg::Remove(a.key.clone()));
        if is_picture(&a.name) {
            column([image_blob(&a.key).fit(Fit::Contain).style([Sizing::max_width(680)]), row([remove])])
                .spacing(Space::Sm)
        } else {
            row([
                text(&a.name).style([Layout::grow()]),
                text(&size_words(a.size)).dim().style(small()),
                button("Save a copy").on_press(Msg::SaveCopy(a.key.clone())),
                remove,
            ])
            .spacing(Space::Sm)
        }
    }))
    .spacing(Space::Md)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn t(s: &str) -> Value {
        Value::Text(s.into())
    }

    #[test]
    fn keeping_inserts_the_row_and_gives_it_the_blob() {
        match keep(7, "file-ab", "photo.jpg", 2048) {
            Ask::SqlBatch(msg, steps) => {
                assert_eq!(msg, "kept");
                assert!(matches!(&steps[0], Step::Sql(sql, _) if sql.starts_with("INSERT INTO attachments")));
                assert_eq!(steps[1], Step::blob_own("file-ab", "attachments", "file-ab"));
            }
            _ => panic!("expected a write"),
        }
    }

    #[test]
    fn a_discard_removes_the_blob_and_a_remove_the_row() {
        match discard("file-ab") {
            Ask::SqlBatch(_, steps) => assert_eq!(steps, alloc::vec![Step::blob_delete("file-ab")]),
            _ => panic!("expected a write"),
        }
        match remove("file-ab") {
            Ask::SqlBatch(msg, steps) => {
                assert_eq!(msg, "removed");
                assert!(
                    matches!(&steps[0], Step::Sql(sql, p) if sql.starts_with("DELETE FROM attachments") && *p == params!["file-ab"])
                );
            }
            _ => panic!("expected a write"),
        }
    }

    #[test]
    fn rows_read_their_note_and_files() {
        let (note, files) = rows(&[
            alloc::vec![Value::Integer(7), t("file-a"), t("photo.jpg"), Value::Integer(2048)],
            alloc::vec![Value::Integer(7), t("file-b"), t("report.pdf"), Value::Integer(10)],
        ]);
        assert_eq!(note, Some(7));
        assert_eq!(files.iter().map(|f| f.name.as_str()).collect::<Vec<_>>(), alloc::vec!["photo.jpg", "report.pdf"]);
        assert_eq!(rows(&[]), (None, Vec::new()));
        assert_eq!(key_of(&[alloc::vec![t("file-0f")]]), Some(String::from("file-0f")));
    }

    #[test]
    fn pictures_are_known_by_name() {
        assert!(is_picture("Photo.JPG") && is_picture("a.jpeg") && is_picture("x.png"));
        assert!(!is_picture("report.pdf") && !is_picture("png"));
    }

    #[test]
    fn sizes_read_as_a_person_says_them() {
        assert_eq!(size_words(512), "512 bytes");
        assert_eq!(size_words(2048), "2 KB");
        assert_eq!(size_words(2_516_582), "2.4 MB");
    }

    #[test]
    fn a_picture_shows_and_a_file_offers_a_copy() {
        let v = view(&[
            Attachment { key: "file-a".into(), name: "photo.jpg".into(), size: 2048 },
            Attachment { key: "file-b".into(), name: "report.pdf".into(), size: 2_516_582 },
        ]);
        let shown = String::from_utf8_lossy(&v.encode()).into_owned();
        assert!(shown.contains("blob:file-a"));
        assert!(shown.contains("report.pdf") && shown.contains("2.4 MB") && shown.contains("Save a copy"));
        // Each has Remove (the encoded view holds its label and its message).
        assert!(shown.contains("Remove"));
        assert!(!shown.contains("blob:file-b"));
    }
}
