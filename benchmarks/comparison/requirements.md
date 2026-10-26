# Comparison requirements: file manager (Aster vs Tauri)

This document defines the functionality and measurement conditions that every implementation in
the comparison must satisfy. An implementation that does not meet a requirement must be reported
as "not implemented", never silently skipped (05 section 2.3).

## 1. Functional requirements

| ID | Requirement | Acceptance |
|---|---|---|
| F1 | Window: title "File Manager", preferred 1100 x 720 logical px, minimum 800 x 500 | Window size and minimum verified at runtime |
| F2 | Sidebar: fixed 240 px wide, with three labels (All files, Recent, Favorites), top-aligned | Sidebar width 240 px; labels start at the top |
| F3 | Toolbar: four buttons in one row (New folder, Refresh, Theme toggle, Language toggle), natural button height | Toolbar height below 60 px |
| F4 | Search field: single line, fills the content width, placeholder "Search files" | Field height below 60 px; filter updates the list on every keystroke |
| F5 | File list: 24 rows from the same mock data (name, kind, modified date), fills the remaining content height, scrollable | List height above 300 px at 1100 x 720; all rows reachable by scrolling |
| F6 | Empty result: a query with no matches shows zero rows and does not fail | Zero rows, no error |
| F7 | Selection: clicking a row shows its name in the detail panel; initial text "No selection" | Detail text equals the selected name |
| F8 | Detail panel: fixed 280 px wide, on the right of the list | Panel width 280 px |
| F9 | Theme toggle: switches light and dark background; the button label names the theme it switches to | Background colour changes; label changes |
| F10 | Language toggle: switches all visible labels between en and zh-CN without restart | All labels change; second toggle restores English |
| F11 | Long file names are elided with an ellipsis and do not break the layout | Visible "..." on the long row; no overflow |
| F12 | File selection dialog: NOT required in this comparison. Both implementations omit it; the omission is recorded | Recorded as not implemented in both |

Both implementations use the same mock data (`spikes/file-manager-core/src/mock.rs`) and the same
locale strings (`locales/en.toml`, `locales/zh-CN.toml`). Neither may add features beyond this list.

## 2. Layout and display conditions

- Logical window size 1100 x 720, scale factor 1.0 (display set to default scaling), light theme.
- Each implementation is measured at the same window size. Screenshots are taken with
  `benchmarks/scripts/capture-app-window.sh`.
- Geometry checks use element rectangles reported by the implementation's own layout engine
  (Aster: headless probe; Tauri: `getBoundingClientRect` through the `ASTER_COMPARE_GEOMETRY` mode).

## 3. Measurement conditions

| Metric | Method | Notes |
|---|---|---|
| Executable size | `benchmarks/scripts/measure-size.sh` on the release executable | Both implementations |
| Full distribution size | `.app` bundle size (`du -sk`) and a DMG if produced | Tauri produces a bundle; Aster does not yet, so the value is recorded as missing |
| Cold start | `benchmarks/scripts/measure-startup.py` (three segments plus total), 10 runs | Same caliber as `benchmarks/results/SCHEMA.md` |
| First frame | Tauri reports the first painted frame through the same marks (see `SCHEMA.md`) | Aster reports `FIRST_FRAME_US` from the rendering notifier |
| Idle memory | `benchmarks/scripts/measure-memory.py`: RSS sampled 30 s after the window is ready, for the process and for its descendant processes | Tauri uses WKWebView helper processes; both views are reported |
| AI modification | Same task for both: change the sidebar width from 240 to 280 px; acceptance = geometry check passes | Count edits and failed verification attempts |

## 4. Known differences that must be reported, not hidden

- Tauri renders with the system WebView; Aster renders with Slint. Their processes differ.
- Aster generates Slint source at start-up (interpreter). Tauri loads HTML/CSS/JS from the bundle.
- Executable size is not comparable across architectures alone. Report the full bundle too.
- The Tauri app is a minimal implementation; no framework (React, Svelte) is used, to avoid adding
  framework cost to the comparison.
