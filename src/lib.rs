//! # Dithers Library
//!
//! A Rust library for applying various dithering algorithms to images.
//!
//! This library provides:
//! - Multiple dithering algorithms (Floyd-Steinberg, Jarvis, Atkinson, etc.)
//! - Color palette support (Monochrome, 8-color, 16-color)
//! - Image processing utilities
//!
//! ## Example
//!
//! ```no_run
//! use dithers::dither::{open_image, dither, save_image, DitherMethod};
//! use dithers::palette::ColorPalette;
//! use std::path::PathBuf;
//!
//! let (mut buffer, width, height) = open_image(&PathBuf::from("input.png"));
//! dither(&mut buffer, DitherMethod::FloydSteinberg, ColorPalette::Monochrome, width, height);
//! save_image(buffer, PathBuf::from("output.png"), width, height);
//! ```

pub mod args;
pub mod dither;
pub mod palette;
