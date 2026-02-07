//! Command-line argument parsing for the dither CLI.

use crate::dither::DitherMethod;
use crate::palette::ColorPalette;
use clap::Parser;
use std::path::PathBuf;

/// Command-line arguments for the dithers CLI tool.
///
/// A simple command-line tool for dithering images with various algorithms and color palettes.
#[derive(Parser, Debug)]
#[clap(author, version, about)]
pub struct Args {
  /// Input image file path
  #[clap(short, long = "in")]
  pub in_img: PathBuf,

  /// Output image file path (optional)
  #[clap(short, long = "out", default_value = "out.png")]
  pub out_img: Option<PathBuf>,

  /// Dithering algorithm to use
  #[clap(short, long = "dither", default_value_t, value_enum)]
  pub dither_type: DitherMethod,

  /// Color palette for quantization
  #[clap(short, long = "color", default_value_t, value_enum)]
  pub color_palette: ColorPalette,
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_args_default_values() {
    // Test that default values work as expected when parsing minimal args
    let args = Args::try_parse_from(&["dithers", "-i", "test.jpg"]).unwrap();

    assert_eq!(args.in_img, PathBuf::from("test.jpg"));
    assert_eq!(args.out_img, Some(PathBuf::from("out.png")));
    assert_eq!(args.dither_type, DitherMethod::FloydSteinberg);
    assert_eq!(args.color_palette, ColorPalette::Monochrome);
  }

  #[test]
  fn test_args_full_specification() {
    let args = Args::try_parse_from(&["dithers", "-i", "input.png", "-o", "output.jpg", "-d", "atkinson", "-c", "color16"]).unwrap();

    assert_eq!(args.in_img, PathBuf::from("input.png"));
    assert_eq!(args.out_img, Some(PathBuf::from("output.jpg")));
    assert_eq!(args.dither_type, DitherMethod::Atkinson);
    assert_eq!(args.color_palette, ColorPalette::COLOR16);
  }

  #[test]
  fn test_args_missing_input_fails() {
    let result = Args::try_parse_from(&["dithers"]);
    assert!(result.is_err(), "Should fail when input file is not specified");
  }

  #[test]
  fn test_args_help_works() {
    let result = Args::try_parse_from(&["dithers", "--help"]);
    assert!(result.is_err()); // clap returns Err for --help, but its a special case
  }

  #[test]
  fn test_all_dither_methods_parseable() {
    let methods = [
      "none",
      "floyd-steinberg",
      "simple2-d",
      "jarvis",
      "atkinson",
      "stucki",
      "burkes",
      "sierra",
      "two-row-sierra",
      "sierra-lite",
      "bayer2x2",
      "bayer4x4",
      "bayer8x8",
    ];

    for method in methods {
      let args = Args::try_parse_from(&["dithers", "-i", "test.jpg", "-d", method]);
      assert!(args.is_ok(), "Should be able to parse dither method: {}", method);
    }
  }

  #[test]
  fn test_all_color_palettes_parseable() {
    let palettes = ["monochrome", "color8", "color16"];

    for palette in palettes {
      let args = Args::try_parse_from(&["dithers", "-i", "test.jpg", "-c", palette]);
      assert!(args.is_ok(), "Should be able to parse color palette: {}", palette);
    }
  }
}
