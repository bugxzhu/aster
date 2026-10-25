use std::time::Instant;

slint::include_modules!();

fn main() -> Result<(), slint::PlatformError> {
    // Measurement start: entry into main. Process creation time is not captured here,
    // so the reported value is a lower bound of the SPEC.md cold-start definition.
    let start = Instant::now();

    let ui = MainWindow::new()?;

    // First frame: report once after the first rendered frame (see benchmarks/results/SCHEMA.md).
    let mut reported = false;
    let report = ui.window().set_rendering_notifier(move |state, _| {
        if !reported && matches!(state, slint::RenderingState::AfterRendering) {
            reported = true;
            eprintln!("FIRST_FRAME_MS={}", start.elapsed().as_millis());
        }
    });
    if let Err(e) = report {
        eprintln!("rendering notifier unavailable: {e}");
    }

    let weak = ui.as_weak();
    ui.on_increment(move || {
        if let Some(ui) = weak.upgrade() {
            ui.set_count(ui.get_count() + 1);
        }
    });

    ui.run()
}
