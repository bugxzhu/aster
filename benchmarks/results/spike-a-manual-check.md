# Spike A manual check (Review Focus items)

Date: 2026-10-25
Application: `spikes/file-manager-slint` (debug build, macOS, Slint 1.18.1)

Automated evidence (run by the agent):

```
cargo test -p file-manager-slint      # mock data (5 tests) and locale packs (4 tests): all pass
cargo clippy --all-targets --workspace -- -D warnings   # clean
./target/debug/file-manager-slint     # launched; still running after 8 s, no error output
```

Interactive checks: the agent cannot operate a display, IME or DPI settings.
Each item below must be filled in by a person. "Not executed" means it has not been run.

| Item | Action | Expected | Result |
|---|---|---|---|
| Chinese IME composition (EXP-005 precondition) | Type pinyin in the search box with a system Chinese IME | Composition text visible; candidate window near caret; committed text correct | Not executed |
| Empty result | Search `zzz` | List is empty; no crash | Not executed (logic covered by `empty_list_shows_empty_state`) |
| DPI / display scale | Switch system scale 100% -> 200% | Layout in dp unchanged; text sharp | Not executed |
| Window minimum size | Shrink window below 800 x 500 | Window stops at the minimum; no misalignment | Not executed |
| Long file name | Inspect the "Long file name ..." row | Text elided; layout intact | Not executed |
| Language switch | Click the Language button | Labels change to zh-CN without restart | Not executed |
| Theme switch | Click the Dark/Light button | Background changes; layout unchanged | Not executed |

Notes:

- Language and theme switching are wired in code but have not been confirmed visually.
- The IME, DPI and window-minimum checks determine whether EXP-005 and the G2 gate can pass later. They must be completed by a person before G0 review.
