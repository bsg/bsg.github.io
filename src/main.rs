#![warn(clippy::all, rust_2018_idioms)]

#[cfg(target_arch = "wasm32")] // make rust-analyzer happy
fn main() {
    // Redirect `log` message to `console.log` and friends:
    eframe::WebLogger::init(log::LevelFilter::Debug).ok();

    let web_options = eframe::WebOptions::default();

    wasm_bindgen_futures::spawn_local(async {
        eframe::WebRunner::new()
            .start(
                "the_canvas_id", // hardcode it
                web_options,
                Box::new(|cc| Box::new(bsgapp::BsgApp::new(cc))),
            )
            .await
            .expect("failed to start eframe");
    });
}

// we build only to wasm
#[cfg(not(target_arch = "wasm32"))]
fn main() {}
