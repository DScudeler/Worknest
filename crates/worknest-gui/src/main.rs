//! Worknest GUI Application Binary
//!
//! **NOTE**: This binary is NOT used for the actual application.
//!
//! The real WASM entry point is in app.rs:
//! - Function: `pub fn start()` with `#[wasm_bindgen(start)]` attribute
//! - This gets called automatically when the WASM module loads in the browser
//!
//! This main.rs exists only to satisfy Cargo's requirement for binaries.

fn main() {
    eprintln!("╔════════════════════════════════════════════════════════════╗");
    eprintln!("║           Worknest GUI - Web Application Only             ║");
    eprintln!("╚════════════════════════════════════════════════════════════╝");
    eprintln!();
    eprintln!("This application runs as WebAssembly in a browser.");
    eprintln!("It cannot be executed as a native binary.");
    eprintln!();
    eprintln!("To run the application:");
    eprintln!("  trunk serve              # Development mode");
    eprintln!("  trunk serve --release    # Production mode");
    eprintln!();
    eprintln!("The WASM entry point is: src/app.rs::start()");
    std::process::exit(1);
}
