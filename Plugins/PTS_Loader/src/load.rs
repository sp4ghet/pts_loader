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

#[derive(Debug, Eq, PartialEq, Clone)]
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

// error messages and splitting chars for use in main loop
// negligible performance improvement but still feels like best practice.
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

pub fn from_file(path: &str) -> Result<Vec<PtsPoint>> {
    let file = fs::File::open(path).with_context(|| format!("Failed to load file {}", path))?;
    // Set the buffer size to roughly 80MB, worked best on my machine™
    // Default is 8KB, which was way too small for this workload.
    let buffer_cap = (2_usize.pow(23)).min(file.metadata()?.len() as _);
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

    // progress bar stuff
    let pb = Arc::new(Mutex::new(ProgressBar::new(count as _)));
    let pb_counter = std::sync::atomic::AtomicUsize::new(0);

    // preallocate final output
    let mut points = Vec::<PtsPoint>::with_capacity(count);
    let err_opt: Mutex<Option<anyhow::Error>> = Mutex::new(None);

    // preallocate input batch vector
    // assume roughly 40 chars per line
    // better to be conservative here
    let batch_size = buffer_cap / 15;
    let mut batch = vec![String::new(); batch_size];
    let mut i = 0;
    while i < count {
        // drain the buffer into a batch
        // important to not alloc here, roughly 2x speed improvement
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
        let points_batch = batch.par_iter().take(k).map(|line| {
            let f = |line: &String| -> Result<PtsPoint> {
                // trim needed since newline is kept from `read_line`
                let mut tokens = line.trim().split(*SPACE);

                // pts format is x,y,z,intensity,r,g,b : (f32, f32, f32, i32, u8, u8, u8)
                // use iterator without `collect` to avoid alloc
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

                // progressbar things
                let idx = pb_counter.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                if idx % 500_000 == 0 {
                    pb.lock().unwrap().set(idx as _);
                }

                Ok(point)
            };

            let default = PtsPoint {
                point: Vec3::<f32> {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                },
                intensity: 0,
                rgb: Vec3::<u8> { x: 0, y: 0, z: 0 },
            };

            match f(line) {
                Ok(x) => x,
                Err(e) => {
                    let mut emut = err_opt.lock().unwrap();
                    emut.replace(e);
                    default
                }
            }
        });

        points.par_extend(points_batch);
        // iterator isn't consumed until this point
        // need to check errors here
        if err_opt.lock().unwrap().is_some() {
            let e = err_opt.into_inner()?.unwrap();
            return Err(e);
        }
    }

    pb.lock().unwrap().finish();
    Ok(points)
}
