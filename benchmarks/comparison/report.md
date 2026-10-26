# Comparison report: Aster (Slint adapter) vs Tauri, file manager

- Date: 2026-10-26
- Platform: macOS 12.7.6, Intel x86_64, built-in Retina display (scale factor 2)
- Requirements: `benchmarks/comparison/requirements.md`
- Raw records: `benchmarks/results/aster/`, `benchmarks/results/tauri/`, `benchmarks/results/*.json`
- Status: **partial.** Functional, geometry and size results are measured. Startup and idle memory
  were measured under system load and are provisional. Manual checks were skipped by decision
  (see section 6).

## 1. What was compared

Both implementations show the same window: sidebar, toolbar, search field, 24-row file list
generated from the same mock data (`benchmarks/comparison/generate-data.py`), and a detail panel.
Both read the same locale strings. Neither implements the file dialog (F12), which is recorded as
not implemented in both.

| | Aster spike | Tauri app |
|---|---|---|
| UI description | Aster model (`aster-ui-model`, `file-manager-aster`) | HTML, CSS and vanilla JavaScript |
| Rendering | Slint 1.18.1, via the Slint interpreter; `.slint` source generated at start-up | System WebView (WKWebView) |
| Business logic | Rust (`file-manager-core`) | JavaScript (in the page) |
| Executable | `file-manager-aster` (release) | `tauri-file-manager` (release) |

Aster is a spike; Tauri is a minimal page with no framework. Neither is a product.

## 2. Functional results

Automated checks only. A human did not look at the screen (see section 6).

| Req | Check | Aster | Tauri |
|---|---|---|---|
| F1 | Window content 1100 x 720 | yes (set in test) | yes (frame 748 incl. 28 pt title bar) |
| F1 | Minimum content 800 x 500 | yes, no layout diagnostics | yes, all elements inside the window |
| F2 | Sidebar 240 px, labels at top | yes (real layout test) | yes (x=0, width 240, labels at y=16) |
| F3 | Toolbar height < 60 px | yes (32 px) | yes (32 px) |
| F4 | Search filters the list | yes (callback test) | yes ("report" gives 2 rows; cleared gives 24) |
| F5 | List fills height, 24 rows | yes (600 px at 1100 x 720) | yes (600 px) |
| F6 | Empty result shows zero rows | yes (callback test) | yes (0 rows) |
| F7 | Selection shows the name | yes (callback test) | yes ("todo.txt") |
| F8 | Detail panel 280 px | yes | yes |
| F9 | Theme toggle changes background and label | yes (label test; background not checked) | yes (class toggled; label "Light") |
| F10 | Language toggle changes labels, restores | yes (callback test) | yes ("新建文件夹", restored) |
| F11 | Long name elided | yes in screenshot; not asserted | yes (`scrollWidth > clientWidth`) |

Interpretation: both implementations meet the automated functional requirements. The Aster test
for F9 checks the button label but not the background colour; that colour check is still open.

## 3. Geometry and layout

Both windows were checked with their own layout engines. Aster uses Slint's real layout through the
headless probe (`aster-check-layout`); Tauri uses `getBoundingClientRect` in the page.

- Before the Aster layout fix, the generated layout was wrong: toolbar buttons, the search field and
  the sidebar labels stretched over the full window height, and the file list was almost hidden.
  The earlier "matches the model" conclusion only checked widths and was not valid. The fix
  (stretch and alignment rules in `aster-backend-slint`) is covered by real-layout tests, and the
  before and after screenshots are in `benchmarks/results/evidence/`.
- Tauri's layout was correct from the first build; it is plain CSS flex.

## 4. Experiment 03: sidebar 240 -> 280 px

| | Aster | Tauri |
|---|---|---|
| Edits | 1 structured patch (`patches/exp03-sidebar-280.json`, with `expect: 240dp`) | 1 line in `styles.css` |
| Rebuild needed | No. The window is generated at start-up | Yes. Rebuild took about 151 s (macOS, load ~20) |
| Verification | Real layout: sidebar 280, content x 280 | Real DOM: sidebar 280, content x 280, width 820 |
| Revert | Not needed (patch is applied per run) | Done; second rebuild took about 174 s |
| Evidence | Patch screenshots in `benchmarks/results/evidence/` | `benchmarks/results/evidence/2026-10-26-exp03-tauri-sidebar-280.png` |

The difference is architectural: Aster changes the model and regenerates the UI at start-up;
Tauri embeds the assets and needs a Rust build for each change. The timing of the Aster edit cycle
was not measured separately.

## 5. Size

Measured with `benchmarks/scripts/measure-size.sh` (release builds). These values do not depend on
system load, but they describe different things (see the notes).

| Artefact | Aster | Tauri |
|---|---|---|
| Executable, bytes | 11,314,432 (file manager) / 7,120,304 (B1 minimal) | 3,983,704 |
| Executable, gzip -9 bytes | 5,913,077 / 4,149,915 | 1,293,730 |
| Distribution bundle | not produced yet | `.app`, 3,896 KiB (3.8 MiB) |

Notes:

- Tauri's executable does not include the WebView runtime, which macOS provides. Its bundle includes
  the page assets. Aster's executable includes the Slint runtime and the interpreter, which is why
  the Aster figure is larger.
- The Aster figure is for the executable only. A bundle comparison is still missing.
- The 7.1 MB B1 figure is a counter application, not comparable with the file manager.
- Size alone does not support any conclusion about the architectures.

## 6. Startup and memory

**Startup:** the only complete set is from 2026-10-26, taken while the system load average was
between 26 and 33 on a 4-thread machine with memory exhausted. The values below are **provisional**
and must not be used for a comparison. The same script (`benchmarks/scripts/measure-startup.py`,
10 runs each) must be run again on an idle system.

| App (load ~30) | process_to_main (median) | window_create (median) | first_frame (median) | total (median, range) |
|---|---|---|---|---|
| Aster B1 | 86.8 | 204.4 | 461.5 | 794.0 (546 to 1024) |
| Aster file manager | 47.9 | 702.9 | 986.5 | 1805.3 (1370 to 2441) |
| Aster file manager, second run | 137.0 | 561.0 | 557.7 | 1321.9 (931 to 1588) |
| Tauri | 157.5 | 407.5 | 593.3 | 1157.3 (895 to 2420) |

On this data, the Aster file manager's window-creation cost (about 0.4 to 0.7 s) is dominated by
generating and compiling the Slint source at start-up. The earlier 413 ms figure was measured under
lower load. The variance between runs is larger than the difference between some of these values.

**Idle memory:** not measured. `benchmarks/scripts/measure-memory.py` is ready. Tauri's memory must
be measured as the process tree, because WebKit uses helper processes.

## 7. Limitations

1. Startup and memory are not valid for comparison yet (section 6).
2. Manual checks were skipped. IME, display scaling and visual quality are unverified for both
   implementations (see section 8).
3. F9 background colour is not checked automatically in Aster.
4. Single platform (macOS 12.7.6, x86_64). No ARM64, Windows or Linux data.
5. Tauri's page is not framework-based; a framework-based app would have different costs.
6. The Aster geometry probe is a verification tool built on Slint's testing backend. It is not
   part of the application binary, and it checks widths, heights and overflow only.
7. The file manager dialog (F12) was not implemented in either version.
8. The comparison measures one application. It does not show that either architecture is better
   for other applications.

## 8. Manual checks (skipped by decision)

The machine used for this work has no interactive display for a tester. The following checks were
not executed. The owner decided on 2026-10-26 to move them to gate G2 (EXP-005):

- Chinese IME composition and candidate window position
- System display scaling 100% -> 200%
- Visual review of layout, theme and language switching

They are not marked as passed in any record.

## 9. Conclusion

Both implementations meet the automated functional and geometry requirements for this
application at the sizes tested. The main measured difference so far is in the work needed for a
layout change: Aster regenerates its UI at start-up, while Tauri needs a rebuild. Start-up time and
memory remain open. No claim of advantage for either side is supported by the data today.
