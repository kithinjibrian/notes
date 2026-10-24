# CLAUDE.md — Notes

Notes is a Rapidkoi app, written in Rust on the SDK `rapidkoi-rust`, that works **fully offline**: notes with a title and longer text, kept in the app's own database on this device, each with its own link (`…?note=4`), and search. It never touches the network: no requests, no servers in its manifest, no sign-in. It's the example of what an app gets from Rapidkoi alone: screens, storage, links.

Rapidkoi (the shell that runs it) is `../rapidkoi`. The SDK is `../rapidkoi-rust`, used here as a git dependency at tag `v0.1.0`.

Where things are recorded: **DECISIONS.md** (each decision and its why), **PRPs/** (a spec per feature), git's log (commit messages say what and why).

---

## HOW WE WORK

- **Discuss before building.** Read the SDK's source rather than guess its API, and say what you checked.
- **A feature needs an approved PRP** (`PRPs/prp-{NNN}-{name}.md` from `PRPs/TEMPLATE.md`). Without one: ask about it one question at a time, each with your recommended answer, then write the PRP and wait for approval. Small fixes and chores skip the PRP when the owner says so.
- **One PRP at a time**, on its own branch from `main`. Merge into `main` only when the owner asks, after the tests pass.
- **Never make an architectural choice silently**: check DECISIONS.md; an `open` entry blocks the work.
- **A missing or broken SDK or Rapidkoi feature is never worked around.** Record it in DECISIONS.md, report it, and stop: it's fixed in `../rapidkoi-rust` or `../rapidkoi` under their own rules.
- **No copy in screens beyond what the owner asked for.** Files stay at or under 500 lines.

## TESTING

- **Test first**: a failing `#[test]` before the code that passes it.
- An app is pure: `handle(self, event) -> Next<Self>` and `view(&self) -> View<Msg>`. Test it by feeding events (`Event::Init`, `Event::Message { .. }`, `Event::Rows { .. }`, `Event::Done { .. }`, `Event::Failed { .. }`, `Event::Change { tables }`) and checking the state, what it asks for (`.asks()`), what goes in the link (`.linked()`) and the view (`assert_eq!` on `View`).
- **Run the tests yourself**: `cargo test`, then `cargo fmt --check` and `cargo clippy --all-targets -- -D warnings`. A task is done when all three pass and the wasm build succeeds.
- The owner checks it on screen (`scripts/dev.sh`, then `../rapidkoi/main rapidkoi://localhost:8002`). Never claim something looks or works on screen because it compiled.

## HOW A RAPIDKOI APP WORKS (the rules that bite)

- **Everything lives in the state.** No `static`, no globals, nothing kept between calls: Rapidkoi rebuilds a window from its state (restore, reload, links), and hidden variables break that. The state derives `rapidkoi::State` and `Default`; `Default` is a fresh start.
- **Events:** `Init` (fresh), `Opened` (every open: ask for what you need here), `Resume`, `Change { tables }` (another window changed these tables: read again), `Message`, and the database's answers `Rows`, `Done`, `Failed`. Ignore the rest with `_ => Next::state(self)`.
- **Links:** `Next::state(s).link(Notes::field)` puts a field in the window's address (`.then(...)` for a nested field). A link carries an ID, never the note's content: the window that opens it reads the row.
- **Storage** (contract 6): `Ask::sql(msg, sql, params![...])` for one statement, answered by `Rows`; `Ask::sql_batch(msg, steps)` for several in one transaction, answered by `Done`. Values go in **only as parameters**, never formatted into SQL. The schema is made on open with `CREATE TABLE IF NOT EXISTS …` and versioned with `PRAGMA user_version`; a change is a migration, never a dropped table. The database is the truth: the window keeps a copy of the rows it read and reads again after its own writes and on `Change`.
- **Limits:** a call takes at most 1 second, an answer 10,000 rows or 4 MB (`cut` says rows were left out), a database 100 MB, a statement 100 KB, a batch 100 steps. A `Failed` comes with Rapidkoi's sentence: show it, don't crash.
- **Screens:** the SDK's widgets (`column`, `row`, `text`, `button`, `input`, `text_area`, `checkbox`, `tabs`, `spacer`) and `.style([...])` with its typed values. Prefer Rapidkoi's colour names and spacing (`Space::Sm`, …) over raw values, so the person's light, dark, contrast and text size apply.
- **Messages:** an enum deriving `rapidkoi::Message`, with parts of the app mapping theirs (`.map`).

## WHERE TO LOOK

- The SDK's source (the tag in use): `../rapidkoi-rust/rapidkoi/src/` (`lib.rs` App, Ask, export!; `abi.rs` Event; `state.rs` State, Next, links; `storage.rs` Value, params!; `view.rs` widgets; `style.rs`; `raw.rs`). Its docs: `cargo doc --open -p rapidkoi`.
- The shortest app: `../rapidkoi-rust/examples/counter/src/lib.rs`.
- A real app with storage, links and an editor (removed from Rapidkoi, still worth reading): `git -C ../rapidkoi show 3db1f65^:guest/rust/tasks/src/lib.rs`.
- The contract itself: `../rapidkoi/docs/specs/contract-6.md` (storage), `contract-5.md` (screens, styles, roles), `contract-4.md` (state, links), `app-events.md`.

## COMMANDS

```bash
cargo test                                               # the tests (host)
cargo fmt --check && cargo clippy --all-targets -- -D warnings
cargo build --release --target wasm32-unknown-unknown    # target/wasm32-unknown-unknown/release/notes.wasm
./scripts/dev.sh                                         # build, pack notes.rkb with ../rapidkoi/main, serve on 127.0.0.1:8002
../rapidkoi/main rapidkoi://localhost:8002               # opens a window: the owner's
```

## SECURITY AND OUTWARD ACTIONS

- Never hardcode secrets. Never build SQL from text: parameters only.
- Ask before anything that leaves this machine: creating the GitHub repository, adding a remote, pushing, tagging, publishing a signed site. Before a push or tag, check the tests' real exit status.
- `../rapidkoi`, `../rapidkoi-rust` and the other sibling repositories are read for reference and never written from here.
