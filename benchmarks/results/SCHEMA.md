# Benchmark result schema

Each measurement is stored as one JSON file: `benchmarks/results/<YYYY-MM-DD>-<platform>-<implementation>.json`.

## Required fields

| Field | Meaning |
|---|---|
| `date` | Measurement date (ISO 8601) |
| `application` | Benchmark app id (`B1`, `B2`, `B3`) |
| `implementation` | Framework and version (for example `slint-1.18.1`) |
| `build` | Profile, optimisation settings, toolchain version |
| `environment` | CPU, RAM, OS version and build, display resolution and scale, power state |
| `binary` | `binary_bytes`, `gzip_bytes`, `sha256` (from `measure-size.sh`) |
| `cold_start_ms` | Object with `runs`, `min`, `median`, `max` |
| `notes` | Known limitations and deviations |

## Cold-start definition used in this project

- Metric: time from the start of `main()` to the first frame rendered, as reported by `FIRST_FRAME_MS` on standard error.
- Limitation: process creation time before `main()` is not included. Competitor frameworks must be measured with the same definition before comparison.
- Repetitions: at least 10 runs; report median, minimum and maximum. Keep failed runs.

## Timing caliber

Applications report timing only when the environment variable `ASTER_TIMING` is set. They print epoch microseconds on standard error:

| Mark | Printed at |
|---|---|
| `MAIN_START_US` | first statement of `main()` |
| `WINDOW_READY_US` | after the window and UI are created, before the event loop starts |
| `FIRST_FRAME_US` | first `AfterRendering` notification of the window |
| `SCALE_FACTOR` | window scale factor (informational) |

`benchmarks/scripts/measure-startup.py` runs the binary, records the spawn time on the parent side, and reports four values in milliseconds:

| Value | Definition |
|---|---|
| `process_to_main_ms` | spawn -> `MAIN_START_US` (process creation and loader time before `main`) |
| `window_create_ms` | `MAIN_START_US` -> `WINDOW_READY_US` |
| `first_frame_ms` | `WINDOW_READY_US` -> `FIRST_FRAME_US` |
| `total_ms` | spawn -> `FIRST_FRAME_US` (what the user waits for) |

Rules:

- Always report the four values, never only the total.
- Report median, minimum and maximum over at least 10 runs. Keep outliers.
- Only compare numbers produced with this caliber against each other. A competitor baseline must use the same definitions, the same functionality, and the same script or an equivalent that records the same marks.
