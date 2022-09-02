use anyhow::{Context, Result};
use num_traits::Num;
use pbr::ProgressBar;
use rayon::prelude::*;
use std::fs;
use std::io::{BufRead, BufReader};
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
    let file = fs::File::open(path).with_context(|| format!("Failed to load file {}", path))?;
    let buffer_cap = (2_usize.pow(23)).min(file.metadata()?.len() as _); // ~80 MB max
    let mut file = BufReader::with_capacity(buffer_cap, file);

    let mut line = String::with_capacity(64);
    file.read_line(&mut line)
        .map_err(|_e| LoadError::InvalidInputError(format!("No data in file {}", path)))?;

    let count_str = line.trim();
    let count = parse_str::<usize>(count_str).with_context(|| {
        format!(
            "Couldn't find row count in first line of {}, got {}",
            path, count_str
        )
    })?;

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

    let pb = Arc::new(Mutex::new(ProgressBar::new(count as _)));
    let pb_counter = std::sync::atomic::AtomicUsize::new(0);

    let mut points = Vec::with_capacity(count);

    // assume roughly 40 chars per line
    // better to be conservative here
    let batch_size = buffer_cap / 15;
    let mut batch = vec![String::new(); batch_size];
    let mut i = 0;
    while i < count {
        // drain the buffer into a batch
        let mut j = 0;
        let mut k = 0;
        while j < buffer_cap {
            let line = &mut batch[k];
            line.clear();
            let res = file.read_line(line)?;
            j += res;
            if res == 0 || k + 1 == batch.len() {
                break;
            }
            i += 1;
            k += 1;
        }

        // parse the batch
        let points_batch: Result<Vec<PtsPoint>> = batch
            .par_iter()
            .take(k)
            .map(|line| {
                let mut tokens = line.trim().split(*SPACE);

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

                let idx = pb_counter.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                if idx % 500_000 == 0 {
                    pb.lock().unwrap().set(idx as _);
                }
                Ok(point)
            })
            .collect();
        let points_batch = points_batch?;
        points.par_extend(points_batch);
    }

    pb.lock().unwrap().finish();
    Ok(points)
}
