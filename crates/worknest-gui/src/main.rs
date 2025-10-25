//! Worknest GUI Application
//!
//! Desktop application for Worknest built with egui.

use eframe::egui;

fn main() -> Result<(), eframe::Error> {
    // Setup logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    tracing::info!("Starting Worknest");

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1280.0, 720.0])
            .with_min_inner_size([800.0, 600.0])
            .with_icon(load_icon()),
        ..Default::default()
    };

    eframe::run_native(
        "Worknest",
        options,
        Box::new(|cc| Box::new(WorknestApp::new(cc))),
    )
}

/// Load application icon
fn load_icon() -> egui::IconData {
    // Placeholder: will add actual icon later
    egui::IconData {
        rgba: vec![255; 32 * 32 * 4], // White 32x32 icon
        width: 32,
        height: 32,
    }
}

/// Main application structure
struct WorknestApp {
    _creation_context: Option<()>,
}

impl WorknestApp {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self {
            _creation_context: None,
        }
    }
}

impl eframe::App for WorknestApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Worknest");
            ui.separator();

            ui.vertical_centered(|ui| {
                ui.add_space(100.0);
                ui.heading("Welcome to Worknest");
                ui.add_space(20.0);
                ui.label("A modern project and task manager for developers");
                ui.add_space(40.0);
                ui.label("🚧 Under Development 🚧");
                ui.add_space(20.0);
                ui.label("This is the initial setup. Features coming soon!");
            });
        });
    }
}
