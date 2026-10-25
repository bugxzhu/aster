# Experiment 05: backend isolation

Date: 2026-10-25
Question: can the rendering backend be replaced without changing the UI model or the business layer?

## Dependency structure (normal dependencies, from `cargo tree -e normal`)

| Crate | Role | Direct normal dependencies | Slint-family crates in normal graph |
|---|---|---|---|
| `aster-ui-model` | UI model, validation, patches | serde, serde_json | 0 |
| `file-manager-core` | business state, mock data, locale packs | serde, toml | 0 |
| `aster-backend-text` | second backend (text tree) | aster-ui-model | 0 |
| `aster-backend-slint` | Slint adapter (generates `.slint`, runs interpreter) | aster-ui-model, slint, slint-interpreter, pollster | 34 |
| `file-manager-aster` | application shell, chooses the backend | aster-ui-model, aster-backend-slint, aster-backend-text, file-manager-core, slint, serde_json | (shell only) |

Dependency direction: `file-manager-aster` -> backend adapters -> `aster-ui-model`. The model and the business core never depend on a backend.

## Automated check

`cargo run -p xtask -- check-layering` checks the resolved normal dependency graph, including transitive dependencies:

- `aster-ui-model` must not reach `slint*`, `aster-backend-*` or `file-manager-*`.
- `file-manager-core` must not reach `slint*`, `aster-backend-*`, `file-manager-aster` or `file-manager-slint`.
- `aster-backend-text` must not reach `slint*` or the application.
- `aster-backend-slint` must not reach the application.

Result: `layering check passed (4 rules)`.

The checker itself was tested. A temporary direct dependency from `aster-ui-model` to `slint` (`slint = "=1.18.1"`) produced:

```
LAYERING VIOLATION: aster-ui-model: UI model must not depend on any rendering backend or application (path: aster-ui-model -> slint)
```

and exit code 1. The change was reverted and the check passed again.

## Replacement demonstration

The same `UiSpec` and the same `AppState` are rendered by two backends. Only the application shell selects the backend:

```
file-manager-aster --backend slint   # opens the Slint window
file-manager-aster --backend text    # prints the model as a text tree
```

With `--patch spikes/file-manager-aster/patches/exp03-sidebar-280.json`, the text output changes the sidebar line from `width=240dp` to `width=280dp`, and nothing else changes in the text output (`aster-backend-text` test `text_backend_reflects_a_structured_patch_on_the_same_model`).

## Test evidence

- `cargo test -p file-manager-core`: business logic runs without any backend (search, selection, theme, locale switching).
- `cargo test -p aster-ui-model`: validation and patches run without any backend.
- `cargo test -p aster-backend-text`: second backend renders the same model.

## Limits of this evidence

- The text backend shows that the model is renderable without Slint. It is not a visual backend. A real replacement (for example Skia or Qt) is still hypothetical.
- The layout geometry shown on screen comes from Slint's layout engine through the adapter. `aster-ui-model` only has a minimum-width checker (`check_window`). Aster's own layout semantics (02 section 5) are not implemented independently. This is the main remaining coupling and it is recorded as a risk for P2 (R-03).
- The adapter supports only the widget and state vocabulary used by the file manager. Adding a widget means changing the model enum, the adapter, and the text backend.
- `file-manager-aster` depends on both backends. This is intentional for the spike; the layering rules do not restrict the application shell.
