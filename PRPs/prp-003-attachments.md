## FEATURE: Attachments: a note keeps files the person chose, pictures shown in it, other files saved back out

## OBJECTIVE
In an open note, **Attach** opens Rapidkoi's dialog ("Open a file for Notes"); the person picks any file, and it's kept with the note. Pictures (PNG, JPEG) show below the note's text, at the note's width; other files show as a row with their name and size and **Save a copy**, which writes it wherever the person chooses. **Remove** takes an attachment away without asking first (it only removes Notes' own copy). Deleting a note takes its attachments with it. Everything stays on this device; Notes still never touches the network.

## CONTEXT
- **The owner's answers (2026-11-01):** any file, not only pictures; the words "Attach", "Save a copy", "Remove", with no question before removing; attachments below the text, pictures at the note's width, other files as compact rows.
- **Rapidkoi:** contract 9 with files (`../rapidkoi/docs/specs/contract-9.md`, "Blobs" and "The person's files"; `../rapidkoi/docs/specs/files.md`, DECISION-089 there). The app learns a file's name and size, never its path.
- **SDK:** `../rapidkoi-rust` at `v0.3.0-pre.4`, checked: `Ask::file_open(msg, key)` (`files.rs`, `lib.rs`:291), `Ask::file_save_as(msg, key).name(…)` (`lib.rs`:306), `Step::blob_own(key, table, id)` (`blobs.rs`:89), `Ask::sql_batch_steps` (`lib.rs`:237), `image_blob(key)` (`picture.rs`:53), `Event::FileOpened`, `FileSavedAs`, `FileCancelled` (`abi.rs`:116–123).
- **Files to read first:** `src/lib.rs` (340 lines: state, messages, `handle`), `src/store.rs` (242: schema, reads, writes), `src/view.rs` (364: the note's column).
- **Creates:** `src/attachments.rs` (the attachments' reads, writes and rows; `store.rs` and `view.rs` would pass 500 otherwise), its tests inside it as the other modules have theirs.
- **Changes:** `Cargo.toml` (the SDK tag), `src/lib.rs`, `src/store.rs` (schema version 2, deleting a note's attachments), `src/view.rs` (the attachments below the text), `rapidkoi.toml` and `Cargo.toml` versions (0.7.0), `CLAUDE.md` (it still names `v0.2.0-pre.4` and contract 7).
- **Decisions:** none open.

## REQUIREMENTS

### Must Do
- **The SDK at `v0.3.0-pre.4`.**
- **The table**, migration to `user_version` 2: `attachments (key TEXT PRIMARY KEY, note INTEGER NOT NULL, name TEXT NOT NULL, size INTEGER NOT NULL, created INTEGER NOT NULL)`, an index on `note`. Made by the same setup batch, with `IF NOT EXISTS`; nothing is dropped.
- **Attaching:** Attach asks SQLite for a new key (`SELECT 'file-' || hex(randomblob(8))`: unique across windows, no clock needed), then `Ask::file_open("attached", key)`. On `FileOpened`: one batch inserting the row and `Step::blob_own(key, "attachments", key)`, so the file belongs to its row. On `FileCancelled`: nothing. Then the note's attachments are read again.
- **Showing them:** below the text, in the order attached. A name ending `.png`, `.jpg` or `.jpeg` (any case) shows as `image_blob(key)`, fitted to the note's width; anything else is a row: its name, its size ("2.4 MB", "830 KB", "512 bytes"), **Save a copy**. Each has **Remove**.
- **Save a copy:** `Ask::file_save_as("copied", key).name(name)`; cancelling does nothing.
- **Remove:** a batch deleting the row; Rapidkoi removes the file with it (its owner row is gone).
- **Deleting a note** deletes its attachments' rows in the same batch, before the note.
- **Other windows:** a `Change` naming `attachments` reads the open note's attachments again.
- **Failures** show in Notes' existing problem line, in Rapidkoi's words ("This file is over 16 MB", "This app's files are full (1 GB)", …). If the batch after `FileOpened` fails, the blob it left is removed (`Step::blob_delete`), so nothing is kept that no row names.

### Must NOT Do
- Any network, any server in the manifest: unchanged.
- Words on screen beyond "Attach", "Save a copy", "Remove" and the size.
- Opening a file to modify (`edit`), several files at once, dragging files in: not asked for.
- Putting a blob key, a name or anything of an attachment in the link: the link stays `?note=4`.

## DESIGN
- **State:** `open.files: Vec<Attachment>` (`key`, `name`, `size`), read with the note; `attaching: Option<String>` (the key being chosen, between the key's answer and the dialog's), so a second press while one is open does nothing.
- **Messages:** `Attach`, `SaveCopy(String)`, `Remove(String)`.
- **Asks and answers:** `"new_key"` (Rows) → `file_open("attached")` → `FileOpened` / `FileCancelled` / `Failed` → `"kept"` (Done) → `"files"` (Rows). `"copied"` answers `FileSavedAs` (nothing to show) or `Failed`. `"removed"` and `"deleted"` answer `Done`, then the attachments or the list are read again.
- **Size words:** bytes under 1 KB, KB under 1 MB, MB with one decimal; the size is a value, not a sentence.
- **Pictures** are chosen by the name only to decide how to show them; Rapidkoi checks the bytes, and a misnamed file shows its broken-picture mark.

## TESTS
- [ ] The schema reaches version 2 from an empty database and from version 1, keeping notes.
- [ ] Attach asks for a key, then `file_open("attached", key)`; a second Attach before the dialog answers asks nothing.
- [ ] `FileOpened` gives one batch: the row's insert and `blob_own(key, "attachments", key)`; then the attachments are read.
- [ ] `FileCancelled` asks nothing and clears `attaching`.
- [ ] A failed keep batch asks `blob_delete(key)` and shows the sentence.
- [ ] Rows of `attachments` become `open.files`; a picture name and another name show as a picture and as a row with its size and Save a copy (`assert_eq!` on the view's parts).
- [ ] Remove deletes the row; Save a copy asks `file_save_as("copied", key)` with the name.
- [ ] Deleting a note's batch deletes its attachments first.
- [ ] `Change { tables: ["attachments"] }` reads them again.
- [ ] The size words: 512 → "512 bytes", 2048 → "2 KB", 2_516_582 → "2.4 MB".

## ACCEPTANCE
- [ ] Every test above passes.
- [ ] `cargo test`, `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and the wasm build pass; no file over 500 lines.
- [ ] For the owner, by eye (`scripts/dev.sh`, then `../rapidkoi/main rapidkoi://localhost:8002`): Attach shows "Open a file for Notes"; a photo shows in the note at its width; a PDF shows as a row, and Save a copy writes it elsewhere; Remove takes one away; a second window of the same note shows the attachment; deleting the note takes its attachments.

## BUILD NOTES

- **Status:** approved (the owner, 2026-11-01: "approved"); built on its branch.
- The attachments live in the state as `files` with `files_of` (the note they're for), not inside `open`: a note read again (`arrived`) would otherwise drop them, and a note's files can answer before the note itself does.
- Opening a note reads its attachments with it (`note`, then `files`).
- "Attach" sits beside delete in the note's top row; the attachments' text sizes are 0.875 rem.
- Sizes round to the nearest tenth of a MB.
- `cargo test` (38), `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and the wasm build pass; files 206, 472, 252 and about 385 lines.
