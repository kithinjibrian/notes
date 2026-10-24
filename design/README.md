# Notes: design

The first design for Notes, made on a Claude design canvas: https://claude.ai/artifact/StHRrD3hw1PnGKLR5ULyNS (private to the owner unless shared).

- `notes-open.dc.html`: Notes in a Rapidkoi window, a note open. The list (search, new note, notes by Today, This week and Earlier, a title, time and one-line preview each) and the note (a large title, body text in a 680 px column, words and time above, delete asked in place). Each note has its own link (`?note=7`).
- `notes-first-open.dc.html`: the first open: "No notes yet" and New note.

Both use Rapidkoi's own colours, dark and light (`../rapidkoi/src/ui/schemes.la`). Plain text only: contract 5 has no rich text.

These files are the canvas's format: they need its runtime (`support.js`) to run, so open the canvas link to click through them.
