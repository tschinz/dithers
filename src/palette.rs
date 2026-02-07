//! Color palette definitions and utilities.

use crate::dither::QuantizationError;

/// Available color palettes for dithering.
#[derive(clap::ValueEnum, Copy, Clone, Debug, Default, PartialEq)]
pub enum ColorPalette {
  /// Black and white palette (2 colors)
  #[default]
  Monochrome,
  /// 8-color palette
  COLOR8,
  /// 16-color palette
  COLOR16,
}

/// Represents an RGB color.
pub struct Color {
  /// Red component (0-255)
  pub r: u8,
  /// Green component (0-255)
  pub g: u8,
  /// Blue component (0-255)
  pub b: u8,
}

impl From<u32> for Color {
  fn from(v: u32) -> Self {
    Color {
      r: ((v >> 16) & 0xFF) as u8,
      g: ((v >> 8) & 0xFF) as u8,
      b: (v & 0xFF) as u8,
    }
  }
}

impl From<&[u8]> for Color {
  fn from(v: &[u8]) -> Self {
    Color { r: v[0], g: v[1], b: v[2] }
  }
}

/// Maps a color to the closest color in the given palette.
///
/// Returns the closest palette color and the quantization error.
pub fn map_to_palette(orig_color: Color, palette: &[Color]) -> (&Color, QuantizationError) {
  // simple stupid linear search
  // this can be optimized with a better algorithm
  let mut min_distance = f32::INFINITY;
  let mut color = &palette[0];
  for c in palette {
    let distance =
      // sqrt not needed since we only compare distances, not actual values
      //((orig_color.r as f32 - c.r as f32).powi(2) + (orig_color.g as f32 - c.g as f32).powi(2) + (orig_color.b as f32 - c.b as f32).powi(2)).sqrt();
      (orig_color.r as f32 - c.r as f32).powi(2) + (orig_color.g as f32 - c.g as f32).powi(2) + (orig_color.b as f32 - c.b as f32).powi(2);
    if distance < min_distance {
      color = c;
      min_distance = distance;
    }
  }
  let qe = QuantizationError {
    r: orig_color.r as f32 - color.r as f32,
    g: orig_color.g as f32 - color.g as f32,
    b: orig_color.b as f32 - color.b as f32,
  };

  (color, qe)
}

/// 16-color palette with a diverse range of colors.
pub const PALETTE_16C: [Color; 16] = [
  //Color::from(0x000000), // does not work since its a const
  Color { r: 0x00, g: 0x00, b: 0x00 },
  Color { r: 0x9d, g: 0x9d, b: 0x9d },
  Color { r: 0xff, g: 0xff, b: 0xff },
  Color { r: 0xbe, g: 0x26, b: 0x33 },
  Color { r: 0xe0, g: 0x6f, b: 0x8b },
  Color { r: 0x49, g: 0x3c, b: 0x2b },
  Color { r: 0xa4, g: 0x64, b: 0x22 },
  Color { r: 0xeb, g: 0x89, b: 0x31 },
  Color { r: 0xf7, g: 0xe2, b: 0x6b },
  Color { r: 0x2f, g: 0x48, b: 0x4e },
  Color { r: 0x44, g: 0x89, b: 0x1a },
  Color { r: 0xa3, g: 0xce, b: 0x27 },
  Color { r: 0x1b, g: 0x26, b: 0x32 },
  Color { r: 0x00, g: 0x57, b: 0x84 },
  Color { r: 0x31, g: 0xa2, b: 0xf2 },
  Color { r: 0xb2, g: 0xdc, b: 0xef },
];

/// 8-color palette with primary colors.
pub const PALETTE_8C: [Color; 8] = [
  Color { r: 0x00, g: 0x00, b: 0x00 },
  Color { r: 0xcc, g: 0x35, b: 0x00 },
  Color { r: 0x5e, g: 0xc8, b: 0x09 },
  Color { r: 0x1d, g: 0x28, b: 0x6f },
  Color { r: 0x00, g: 0xc4, b: 0xff },
  Color { r: 0x8e, g: 0x8e, b: 0x8e },
  Color { r: 0xff, g: 0xe0, b: 0x52 },
  Color { r: 0xff, g: 0xff, b: 0xff },
];

pub const PALETTE_MONOCHROME: [Color; 2] = [Color { r: 0x00, g: 0x00, b: 0x00 }, Color { r: 0xff, g: 0xff, b: 0xff }];

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_color_palette_default() {
    assert_eq!(ColorPalette::default(), ColorPalette::Monochrome);
  }

  #[test]
  fn test_color_from_u32() {
    let color = Color::from(0xFF8000u32); // Orange
    assert_eq!(color.r, 255);
    assert_eq!(color.g, 128);
    assert_eq!(color.b, 0);
  }

  #[test]
  fn test_color_from_slice() {
    let data = [100, 150, 200];
    let color = Color::from(&data[..]);
    assert_eq!(color.r, 100);
    assert_eq!(color.g, 150);
    assert_eq!(color.b, 200);
  }

  #[test]
  fn test_palette_sizes() {
    assert_eq!(PALETTE_MONOCHROME.len(), 2);
    assert_eq!(PALETTE_8C.len(), 8);
    assert_eq!(PALETTE_16C.len(), 16);
  }

  #[test]
  fn test_monochrome_palette_colors() {
    assert_eq!(PALETTE_MONOCHROME[0].r, 0);
    assert_eq!(PALETTE_MONOCHROME[0].g, 0);
    assert_eq!(PALETTE_MONOCHROME[0].b, 0);
    assert_eq!(PALETTE_MONOCHROME[1].r, 255);
    assert_eq!(PALETTE_MONOCHROME[1].g, 255);
    assert_eq!(PALETTE_MONOCHROME[1].b, 255);
  }

  #[test]
  fn test_map_to_palette_exact_match() {
    let black = Color { r: 0, g: 0, b: 0 };
    let (closest, error) = map_to_palette(black, &PALETTE_MONOCHROME);

    assert_eq!(closest.r, 0);
    assert_eq!(closest.g, 0);
    assert_eq!(closest.b, 0);
    assert_eq!(error.r, 0.0);
    assert_eq!(error.g, 0.0);
    assert_eq!(error.b, 0.0);
  }

  #[test]
  fn test_map_to_palette_gray_to_monochrome() {
    let gray = Color { r: 128, g: 128, b: 128 };
    let (closest, _error) = map_to_palette(gray, &PALETTE_MONOCHROME);

    // Should map to either black or white
    assert!((closest.r == 0 && closest.g == 0 && closest.b == 0) || (closest.r == 255 && closest.g == 255 && closest.b == 255));
  }

  #[test]
  fn test_map_to_palette_quantization_error() {
    let gray = Color { r: 100, g: 100, b: 100 };
    let (_closest, error) = map_to_palette(gray, &PALETTE_MONOCHROME);

    // Error should be the difference between original and quantized
    assert!(error.r != 0.0 || error.g != 0.0 || error.b != 0.0);
  }

  #[test]
  fn test_8color_palette_has_black_and_white() {
    let has_black = PALETTE_8C.iter().any(|c| c.r == 0 && c.g == 0 && c.b == 0);
    let has_white = PALETTE_8C.iter().any(|c| c.r == 255 && c.g == 255 && c.b == 255);

    assert!(has_black, "8-color palette should contain black");
    assert!(has_white, "8-color palette should contain white");
  }

  #[test]
  fn test_16color_palette_has_black_and_white() {
    let has_black = PALETTE_16C.iter().any(|c| c.r == 0 && c.g == 0 && c.b == 0);
    let has_white = PALETTE_16C.iter().any(|c| c.r == 255 && c.g == 255 && c.b == 255);

    assert!(has_black, "16-color palette should contain black");
    assert!(has_white, "16-color palette should contain white");
  }

  #[test]
  fn test_map_to_palette_finds_closest() {
    // Test with a color that should map to a specific color in 8-color palette
    let red_ish = Color { r: 200, g: 30, b: 10 };
    let (closest, _) = map_to_palette(red_ish, &PALETTE_8C);

    // Should map to the red color in the palette
    assert_eq!(closest.r, 0xcc);
    assert_eq!(closest.g, 0x35);
    assert_eq!(closest.b, 0x00);
  }
}
