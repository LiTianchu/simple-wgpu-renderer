use wgpu_tutorial::runner::run;
#[cfg(target_arch = "wasm32")]
use wgpu_tutorial::runner::run_web;

fn main() {
    #[cfg(target_arch = "wasm32")]
    run_web().expect("Failed to run the application in web mode");

    #[cfg(not(target_arch = "wasm32"))]
    run().expect("Failed to run the application");
}
