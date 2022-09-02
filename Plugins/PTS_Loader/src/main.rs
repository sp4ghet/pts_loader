use std::time::Instant;

use anyhow::Result;
use pts_loader;

fn main() -> Result<()> {
    let start = Instant::now();
    let points = pts_loader::load::from_file("tests/large/lineCube0119-002.pts");
    let done = Instant::now();
    let dur = done.duration_since(start);
    match points {
        Ok(points) => {
            println!(
                "Loaded {} points in {} ms, {} pt/ms",
                points.len(),
                dur.as_millis(),
                points.len() as f32 / dur.as_millis() as f32
            );
            Ok(())
        }
        Err(e) => {
            println!("Exited in {} ms", dur.as_millis());
            Err(e)
        }
    }
}
