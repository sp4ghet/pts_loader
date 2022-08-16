use anyhow::{Context, Result};
use num_traits::Num;
use std::fs;
use std::io::prelude::BufRead;
use std::io::BufReader;
use std::{fmt::Display, str::FromStr};

use crate::{PtsPoint, Vec3};

fn parse_str<N>(s: &str) -> Result<N, <N as FromStr>::Err>
where
    N: Num,
    N: FromStr,
{
    s.parse::<N>()
}

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

pub fn from_file(path: &str) -> Result<Vec<PtsPoint>> {
    let file = fs::File::open(path).with_context(|| format!("Failed to load file {}", path))?;
    let file = BufReader::new(file);
    let mut lines = file.lines();

    let count_str = lines.next().ok_or(LoadError::InvalidInputError(format!(
        "No data in file {}",
        path
    )))??;

    let count = parse_str::<usize>(&count_str)
        .with_context(|| format!("Couldn't find row count in first line of {}", path))?;
    let mut points = Vec::<PtsPoint>::with_capacity(count);

    for line in lines {
        let line = line?;
        let mut tokens = line.split(' ');

        let x = parse_str::<f32>(
            tokens
                .next()
                .ok_or(LoadError::InvalidInputError(format!("No x coordinate")))?,
        )?;
        let y = parse_str::<f32>(
            tokens
                .next()
                .ok_or(LoadError::InvalidInputError(format!("No y coordinate")))?,
        )?;
        let z = parse_str::<f32>(
            tokens
                .next()
                .ok_or(LoadError::InvalidInputError(format!("No z coordinate")))?,
        )?;

        let intensity = parse_str::<i32>(
            tokens
                .next()
                .ok_or(LoadError::InvalidInputError(format!("No intensity value")))?,
        )?;
        let r = parse_str::<u8>(
            tokens
                .next()
                .ok_or(LoadError::InvalidInputError(format!("No R color value")))?,
        )?;
        let g = parse_str::<u8>(
            tokens
                .next()
                .ok_or(LoadError::InvalidInputError(format!("No G color value")))?,
        )?;
        let b = parse_str::<u8>(
            tokens
                .next()
                .ok_or(LoadError::InvalidInputError(format!("No B color value")))?,
        )?;

        let point = PtsPoint {
            point: Vec3::<f32> { x, y, z },
            intensity,
            rgb: Vec3::<u8> { x: r, y: g, z: b },
        };

        points.push(point);
    }

    Ok(points)
}
