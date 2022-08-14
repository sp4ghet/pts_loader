use pts_loader::{self, Vec3};

#[test]
fn load_valid_file() {
    let points = pts_loader::load::from_file("tests/fixtures/hoge.pts");
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
    assert_eq!(expect.len(), points.len());
    assert_eq!(expect, points);
}
