# Aster

Aster is a native desktop development language for the era of AI-driven programming. It is an AI-native, native desktop application language and toolchain (project status: design and P0 validation).

## Layout

- `docs/` - design documents. Internal design records are written in Chinese; release documents will be bilingual.
- `spikes/` - throwaway validation prototypes (not depended on by `crates/`)
- `benchmarks/` - benchmark applications, scripts and results
- `xtask/` - repository policy checks
- `locales/` - localization data files

## Commands

```
cargo test --workspace
cargo run -p xtask -- check-english
```

## License

MIT. See `LICENSE`. Third-party dependency review and Slint distribution checks are still pending (see `docs/adr/015-license.md`).
