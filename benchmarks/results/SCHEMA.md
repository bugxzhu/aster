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
