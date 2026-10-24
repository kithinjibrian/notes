# DECISIONS.md — Notes

Each decision and its why. An `open` one blocks the work it affects: ask the owner. When one is resolved, record the outcome and move it to RESOLVED.

---

## OPEN

(none)

---

## RESOLVED

### DECISION-001 — Notes is its own repository, on the SDK by tag

**Outcome (the owner, 2026-10-24):** Notes lives in its own repository, depending on `rapidkoi-rust` by git tag (`v0.1.0`), not as an example inside the SDK.

**Why:** Rapidkoi keeps only the shell and its built-in apps (its PRP-042). Of the two example apps agreed then, Notes is the offline one: storage, links and search, nothing over the network. Trip is the other, using several servers.

**Rules out:** Network requests, declared servers and sign-in in Notes; changing the SDK from this repository.
