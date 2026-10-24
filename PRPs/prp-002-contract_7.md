## FEATURE: Notes on contract 7: the list scrolls on its own, icons from its bundle, no `note=0`

## OBJECTIVE
Notes runs on today's Rapidkoi (contract 7; 0.2.2 is refused). The list of notes scrolls on its own while the open note stays put, and a long note scrolls inside its text box. Search shows a search icon and Delete a trash icon, both drawn in the theme's colours. A window with no note open names none in its address, and a link still opens a note.

## CONTEXT
- Agreed with the owner in Rapidkoi's session (2026-10-24, after PRP-045/046/047 and kivuli 0.58): the SDK to `rapidkoi-rust` `v0.2.0-pre.3`, the scrolling list, the icons (copied from Rapidkoi's Iconsax set, `../rapidkoi/assets/icons/search.svg` and `trash.svg`, their white made `currentColor`), `note: Option<u32>`.
- Rapidkoi now: never scrolls an app's page (its PRP-046), so Notes pads and lays itself out; `Layout::scroll()` on a box; `image("asset:…").color(…)` (its PRP-047); a link can set a field that starts `None` (`kinds`, its DECISION-056); a text area's placeholder is drawn (kivuli 0.55).
- Files: `Cargo.toml`, `rapidkoi.toml` (`[assets]`), `scripts/dev.sh` (`--asset`), `src/lib.rs` (`note`), `src/view.rs`, `assets/`; DECISIONS-002 resolved, 003 updated.

## REQUIREMENTS
### Must Do
- `rapidkoi = { …, tag = "v0.2.0-pre.3" }`; version 0.3.0.
- `note: Option<u32>`: `None` is no note open (never in the address); a link's `note=4` still opens note 4.
- The page pads itself (`Space::Lg`) and fills the window; the list pane is search and + over a scrolling column of notes that takes the rest of the height.
- The search field: a dimmed search icon (16 px) before the text, inside one field.
- Delete: a round, borderless button holding the trash icon (18 px), named "Delete note".
- Both icons packed in the bundle (`[assets]` and `dev.sh`), recoloured from Rapidkoi's names.
### Must NOT Do
- No new screens or copy; the + stays a +.

## TESTS
- [x] A fresh window's `note` is `None` and its address names no note; picking a note links `note`; a link's note is read on opening; a note that's gone closes to `None`
- [x] The list's notes are in a scrolling box; the page pads itself
- [x] The search field shows `asset:search.svg`; Delete shows `asset:trash.svg`, named "Delete note"

## ACCEPTANCE
- [x] `cargo test`, `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, the wasm build
- [ ] For the owner, by eye (`scripts/dev.sh`, `../rapidkoi/main rapidkoi://localhost:8002`): many notes scroll in the list while the note stays; the icons show, light and dark; no `note=0`

## AMENDMENTS
- **Approved by the owner, 2026-10-24** (the scope above, in Rapidkoi's session: "yes").
- **Delete's round button** is its trash icon with 9 px around it (36 px across), since a box's `align` places it, not its children.
- Checked headless in Rapidkoi: the packed 0.3.0 bundle (2 assets) opens and runs on contract 7.
- **0.3.1:** the list's titles and previews are one line cut with "…" (`Text::truncate()`, SDK `v0.2.0-pre.4`; Rapidkoi's DECISION-060): Rapidkoi wraps text to its box by default, and a long preview had pushed the list wider.
- **0.3.2:** + is the add icon (`assets/add.svg`, Iconsax's add: two strokes), 22 px with 8 px around it, so it's centred in its 38 px button (the owner).
- **0.3.3 (the owner: "not faithful to what we drafted"):** the draft's measurements from `design/Notes a note open-html/Main.dc.html`: a 320 px list with 24 px around it, an outlined search field, + as an 18 px icon in a 38 px button, rows with 10 px corners and the draft's text sizes (as rems), a 1 px divider clear of the edges, and the note in a column at most 680 px wide centred in the room beside the list with 48 px either side; the time and word count stay on one line.
- **0.3.4:** the note's column fills the room from the left again (0.3.3's centring squeezed it: kivuli's clamp can't centre, DECISION-004).
