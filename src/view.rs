//! Notes' screens, as designed (`design/notes-open.dc.html`): the list down
//! the left, the open note on the right, and the empty page before the
//! first note. Every colour is one of Rapidkoi's names, so the person's
//! light, dark and contrast choices apply.

use crate::store::Row;
use crate::{Msg, Notes};
use rapidkoi::raw::{boxed, Role};
use rapidkoi::style::{
    Align, Background, Border, Color, Corner, Font, Layout, Sizing, Space as Gap, Spacing, State as On, Text, TextSize,
    Weight, When,
};
use rapidkoi::{button, column, image, input, row, text, text_area, Space, View};

/// The list's width, in pixels (the draft's 320).
const LIST_WIDTH: u16 = 320;

/// The note's column at its widest, in pixels: lines stay easy to read.
const NOTE_WIDTH: u16 = 680;

/// Search's height, in pixels (+ matches it: its icon and the room around it).
const CONTROL: u16 = 38;

/// +'s add icon, and the room around it in its button (38 px across), in pixels.
const ADD_ICON: u16 = 18;
const AROUND_ADD: u16 = 10;

/// The search icon, in pixels.
const SEARCH_ICON: u16 = 16;

/// Delete's trash icon, and the room around it in its round button (36 px across), in pixels.
const TRASH_ICON: u16 = 18;
const AROUND_TRASH: u16 = 9;

/// The corners of search, + and the list's rows, in pixels.
const CORNER: u16 = 10;

/// The room around the list and the note, in pixels: 24 above and at the window's edge, 48 beside the note.
const EDGE: u16 = 24;
const BESIDE_NOTE: u16 = 48;

/// The list's groups, in order.
const GROUPS: [&str; 3] = ["Today", "This week", "Earlier"];

/// The draft's text sizes, as multiples of the body size, so the person's text size applies.
const ROW_TITLE: f64 = 0.875;
const PREVIEW: f64 = 0.8125;
const SMALL: f64 = 0.75;
const NOTE_TITLE: f64 = 1.875;

/// The window: the empty page before the first note, else the list and the note.
pub fn page(n: &Notes) -> View<Msg> {
    let shown = if !n.read {
        // Nothing to show until the list arrives: it's on this device, so that's at once.
        column(Vec::new())
    } else if n.list.is_empty() && n.query.trim().is_empty() {
        empty_page()
    } else {
        row([list_pane(n), divider(), note_pane(n)])
    };
    // Rapidkoi never scrolls an app's page: Notes fills the window.
    shown.style([Layout::grow()]).screen_title("Notes")
}

/// "No notes yet" and New note.
fn empty_page() -> View<Msg> {
    column([text("No notes yet").title(), button("New note").primary().on_press(Msg::New)])
        .spacing(Space::Md)
        .style([Layout::grow(), Layout::align(Align::Center)])
}

/// The line between the list and the note, clear of the window's edges.
fn divider() -> View<Msg> {
    column([boxed().style([Sizing::width(1), Background::color(Color::system("border")), Layout::grow()])])
        .style([Spacing::padding_xy(Gap::Px(EDGE), Gap::None)])
}

/// Search and +, then the notes under their groups, which scroll on their own.
fn list_pane(n: &Notes) -> View<Msg> {
    // One field, outlined: the icon, then the text with no edge of its own.
    let icon = image("asset:search.svg").color(Color::system("text_dim")).style([
        Sizing::width(SEARCH_ICON),
        Sizing::height(SEARCH_ICON),
        Layout::align(Align::Center),
    ]);
    let typed = input(&n.query).placeholder("Search").label("Search notes").on_change(Msg::Search).style([
        Border::all(0, Color::system("background")),
        Background::color(Color::system("background")),
        Font::size(TextSize::Rem(ROW_TITLE)),
        Layout::grow(),
        Layout::align(Align::Center),
    ]);
    let search = row([icon, typed]).spacing(Space::Sm).style([
        Layout::grow(),
        Sizing::height(CONTROL),
        Spacing::padding_xy(Gap::None, Gap::Md),
        Border::all(1, Color::system("border")),
        Border::corner(Corner::Px(CORNER)),
    ]);
    // The add icon, centred by equal room around it: 10 + 18 + 10 = search's height.
    let accent = Color::system("accent");
    let add = boxed()
        .role(Role::Button)
        .label("New note")
        .on_press(Msg::New)
        .style([
            Spacing::padding(Gap::Px(AROUND_ADD)),
            Border::corner(Corner::Px(CORNER)),
            Background::color(accent.clone()),
            When(On::Hover, [Background::color(accent.clone().mix(Color::system("text"), 0.1))]),
            When(On::Pressed, [Background::color(accent.mix(Color::system("text"), 0.2))]),
        ])
        .children([image("asset:add.svg")
            .color(Color::system("on_accent"))
            .style([Sizing::width(ADD_ICON), Sizing::height(ADD_ICON)])]);
    let mut groups = Vec::new();
    for group in GROUPS {
        let rows: Vec<View<Msg>> =
            n.list.iter().filter(|r| r.group == group).map(|r| note_row(r, Some(r.id) == n.note)).collect();
        if rows.is_empty() {
            continue;
        }
        groups.push(
            column([
                text(group).dim().style([
                    Font::size(TextSize::Rem(SMALL)),
                    Font::weight(Weight::Bold),
                    Spacing::padding_xy(Gap::Sm, Gap::Md),
                ]),
                // A list of items, so the open one can be marked selected (contract 7's roles).
                column(rows).spacing(Space::Px(2)).role(Role::List),
            ])
            .spacing(Space::Xs),
        );
    }
    if n.list.is_empty() {
        groups.push(
            text(&format!("No notes match “{}”", n.query.trim()))
                .dim()
                .style([Font::size(TextSize::Rem(PREVIEW)), Spacing::padding_xy(Gap::Lg, Gap::Md)]),
        );
    }
    // Search and + stay; the notes under them scroll on their own, in the height left.
    let notes = column(groups).style([Layout::scroll(), Layout::grow()]);
    column([row([search, add]).spacing(Space::Sm), notes])
        .spacing(Space::Md)
        .style([Sizing::width(LIST_WIDTH), Spacing::padding_xy(Gap::Px(EDGE), Gap::Lg)])
}

/// One note in the list: its title and time, and its first line, each one line. The open one is shaded and selected.
fn note_row(r: &Row, open: bool) -> View<Msg> {
    let title = if r.title.trim().is_empty() { "New note" } else { &r.title };
    let preview = if r.preview.is_empty() { "No text" } else { &r.preview };
    let top = row([
        // It takes the room the time leaves, so the time stays at the end.
        text(title).style([
            Font::size(TextSize::Rem(ROW_TITLE)),
            Font::weight(Weight::Bold),
            Text::truncate(),
            Layout::grow(),
        ]),
        text(&r.when).dim().style([Font::size(TextSize::Rem(SMALL))]),
    ])
    .spacing(Space::Sm);
    let shade = Color::system("surface");
    let mut look = vec![
        Layout::column(),
        Spacing::gap(Gap::Xs),
        Spacing::padding_xy(Gap::Px(10), Gap::Md),
        Border::corner(Corner::Px(CORNER)),
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
        .children([top, text(preview).dim().style([Font::size(TextSize::Rem(PREVIEW)), Text::truncate()])])
}

/// The open note: when it changed and its words, delete; then its title and text in a centred column.
fn note_pane(n: &Notes) -> View<Msg> {
    let room = [Spacing::padding_xy(Gap::Px(EDGE), Gap::Px(BESIDE_NOTE)), Layout::grow()];
    if n.open.id == 0 {
        return column([text("No note open").dim().style([Layout::align(Align::Center)])]).style(room);
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
        // A round button holding the trash icon, as the draft draws it.
        boxed()
            .role(Role::Button)
            .label("Delete note")
            .on_press(Msg::AskDelete)
            .style([
                Layout::align(Align::Center),
                Spacing::padding(Gap::Px(AROUND_TRASH)),
                Border::corner(Corner::Round),
                When(On::Hover, [Background::color(Color::system("surface"))]),
            ])
            .children([image("asset:trash.svg")
                .color(Color::system("text"))
                .style([Sizing::width(TRASH_ICON), Sizing::height(TRASH_ICON)])])
    };
    // The time and words take the room delete leaves, on one line.
    let when = text(&format!("{} · {counted}", n.open.when)).dim().style([
        Font::size(TextSize::Rem(SMALL)),
        Text::truncate(),
        Layout::grow(),
        Layout::align(Align::Center),
    ]);
    let mut parts = vec![row([when, tools]).spacing(Space::Sm).style([Sizing::height(CONTROL)])];
    if let Some(p) = &n.problem {
        parts.push(text(p).dim());
    }
    // On the page itself: no field's edge, as the draft draws it.
    let bare = || [Border::all(0, Color::system("background")), Background::color(Color::system("background"))];
    let title = input(&n.open.title)
        .placeholder("Title")
        .label("Title")
        .on_change(Msg::Title)
        .style(bare())
        .style([Font::size(TextSize::Rem(NOTE_TITLE)), Font::weight(Weight::Bold)]);
    let body = text_area(&n.open.body)
        .placeholder("Write something")
        .label("Note")
        .lines(20)
        .on_change(Msg::Body)
        .style(bare())
        // The rest of the note's height, so anywhere below the title takes the text.
        .style([Font::size(TextSize::Rem(1.0)), Layout::grow()]);
    // At most 680 px, from the left of the room beside the list. The draft
    // centres it; that waits on kivuli (Rapidkoi's DECISION-061), then
    // `Layout::align(Align::Center)` goes back here.
    parts.push(column([title, body]).spacing(Space::Md).style([
        Sizing::max_width(NOTE_WIDTH),
        Layout::grow(),
        Spacing::padding_xy(Gap::Lg, Gap::None),
    ]));
    column(parts).style(room)
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
            note: Some(7),
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

    /// A window with note 7 open.
    fn open_seven() -> Notes {
        Notes {
            read: true,
            note: Some(7),
            list: vec![row_of(7, "Garden", "Today")],
            open: Note { id: 7, title: "Garden".into(), ..Note::default() },
            ..Notes::default()
        }
    }

    #[test]
    fn the_list_scrolls_on_its_own_and_the_page_has_room_around_it() {
        // Rapidkoi never scrolls an app's page (contract 7).
        let v = texts(&page(&open_seven()));
        assert!(v.contains("scroll") && v.contains("vertical"));
        assert!(v.contains("padding"));
    }

    #[test]
    fn the_list_s_lines_and_the_note_s_time_stay_one_line() {
        // Long text wraps by default; these are cut instead: a row's title and preview, and the time and words.
        let v = texts(&page(&open_seven()));
        assert_eq!(v.matches("truncate").count(), 3);
    }

    #[test]
    fn search_add_and_delete_show_their_icons() {
        let v = texts(&page(&open_seven()));
        assert!(v.contains("asset:search.svg") && v.contains("asset:trash.svg") && v.contains("asset:add.svg"));
        assert!(v.contains("Delete note"));
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
            note: Some(7),
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
            note: Some(7),
            list: vec![row_of(7, "", "Today")],
            open,
            problem: Some("This app has no room left (100 MB)".into()),
            ..Notes::default()
        };
        let v = texts(&page(&n));
        assert!(v.contains("This app has no room left") && v.contains("New note"));
    }
}
