# Spike B experiments 02 and 04 (model level)

Date: 2026-10-25
Crate: `spikes/aster-ui-model`
Commands:

```
cargo test -p aster-ui-model --test experiment_02_layout
cargo test -p aster-ui-model --test experiment_04_repair
```

## Experiment 02: layout verification (model level)

| Test | Result |
|---|---|
| window sizes at or above minimum produce no conflict | pass |
| window below minimum reports overflow (700 dp -> overflow 36 dp) | pass |
| long translated label raises minimum and is detected | pass |
| check is deterministic for identical input | pass |

Scope and limits:

- Verifies the layout minimum-size rules in the model only. No real rendering, text measurement, or font fallback is involved.
- The determinism test does not prove DPI independence; it only checks repeated calls.
- Overflow, clipping and overlap detection beyond minimum-width conflicts are not implemented.

## Experiment 04: repair loop (simulated patch)

| Test | Result |
|---|---|
| broken layout reports structured `LAYOUT_CONFLICT` with a suggestion naming `content.min_width` | pass |
| suggested patch (`content.min_width` 560 -> 400) resolves the conflict without changing other nodes | pass |
| patch to the wrong node is rejected by the check | pass |

Scope and limits:

- The patch is applied by a test, not by an AI agent. This is not the EXP-004 AI repair loop, which belongs to P3.
- Success counts, repair attempts and timing are not measured here.
