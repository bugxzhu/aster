use std::time::{SystemTime, UNIX_EPOCH};

slint::include_modules!();

/// Wall-clock microseconds since the Unix epoch, shared clock with the timing script.
fn epoch_us() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_micros())
        .unwrap_or(0)
}

fn main() -> Result<(), slint::PlatformError> {
    // Timing output is produced only when ASTER_TIMING is set (see benchmarks/results/SCHEMA.md).
    let timing = std::env::var_os("ASTER_TIMING").is_some();
    if timing {
        eprintln!("MAIN_START_US={}", epoch_us());
    }

    let ui = MainWindow::new()?;
    if timing {
        eprintln!("WINDOW_READY_US={}", epoch_us());
        let mut reported = false;
        let weak_ui = ui.as_weak();
        let notifier = ui.window().set_rendering_notifier(move |state, _| {
            if !reported && matches!(state, slint::RenderingState::AfterRendering) {
                reported = true;
                // Read the scale factor here, after the window is shown. Before that it can report 1.0.
                if let Some(ui) = weak_ui.upgrade() {
                    eprintln!("SCALE_FACTOR={}", ui.window().scale_factor());
                }
                eprintln!("FIRST_FRAME_US={}", epoch_us());
            }
        });
        if let Err(e) = notifier {
            eprintln!("rendering notifier unavailable: {e}");
        }
    }

    let weak = ui.as_weak();
    ui.on_increment(move || {
        if let Some(ui) = weak.upgrade() {
            ui.set_count(ui.get_count() + 1);
        }
    });

    ui.run()
}
