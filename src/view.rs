//! Notes' screens, as designed (`design/notes-open.dc.html`): the list down
//! the left, the open note on the right, and the empty page before the
//! first note. Every colour is one of Rapidkoi's names, so the person's
//! light, dark and contrast choices apply.

use crate::store::Row;
use crate::{Msg, Notes};
use rapidkoi::raw::{boxed, Role};
use rapidkoi::style::{
    Background, Border, Color, Corner, Font, Layout, Sizing, Space as Gap, Spacing, State as On, TextSize, Weight, When,
};
use rapidkoi::{button, column, input, row, spacer, text, text_area, Space, View};

/// The list's width, in pixels.
const LIST_WIDTH: u16 = 296;

/// The note's column at its widest, in pixels: lines stay easy to read.
const NOTE_WIDTH: u16 = 680;

/// Search's and +'s height, and +'s width, in pixels.
const CONTROL: u16 = 38;

/// The list's groups, in order.
const GROUPS: [&str; 3] = ["Today", "This week", "Earlier"];

/// The window: the empty page before the first note, else the list and the note.
pub fn page(n: &Notes) -> View<Msg> {
    let shown = if !n.read {
        // Nothing to show until the list arrives: it's on this device, so that's at once.
        column(Vec::new())
    } else if n.list.is_empty() && n.query.trim().is_empty() {
        empty_page()
    } else {
        // No `align` here: on the row it would shrink both panes to their content.
        row([list_pane(n), note_pane(n)]).spacing(Space::Xl)
    };
    shown.screen_title("Notes")
}

/// "No notes yet" and New note.
fn empty_page() -> View<Msg> {
    column([text("No notes yet").title(), button("New note").primary().on_press(Msg::New)])
        .spacing(Space::Md)
        .style([Layout::grow(), Layout::align(rapidkoi::style::Align::Center)])
}

/// Search and +, then the notes under their groups.
fn list_pane(n: &Notes) -> View<Msg> {
    let search = input(&n.query)
        .placeholder("Search")
        .label("Search notes")
        .on_change(Msg::Search)
        .style([Layout::grow(), Sizing::height(CONTROL)]);
    let add = button("+")
        .primary()
        .label("New note")
        .on_press(Msg::New)
        .style([Sizing::width(CONTROL), Sizing::height(CONTROL)]);
    let mut parts = vec![row([search, add]).spacing(Space::Sm)];
    for group in GROUPS {
        let rows: Vec<View<Msg>> =
            n.list.iter().filter(|r| r.group == group).map(|r| note_row(r, r.id == n.note)).collect();
        if rows.is_empty() {
            continue;
        }
        parts.push(
            column([
                text(group).dim().style([
                    Font::size(TextSize::Small),
                    Font::weight(Weight::Bold),
                    Spacing::padding_xy(Gap::None, Gap::Md),
                ]),
                // A list of items, so the open one can be marked selected (contract 5's roles).
                column(rows).spacing(Space::Xs).role(Role::List),
            ])
            .spacing(Space::Sm),
        );
    }
    if n.list.is_empty() {
        parts.push(text(&format!("No notes match “{}”", n.query.trim())).dim());
    }
    column(parts).spacing(Space::Md).style([Sizing::width(LIST_WIDTH)])
}

/// One note in the list: its title and time, and its first line. The open one is shaded and selected.
fn note_row(r: &Row, open: bool) -> View<Msg> {
    let title = if r.title.trim().is_empty() { "New note" } else { &r.title };
    let preview = if r.preview.is_empty() { "No text" } else { &r.preview };
    let top = row([
        text(title).style([Font::weight(Weight::Bold)]),
        spacer(),
        text(&r.when).dim().style([Font::size(TextSize::Small)]),
    ])
    .spacing(Space::Sm);
    let shade = Color::system("surface");
    let mut look = vec![
        Layout::column(),
        Spacing::gap(Gap::Xs),
        Spacing::padding_xy(Gap::Sm, Gap::Md),
        Border::corner(Corner::Medium),
        When(On::Hover, [Background::color(shade.clone())]),
    ];
    if open {
        look.push(Background::color(shade));
    }
    boxed()
        .role(Role::ListItem)
        .label(title)
        .selected(open)
        .on_press(Msg::Pick(r.id))
        .style(look)
        .children([top, text(preview).dim().style([Font::size(TextSize::Small)])])
}

/// The open note: when it changed and its words, delete, then its title and text.
fn note_pane(n: &Notes) -> View<Msg> {
    if n.open.id == 0 {
        return column([text("No note open").dim()]).style([Layout::grow()]);
    }
    let words = n.open.body.split_whitespace().count();
    let counted = if words == 1 { String::from("1 word") } else { format!("{words} words") };
    let tools = if n.confirming {
        row([
            text("Delete this note?"),
            button("Keep").on_press(Msg::Keep),
            button("Delete").danger().on_press(Msg::Delete),
        ])
        .spacing(Space::Sm)
    } else {
        button("Delete").subtle().label("Delete note").on_press(Msg::AskDelete)
    };
    let mut parts = vec![row([
        text(&format!("{} · {counted}", n.open.when)).dim().style([Font::size(TextSize::Small)]),
        spacer(),
        tools,
    ])
    .spacing(Space::Sm)];
    if let Some(p) = &n.problem {
        parts.push(text(p).dim());
    }
    // On the page itself: no field's edge, as the design draws it.
    let bare = || [Border::all(0, Color::system("background")), Background::color(Color::system("background"))];
    let title = input(&n.open.title)
        .placeholder("Title")
        .label("Title")
        .on_change(Msg::Title)
        .style(bare())
        .style([Font::size(TextSize::Rem(1.875)), Font::weight(Weight::Bold)]);
    let body = text_area(&n.open.body)
        .placeholder("Write something")
        .label("Note")
        .lines(20)
        .on_change(Msg::Body)
        .style(bare())
        // The rest of the note's height, so anywhere below the title takes the text.
        .style([Font::size(TextSize::Rem(1.0)), Layout::grow()]);
    parts.push(column([title, body]).spacing(Space::Md).style([Sizing::max_width(NOTE_WIDTH), Layout::grow()]));
    column(parts).spacing(Space::Lg).style([Layout::grow()])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::Note;

    fn row_of(id: u32, title: &str, group: &str) -> Row {
        Row { id, title: title.into(), preview: "A line".into(), group: group.into(), when: "10:42".into() }
    }

    /// Every text in a view, in order (what a person reads).
    fn texts(v: &View<Msg>) -> String {
        String::from_utf8_lossy(&v.encode()).into_owned()
    }

    #[test]
    fn before_any_note_it_offers_a_new_one() {
        let v = texts(&page(&Notes { read: true, ..Notes::default() }));
        assert!(v.contains("No notes yet") && v.contains("New note"));
    }

    #[test]
    fn the_list_groups_its_notes_and_the_open_one_shows() {
        let n = Notes {
            read: true,
            note: 7,
            list: vec![row_of(7, "Garden", "Today"), row_of(3, "Chapati", "Earlier")],
            open: Note {
                id: 7,
                title: "Garden".into(),
                body: "Sukuma first, then tomatoes".into(),
                rev: 0,
                when: "10:42".into(),
            },
            ..Notes::default()
        };
        let v = texts(&page(&n));
        assert!(v.contains("Today") && v.contains("Earlier") && !v.contains("This week"));
        assert!(v.contains("10:42 · 4 words"));
        assert!(v.contains("Sukuma first, then tomatoes"));
    }

    #[test]
    fn a_search_with_nothing_found_says_so() {
        let v = texts(&page(&Notes { read: true, query: "zebra".into(), ..Notes::default() }));
        assert!(v.contains("No notes match “zebra”"));
    }

    #[test]
    fn delete_asks_in_place() {
        let open = Note { id: 7, title: "Garden".into(), ..Note::default() };
        let n = Notes {
            read: true,
            note: 7,
            list: vec![row_of(7, "Garden", "Today")],
            open,
            confirming: true,
            ..Notes::default()
        };
        let v = texts(&page(&n));
        assert!(v.contains("Delete this note?") && v.contains("Keep"));
    }

    #[test]
    fn a_problem_is_shown_with_the_note() {
        let open = Note { id: 7, ..Note::default() };
        let n = Notes {
            read: true,
            note: 7,
            list: vec![row_of(7, "", "Today")],
            open,
            problem: Some("This app has no room left (100 MB)".into()),
            ..Notes::default()
        };
        let v = texts(&page(&n));
        assert!(v.contains("This app has no room left") && v.contains("New note"));
    }
}
