//! Worknest GUI Application Binary
//!
//! This is the binary entry point for Worknest web application.
//! The actual application logic is in the `app` module.

fn main() {
    // For wasm32, the app module handles initialization via #[wasm_bindgen(start)]
    // This main function won't actually be called in WASM builds
    #[cfg(target_arch = "wasm32")]
    {
        // The wasm_bindgen start function in app.rs will be called automatically
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        eprintln!("This application is designed to run as WebAssembly in a browser.");
        eprintln!("Please build with: ./build-webapp.sh release");
    }
}
