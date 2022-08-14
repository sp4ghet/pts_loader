use std::time::Instant;

use pts_loader;

fn main() {
    let start = Instant::now();
    let points = pts_loader::load::from_file("tests/fixtures/ShibuyaUnderground.pts");
    let done = Instant::now();
    let dur = done.duration_since(start);
    println!(
        "Loaded {} points in {} ms, {} pt/ms",
        points.len(),
        dur.as_millis(),
        points.len() as f32 / dur.as_millis() as f32
    );
}
