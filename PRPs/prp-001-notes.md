## FEATURE: Notes: write, find and link to notes, kept on this device

## OBJECTIVE
Opening Notes shows your notes down the left, newest first under Today, This week and Earlier, each with its title, when it was last changed and its first line. Clicking one opens it on the right: a large title and the text below, both edited in place and saved as you type. **+** makes a new note and opens it. Search finds notes by their title or text. Each note has its own link (`…?note=7`): opening it opens that note, in any window. Delete asks in place first. With no notes yet, the page says "No notes yet" with a New note button. Everything stays on this device; nothing is ever sent anywhere.

## CONTEXT
- The design: `design/notes-open.dc.html` and `design/notes-first-open.dc.html` (live: the canvas linked in `design/README.md`). Plain text, Rapidkoi's colours, dark and light.
- Agreed with the owner (2026-10-24, Rapidkoi's chat): Notes is the offline example (DECISION-001); notes with a title and longer text, a link per note, search, never the network.
- Files: `src/lib.rs` (the scaffold, replaced) and new modules beside it if it nears 500 lines (`src/store.rs` for the SQL, `src/view.rs` for the screens). `Cargo.toml` unchanged.
- SDK (`../rapidkoi-rust/rapidkoi/src/`, tag v0.1.0), each checked: `Ask::sql`, `Ask::sql_batch`, `params!`, `Value` (`storage.rs`); `Event::{Init, Opened, Rows, Done, Failed, Change, Message}` (`abi.rs`); `Next::state(..).link(..)`, `.ask(..)`, `#[derive(State)]`, `#[derive(Message)]` (`state.rs`, `message.rs`); `column`, `row`, `text`, `button`, `input`, `text_area`, `spacer`, `.style([...])` with `Sizing::width/max_width`, `Layout::grow`, `Background`, `Corner`, `Spacing`, `Font`, `TextSize`, `Weight` (`view.rs`, `style.rs`).
- Contract: `../rapidkoi/docs/specs/contract-6.md` (storage), `contract-5.md` (screens, styles, roles), `contract-4.md` (links).
- **The SDK has no clock.** SQLite does, and lugha-sqlite's guard allows every function but `load_extension` (`../lugha-sqlite/src/guards.la`): times come from `unixepoch()` and `date(…, 'localtime')` in the SQL.
- Decisions: DECISION-001 (resolved). **Q1 below is open.**

## Q1: scrolling (needs the owner)

Contract 5 has no scrolling box: Rapidkoi scrolls the whole page. So the list and the note can't scroll separately, as the design draws them.
- **(a) Recommended for now:** the page scrolls as one, the list beside the note. With a long note the list scrolls away with it; with many notes, the note does. Record the gap in DECISIONS.md and ask Rapidkoi for a scrolling box (a contract change: Rapidkoi's own decision and version bump), then use it here.
- (b) Wait for the scrolling box before building Notes.

## REQUIREMENTS

### Must Do
- **The database** (made on `Opened` with `CREATE TABLE IF NOT EXISTS`, versioned with `PRAGMA user_version = 1`):
  `notes (id INTEGER PRIMARY KEY AUTOINCREMENT, title TEXT NOT NULL DEFAULT '', body TEXT NOT NULL DEFAULT '', created INTEGER NOT NULL, updated INTEGER NOT NULL, rev INTEGER NOT NULL DEFAULT 0)`, with an index on `updated`. `AUTOINCREMENT` so an ID, and its link, is never reused.
- **The list:** one read: id, title, the first non-empty line of the body (cut to 120 characters), the group (`Today` when `date(updated, 'unixepoch', 'localtime') = date('now', 'localtime')`, `This week` within 7 days, else `Earlier`) and the time to show (`%H:%M` today, the weekday this week, `%d %b` before), newest first. Read again after each write and on `Change { tables }` naming `notes`.
- **Search:** typing in the search field reads the list again with `instr(lower(title), lower(?)) > 0 OR instr(lower(body), lower(?)) > 0` (parameters, never pasted in). An empty field is the whole list. No match: "No notes match “…”".
- **A note:** opened by clicking it or by the link: `note` is a linked field. The note's whole title and body are read by its ID. A link to a note that's gone opens the list with no note open.
- **Writing:** every change to the title or text is written as you type (`sql_batch`), with `updated = unixepoch()`. Each write carries the window's own count of edits for that note (`rev`) and changes the row only `WHERE id = ? AND rev < ?`, so a write that lands late never overwrites a newer one.
- **New note** (+): inserts an empty note, then opens it (its ID from `Done`'s `last_rowid`), the search cleared.
- **Delete:** the trash button asks "Delete this note?" with Keep and Delete; Delete removes the row and opens the newest note left, or none.
- **The first open, with no notes:** "No notes yet" and a New note button.
- **A failure** (`Failed`) shows Rapidkoi's sentence, dim, above the note. Notes keeps running.
- **Looks:** as the design: Rapidkoi's colour names and spacing (never raw colours), the list 296 px wide, the note's column at most 680 px, a 30 px bold title, 16 px text.

### Must NOT Do
- No network: no requests, no servers in the manifest, no sign-in.
- No formatting (bold, lists, links in text): contract 5 has plain text only.
- No folders, tags, pinning notes or sorting choices: not asked for.
- Never build SQL from text the person typed.
- Nothing kept outside the state (no `static`).

## DESIGN
- State: `list: Vec<Row>` (the window's copy), `query: String`, `note: u32` (linked; 0 = none), `open: Option<Note>` (`id`, `title`, `body`, `rev`), `confirming: bool`, `loaded: bool`, `problem: Option<String>`.
- Messages: `Search`, `Pick(u32)`, `New`, `Title`, `Body`, `AskDelete`, `Keep`, `Delete`.
- Asks, by their `msg`: `setup` (the schema), `list`, `note`, `saved`, `created`, `deleted`.
- The view in `view.rs`: `list_pane`, `note_pane`, `empty_page`; the page a `row` of the two panes.

## TESTS
- [x] `Opened` asks for the schema, then the list; the schema's SQL is exactly the table above
- [x] `Rows` for `list` fills the list, groups and times as read; an empty list shows "No notes yet"
- [x] Typing in search asks for the list with the text as a parameter, never in the SQL
- [x] Picking a note links `note` and asks for it by ID; a link's `note` asks for it on `Opened`
- [x] A link to a note that's gone opens no note
- [x] Typing in the title or text asks for a write with a higher `rev` each time, and `WHERE id = ? AND rev < ?`
- [x] New inserts, then opens the note from `last_rowid`, the search cleared
- [x] Delete asks first; Keep cancels; Delete removes and opens the newest left
- [x] `Change { tables: ["notes"] }` reads the list again; another table doesn't
- [x] `Failed` shows the sentence and the app runs on
- [x] The view: the list's rows, the open note, "No notes match …", the empty page

## ACCEPTANCE
- [x] Every test above passes
- [x] `cargo test`, `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and the wasm build pass
- [ ] Deployed with `scripts/deploy.sh` as 0.2.0
- [ ] For the owner, by eye: open `rapidkoi://notes.dafifi.net`, write two notes, search, open one by its link in a new window, delete one; close Rapidkoi, go offline, open Notes again: the notes are there

## AMENDMENTS

- **Q1 (the owner, at approval):** (a): the page scrolls as one; DECISION-002 records the gap for Rapidkoi.
- **Rows are list items,** in a list: contract 5 lets only a tab or a list item be `selected`, and Rapidkoi stopped the app when a button was. Found by driving the built app through Rapidkoi's shell, headless, with a real database (a scratch test, not kept): first open, a new note, a title saved and listed, the link `note=1`, search and no match, delete asked then the newest opened, the notes still there in a new window.
- **Found in Rapidkoi, not worked around (DECISION-003):** typing boxes get no accessible name from `label`, and a box of several lines loses its placeholder.
- **A window opened without a link** shows the list and no note open ("No note open"), as the design's empty right side.
- `cargo test` (21), `fmt`, `clippy -D warnings` and the wasm build pass. Version 0.2.0, not yet deployed.
