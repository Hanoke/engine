// Note: do not include mods here, just use. And do not include lib.rs here.

use hanokei_lib::engine::Engine;

fn main() {
    let engine = Engine::new();
    engine.loop_start();
}
