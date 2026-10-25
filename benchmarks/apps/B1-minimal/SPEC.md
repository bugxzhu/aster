# B1 Minimal application: functional specification

Purpose: measure the fixed runtime cost (EXP-001). See `docs/规范/05-基准应用与性能测试计划.md` section 2.

Required features (and nothing else, 05 section 2.3):

1. One window with the title "B1 Minimal".
2. One heading text ("Counter").
3. One text showing the current count ("Count: N"), starting at 0.
4. One button ("Increment") that adds 1 to the count when clicked.

Build profile: release profile from the workspace root (`opt-level = "z"`, LTO, `codegen-units = 1`, `strip = true`, `panic = "abort"`). Record it with every result, and report against competitors' default configurations separately (05 section 4.4).

Cold-start instrumentation: prints `FIRST_FRAME_MS=<n>` to standard error after the first rendered frame. This output is for measurement only.
