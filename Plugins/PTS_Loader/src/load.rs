use std::fs;
use std::io::prelude::BufRead;
use std::io::BufReader;

use crate::{PtsPoint, Vec3};

pub fn from_file(path: &str) -> Vec<PtsPoint> {
    let file = fs::File::open(path).expect(&format!("Couldn't open file: {}", path));
    let file = BufReader::new(file);
    let mut lines = file.lines();

    let count_str = lines.next().unwrap().unwrap();
    let count = count_str.parse::<usize>().unwrap();
    let mut points = Vec::<PtsPoint>::with_capacity(count);

    for line in lines {
        let line = line.unwrap();
        let mut tokens = line.split(' ');

        let x = tokens.next().unwrap().parse::<f32>().unwrap();
        let y = tokens.next().unwrap().parse::<f32>().unwrap();
        let z = tokens.next().unwrap().parse::<f32>().unwrap();

        let intensity = tokens.next().unwrap().parse::<i32>().unwrap();
        let r = tokens.next().unwrap().parse::<u8>().unwrap();
        let g = tokens.next().unwrap().parse::<u8>().unwrap();
        let b = tokens.next().unwrap().parse::<u8>().unwrap();

        let point = PtsPoint {
            point: Vec3::<f32> { x, y, z },
            intensity,
            rgb: Vec3::<u8> { x: r, y: g, z: b },
        };

        points.push(point);
    }

    points
}
