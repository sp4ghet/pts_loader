use pts_loader::{self, load::LoadError, Vec3};
use std::{panic::panic_any, str::FromStr};

#[test]
fn load_valid_file() {
    let points = pts_loader::load::from_file("tests/fixtures/valid.pts");
    let expect = vec![
        pts_loader::PtsPoint {
            point: Vec3::<f32> {
                x: -0.41025,
                y: -2.0806,
                z: 8.00981,
            },
            intensity: 55,
            rgb: Vec3::<u8> {
                x: 52,
                y: 44,
                z: 65,
            },
        };
        8
    ];
    let points = points.unwrap();
    assert_eq!(expect.len(), points.len());
    assert_eq!(expect, points);
}

#[test]
fn load_no_header_file() {
    let path = "tests/fixtures/invalid_no_header.pts";
    let points = pts_loader::load::from_file(path);
    match points {
        Ok(_) => panic!("Read invalid file as valid"),
        Err(e) => {
            assert_eq!(
                "Couldn't find row count in first line of tests/fixtures/invalid_no_header.pts, got -0.41025 -2.0806 8.00981 55 52 44 65",
                format!("{}", e)
            );
        }
    }
}

#[test]
fn load_invalid_row_file() {
    let path = "tests/fixtures/invalid_row.pts";
    let points = pts_loader::load::from_file(path);

    match points {
        Ok(_) => panic!("Read invalid file as valid"),
        Err(e) => {
            assert_eq!("No B color value", format!("{}", e));
        }
    }
}

#[test]
fn load_non_existent_file() {
    let path = "tests/fixtures/no_such.pts";
    let points = pts_loader::load::from_file(path);

    match points {
        Ok(_) => panic!("Read invalid file as valid"),
        Err(e) => {
            assert_eq!(
                "Failed to load file tests/fixtures/no_such.pts",
                format!("{}", e)
            );
        }
    }
}
