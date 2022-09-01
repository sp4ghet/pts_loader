use anyhow::{Context, Result};
use num_traits::Num;
use pbr::ProgressBar;
use rayon::prelude::*;
use std::fs;
use std::io::Read;
use std::sync::{Arc, Mutex};
use std::{fmt::Display, str::FromStr};

use lazy_static::lazy_static;

use crate::{PtsPoint, Vec3};

#[derive(Debug, Eq, PartialEq)]
pub enum LoadError {
    InvalidInputError(String),
}

impl std::error::Error for LoadError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }

    fn cause(&self) -> Option<&dyn std::error::Error> {
        self.source()
    }
}

impl Display for LoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LoadError::InvalidInputError(s) => f.write_fmt(format_args!("{}", s)),
        }
    }
}

fn parse_str<N>(s: &str) -> Result<N, <N as FromStr>::Err>
where
    N: Num,
    N: FromStr,
{
    s.parse::<N>()
}

pub fn from_file(path: &str) -> Result<Vec<PtsPoint>> {
    let mut file = fs::File::open(path).with_context(|| format!("Failed to load file {}", path))?;
    let mut contents = String::with_capacity(file.metadata()?.len() as _);
    file.read_to_string(&mut contents)?;
    let mut lines = contents.lines();

    let count_str = lines.next().ok_or(LoadError::InvalidInputError(format!(
        "No data in file {}",
        path
    )))?;

    let count = parse_str::<usize>(&count_str.trim()).with_context(|| {
        format!(
            "Couldn't find row count in first line of {}, got {}",
            path, count_str
        )
    })?;

    let i = std::sync::atomic::AtomicUsize::new(0);
    let pb = Arc::new(Mutex::new(ProgressBar::new(count as _)));

    lazy_static! {
        static ref SPACE: char = ' ';
        static ref X_ERR: LoadError = LoadError::InvalidInputError(format!("No x coordinate"));
        static ref Y_ERR: LoadError = LoadError::InvalidInputError(format!("No y coordinate"));
        static ref Z_ERR: LoadError = LoadError::InvalidInputError(format!("No z coordinate"));
        static ref I_ERR: LoadError = LoadError::InvalidInputError(format!("No intensity value"));
        static ref R_ERR: LoadError = LoadError::InvalidInputError(format!("No R color value"));
        static ref G_ERR: LoadError = LoadError::InvalidInputError(format!("No G color value"));
        static ref B_ERR: LoadError = LoadError::InvalidInputError(format!("No B color value"));
    }

    let points: Result<Vec<PtsPoint>> = lines
        .par_bridge()
        .map(|line| {
            let mut tokens = line.split(*SPACE);

            let x = parse_str::<f32>(tokens.next().ok_or(&*X_ERR)?)?;
            let y = parse_str::<f32>(tokens.next().ok_or(&*Y_ERR)?)?;
            let z = parse_str::<f32>(tokens.next().ok_or(&*Z_ERR)?)?;

            let intensity = parse_str::<i32>(tokens.next().ok_or(&*I_ERR)?)?;
            let r = parse_str::<u8>(tokens.next().ok_or(&*R_ERR)?)?;
            let g = parse_str::<u8>(tokens.next().ok_or(&*G_ERR)?)?;
            let b = parse_str::<u8>(tokens.next().ok_or(&*B_ERR)?)?;

            let point = PtsPoint {
                point: Vec3::<f32> { x, y, z },
                intensity,
                rgb: Vec3::<u8> { x: r, y: g, z: b },
            };

            let prev_count = i.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            if prev_count % 500_000 == 0 {
                let mut pb = pb.lock().unwrap();
                pb.set(prev_count as _);
            }
            Ok(point)
        })
        .collect();
    pb.lock().unwrap().finish();

    points
}
