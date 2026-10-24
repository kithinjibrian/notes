# Notes

Notes for [Rapidkoi](https://github.com/kithinjibrian/rapidkoi): a title and text for each note, kept on this device in the app's own database, each with its own link, and search. It works fully offline and never uses the network.

Written in Rust on [rapidkoi-rust](https://github.com/kithinjibrian/rapidkoi-rust).

```bash
cargo test
./scripts/dev.sh                              # build, pack and serve on 127.0.0.1:8002 (needs Rapidkoi's ./main at ../rapidkoi/main)
../rapidkoi/main rapidkoi://localhost:8002    # open it
```
