# DECISIONS.md — Notes

Each decision and its why. An `open` one blocks the work it affects: ask the owner. When one is resolved, record the outcome and move it to RESOLVED.

---

## OPEN

(none)

---

## DEFERRED

### DECISION-002 — The list and the note can't scroll on their own

**Status:** deferred (needs Rapidkoi: a scrolling box in its contract)
**Raised:** 2026-10-24, PRP-001

**Question:** The design scrolls the list and the open note separately. Contract 5 has no scrolling box: Rapidkoi scrolls the whole page, so a long note scrolls the list away, and many notes scroll the note away.

**Meanwhile (the owner, PRP-001 Q1):** Notes builds now with the page scrolling as one. Rapidkoi is asked for a scrolling box (a contract change, Rapidkoi's own decision and version bump); Notes uses it once the SDK has it. Nothing is worked around.

---

### DECISION-003 — Rapidkoi doesn't name typing boxes, and drops a text area's placeholder

**Status:** deferred (needs Rapidkoi: `src/ui/render.la`'s `typing_box`)
**Raised:** 2026-10-24, PRP-001 (found driving the built app through Rapidkoi's shell)

**Question:** Rapidkoi draws an app's typing box without its `label` as the box's accessible name, and draws a box of several lines without its placeholder. So Notes' text box shows no "Write something" when empty, and a screen reader can't name the search field ("Search notes") or the text box ("Note").

**Meanwhile:** Notes sets both as the contract says; they appear once Rapidkoi draws them. Nothing is worked around.

---

## RESOLVED

### DECISION-001 — Notes is its own repository, on the SDK by tag

**Outcome (the owner, 2026-10-24):** Notes lives in its own repository, depending on `rapidkoi-rust` by git tag (`v0.1.0`), not as an example inside the SDK.

**Why:** Rapidkoi keeps only the shell and its built-in apps (its PRP-042). Of the two example apps agreed then, Notes is the offline one: storage, links and search, nothing over the network. Trip is the other, using several servers.

**Rules out:** Network requests, declared servers and sign-in in Notes; changing the SDK from this repository.
