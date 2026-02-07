//! Image dithering algorithms and utilities.

use std::path::PathBuf;

use image::{ExtendedColorType, ImageReader};

use crate::palette::{Color, ColorPalette, PALETTE_8C, PALETTE_16C, PALETTE_MONOCHROME, map_to_palette};

/// Available dithering methods.
#[derive(clap::ValueEnum, Copy, Clone, Debug, Default, PartialEq)]
pub enum DitherMethod {
  None,
  #[default]
  FloydSteinberg,
  Simple2D,
  Jarvis,
  Atkinson,
  Stucki,
  Burkes,
  Sierra,
  TwoRowSierra,
  SierraLite,
  Bayer2x2,
  Bayer4x4,
  Bayer8x8,
}

pub struct QuantizationError {
  pub r: f32,
  pub g: f32,
  pub b: f32,
}

pub const FLOYD_STEINBERG: [f32; 6] = [0.0, 0.0, 7.0 / 16.0, 3.0 / 16.0, 5.0 / 16.0, 1.0 / 16.0];
pub const JARVIS: [f32; 15] = [
  0.0,
  0.0,
  0.0,
  7.0 / 48.0,
  5.0 / 48.0,
  3.0 / 48.0,
  5.0 / 48.0,
  7.0 / 48.0,
  5.0 / 48.0,
  3.0 / 48.0,
  1.0 / 48.0,
  3.0 / 48.0,
  5.0 / 48.0,
  3.0 / 48.0,
  1.0 / 48.0,
];
// Bayer(n)=( 4⋅Bayer(n−1)+0 4⋅Bayer(n−1)+2 )
//            4⋅Bayer(n−1)+3 4⋅Bayer(n−1)+1
// Bayer(0)
/// 2x2 Bayer matrix for ordered dithering
pub const BAYER2X2: [f32; 4] = [0.0, 2.0 / 4.0, 3.0 / 4.0, 1.0 / 4.0];
/// 4x4 Bayer(1) matrix for ordered dithering
pub const BAYER4X4: [f32; 16] = [
  0.0,
  8.0 / 16.0,
  2.0 / 16.0,
  10.0 / 16.0,
  12.0 / 16.0,
  4.0 / 16.0,
  14.0 / 16.0,
  6.0 / 16.0,
  3.0 / 16.0,
  11.0 / 16.0,
  1.0 / 16.0,
  9.0 / 16.0,
  15.0 / 16.0,
  7.0 / 16.0,
  13.0 / 16.0,
  5.0 / 16.0,
];
/// 8x8 Bayer(2) matrix for ordered dithering
pub const BAYER8X8: [f32; 64] = [
  0.0,
  32.0 / 64.0,
  8.0 / 64.0,
  40.0 / 64.0,
  2.0 / 64.0,
  34.0 / 64.0,
  10.0 / 64.0,
  42.0 / 64.0,
  48.0 / 64.0,
  16.0 / 64.0,
  56.0 / 64.0,
  24.0 / 64.0,
  50.0 / 64.0,
  18.0 / 64.0,
  58.0 / 64.0,
  26.0 / 64.0,
  12.0 / 64.0,
  44.0 / 64.0,
  4.0 / 64.0,
  36.0 / 64.0,
  14.0 / 64.0,
  46.0 / 64.0,
  6.0 / 64.0,
  38.0 / 64.0,
  60.0 / 64.0,
  28.0 / 64.0,
  52.0 / 64.0,
  20.0 / 64.0,
  62.0 / 64.0,
  30.0 / 64.0,
  54.0 / 64.0,
  22.0 / 64.0,
  3.0 / 64.0,
  35.0 / 64.0,
  11.0 / 64.0,
  43.0 / 64.0,
  1.0 / 64.0,
  33.0 / 64.0,
  9.0 / 64.0,
  41.0 / 64.0,
  51.0 / 64.0,
  19.0 / 64.0,
  59.0 / 64.0,
  27.0 / 64.0,
  49.0 / 64.0,
  17.0 / 64.0,
  57.0 / 64.0,
  25.0 / 64.0,
  15.0 / 64.0,
  47.0 / 64.0,
  7.0 / 64.0,
  39.0 / 64.0,
  13.0 / 64.0,
  45.0 / 64.0,
  5.0 / 64.0,
  37.0 / 64.0,
  63.0 / 64.0,
  31.0 / 64.0,
  55.0 / 64.0,
  23.0 / 64.0,
  61.0 / 64.0,
  29.0 / 64.0,
  53.0 / 64.0,
  21.0 / 64.0,
];

pub const SIMPLE2D: [f32; 4] = [0.0, 0.5, 0.5, 0.0];

pub const ATKINSON: [f32; 12] = [0.0, 0.0, 1.0 / 8.0, 1.0 / 8.0, 1.0 / 8.0, 1.0 / 8.0, 1.0 / 8.0, 0.0, 0.0, 1.0 / 8.0, 0.0, 0.0];

pub const STUCKI: [f32; 15] = [
  0.0,
  0.0,
  0.0,
  8.0 / 42.0,
  4.0 / 42.0,
  2.0 / 42.0,
  4.0 / 42.0,
  8.0 / 42.0,
  4.0 / 42.0,
  2.0 / 42.0,
  1.0 / 42.0,
  2.0 / 42.0,
  4.0 / 42.0,
  2.0 / 42.0,
  1.0 / 42.0,
];

pub const BURKES: [f32; 10] = [
  0.0,
  0.0,
  0.0,
  8.0 / 32.0,
  4.0 / 32.0,
  2.0 / 32.0,
  4.0 / 32.0,
  8.0 / 32.0,
  4.0 / 32.0,
  2.0 / 32.0,
];

pub const SIERRA: [f32; 15] = [
  0.0,
  0.0,
  0.0,
  5.0 / 32.0,
  3.0 / 32.0,
  2.0 / 32.0,
  4.0 / 32.0,
  5.0 / 32.0,
  4.0 / 32.0,
  2.0 / 32.0,
  0.0,
  2.0 / 32.0,
  3.0 / 32.0,
  2.0 / 32.0,
  0.0,
];
pub const TWOROWSIERRA: [f32; 10] = [
  0.0,
  0.0,
  0.0,
  4.0 / 16.0,
  3.0 / 16.0,
  1.0 / 16.0,
  2.0 / 16.0,
  3.0 / 16.0,
  2.0 / 16.0,
  1.0 / 16.0,
];
pub const SIERRALITE: [f32; 6] = [0.0, 0.0, 2.0 / 4.0, 1.0 / 4.0, 1.0 / 4.0, 0.0];

/// Opens an image file and returns its RGB buffer, width, and height.
///
/// # Panics
///
/// This function will panic if:
/// - The image file cannot be opened
/// - The image cannot be decoded
#[must_use]
pub fn open_image(path: &PathBuf) -> (Vec<u8>, u32, u32) {
  //let image = ImageReader::open(path).unwrap().decode().unwrap().into_rgba8();
  let image = ImageReader::open(path).unwrap().decode().unwrap().into_rgb8();

  let (width, height) = image.dimensions();
  let buffer = image.into_raw();
  (buffer, width, height)
}

pub fn save_image(buffer: Vec<u8>, path: PathBuf, width: u32, height: u32) {
  let _ = image::save_buffer(path, &buffer, width, height, ExtendedColorType::Rgb8);
}

pub fn dither(buffer: &mut [u8], dither_type: DitherMethod, color_palette: ColorPalette, width: u32, height: u32) {
  // get the color palette as slice
  let color_palette = match color_palette {
    ColorPalette::Monochrome => &PALETTE_MONOCHROME[..],
    ColorPalette::COLOR8 => &PALETTE_8C[..],
    ColorPalette::COLOR16 => &PALETTE_16C[..],
  };

  match dither_type {
    DitherMethod::None => {
      // Just quantize without dithering
      for cy in 0..height {
        for cx in 0..width {
          let i = ((cy * width + cx) * 3) as usize;
          let (new_color, _) = map_to_palette(Color::from(&buffer[i..i + 3]), color_palette);
          buffer[i] = new_color.r;
          buffer[i + 1] = new_color.g;
          buffer[i + 2] = new_color.b;
        }
      }
    }
    DitherMethod::Bayer2x2 | DitherMethod::Bayer4x4 | DitherMethod::Bayer8x8 => {
      apply_bayer_dithering(buffer, dither_type, color_palette, width, height);
    }
    _ => {
      apply_error_diffusion(buffer, dither_type, color_palette, width, height);
    }
  }
}

fn apply_error_diffusion(buffer: &mut [u8], dither_type: DitherMethod, color_palette: &[Color], width: u32, height: u32) {
  // Define kernel patterns for each algorithm
  let (kernel, kernel_width, kernel_height, kernel_x_offset) = match dither_type {
    DitherMethod::FloydSteinberg => (&FLOYD_STEINBERG[..], 3, 2, 1),
    DitherMethod::Simple2D => (&SIMPLE2D[..], 2, 2, 0),
    DitherMethod::Jarvis => (&JARVIS[..], 5, 3, 2),
    DitherMethod::Atkinson => (&ATKINSON[..], 4, 3, 1),
    DitherMethod::Stucki => (&STUCKI[..], 5, 3, 2),
    DitherMethod::Burkes => (&BURKES[..], 5, 2, 2),
    DitherMethod::Sierra => (&SIERRA[..], 5, 3, 2),
    DitherMethod::TwoRowSierra => (&TWOROWSIERRA[..], 5, 2, 2),
    DitherMethod::SierraLite => (&SIERRALITE[..], 3, 2, 1),
    _ => return, // Should not reach here
  };

  for cy in 0..height {
    for cx in 0..width {
      let i = ((cy * width + cx) * 3) as usize;
      let (new_color, qe) = map_to_palette(Color::from(&buffer[i..i + 3]), color_palette);
      buffer[i] = new_color.r;
      buffer[i + 1] = new_color.g;
      buffer[i + 2] = new_color.b;

      // Spread quantization error to neighboring pixels
      for ky in 0..kernel_height {
        for kx in 0..kernel_width {
          let ki = (ky * kernel_width + kx) as usize;
          if kernel[ki] == 0.0 {
            continue;
          }

          let nx = cx as isize + kx as isize - kernel_x_offset as isize;
          let ny = cy as isize + ky as isize;

          // Skip current pixel (should be 0 in kernel anyway)
          if nx == cx as isize && ny == cy as isize {
            continue;
          }

          if nx < 0 || nx >= width as isize || ny < 0 || ny >= height as isize {
            continue;
          }

          let ni = ((ny as u32 * width + nx as u32) * 3) as usize;
          buffer[ni] = (f32::from(buffer[ni]) + (qe.r * kernel[ki])).round().clamp(0.0, 255.0) as u8;
          buffer[ni + 1] = (f32::from(buffer[ni + 1]) + (qe.g * kernel[ki])).round().clamp(0.0, 255.0) as u8;
          buffer[ni + 2] = (f32::from(buffer[ni + 2]) + (qe.b * kernel[ki])).round().clamp(0.0, 255.0) as u8;
        }
      }
    }
  }
}

fn apply_bayer_dithering(buffer: &mut [u8], dither_type: DitherMethod, color_palette: &[Color], width: u32, height: u32) {
  let (matrix, matrix_size) = match dither_type {
    DitherMethod::Bayer2x2 => (&BAYER2X2[..], 2),
    DitherMethod::Bayer4x4 => (&BAYER4X4[..], 4),
    DitherMethod::Bayer8x8 => (&BAYER8X8[..], 8),
    _ => return,
  };

  for cy in 0..height {
    for cx in 0..width {
      let i = ((cy * width + cx) * 3) as usize;
      let matrix_x = (cx % matrix_size as u32) as usize;
      let matrix_y = (cy % matrix_size as u32) as usize;
      let threshold = matrix[matrix_y * matrix_size + matrix_x];

      // Apply threshold to each color channel
      let mut color = Color::from(&buffer[i..i + 3]);
      color.r = ((f32::from(color.r) / 255.0 + threshold - 0.5).clamp(0.0, 1.0) * 255.0) as u8;
      color.g = ((f32::from(color.g) / 255.0 + threshold - 0.5).clamp(0.0, 1.0) * 255.0) as u8;
      color.b = ((f32::from(color.b) / 255.0 + threshold - 0.5).clamp(0.0, 1.0) * 255.0) as u8;

      let (new_color, _) = map_to_palette(color, color_palette);
      buffer[i] = new_color.r;
      buffer[i + 1] = new_color.g;
      buffer[i + 2] = new_color.b;
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::palette::{PALETTE_8C, PALETTE_MONOCHROME};

  #[test]
  fn test_quantization_error_creation() {
    let error = QuantizationError { r: 10.5, g: -5.2, b: 0.0 };
    assert_eq!(error.r, 10.5);
    assert_eq!(error.g, -5.2);
    assert_eq!(error.b, 0.0);
  }

  #[test]
  fn test_dither_method_default() {
    assert_eq!(DitherMethod::default(), DitherMethod::FloydSteinberg);
  }

  #[test]
  fn test_error_diffusion_kernels_have_correct_size() {
    // Floyd-Steinberg: 2x3 = 6 elements
    assert_eq!(FLOYD_STEINBERG.len(), 6);

    // Simple2D: 2x2 = 4 elements
    assert_eq!(SIMPLE2D.len(), 4);

    // Jarvis: 3x5 = 15 elements
    assert_eq!(JARVIS.len(), 15);

    // Atkinson: 3x4 = 12 elements
    assert_eq!(ATKINSON.len(), 12);

    // Stucki: 3x5 = 15 elements
    assert_eq!(STUCKI.len(), 15);

    // Burkes: 2x5 = 10 elements
    assert_eq!(BURKES.len(), 10);

    // Sierra: 3x5 = 15 elements
    assert_eq!(SIERRA.len(), 15);

    // Two-row Sierra: 2x5 = 10 elements
    assert_eq!(TWOROWSIERRA.len(), 10);

    // Sierra Lite: 2x3 = 6 elements
    assert_eq!(SIERRALITE.len(), 6);
  }

  #[test]
  fn test_bayer_matrices_have_correct_size() {
    assert_eq!(BAYER2X2.len(), 4); // 2x2
    assert_eq!(BAYER4X4.len(), 16); // 4x4
    assert_eq!(BAYER8X8.len(), 64); // 8x8
  }

  #[test]
  fn test_kernel_weights_sum_to_one() {
    // Floyd-Steinberg weights should sum to 1.0 (excluding the center pixel which is 0)
    let floyd_sum: f32 = FLOYD_STEINBERG.iter().sum();
    assert!((floyd_sum - 1.0).abs() < f32::EPSILON);

    // Sierra Lite weights should sum to 1.0
    let sierra_lite_sum: f32 = SIERRALITE.iter().sum();
    assert!((sierra_lite_sum - 1.0).abs() < f32::EPSILON);
  }

  #[test]
  fn test_dither_none_only_quantizes() {
    let mut buffer = vec![128, 128, 128, 64, 64, 64]; // 2 pixels: gray, dark gray
    let original = buffer.clone();

    dither(&mut buffer, DitherMethod::None, ColorPalette::Monochrome, 2, 1);

    // Should be quantized to black and white, but no error diffusion
    assert_ne!(buffer, original);

    // All pixels should be either 0 or 255 for monochrome
    for chunk in buffer.chunks_exact(3) {
      let (r, g, b) = (chunk[0], chunk[1], chunk[2]);
      assert!(r == 0 || r == 255);
      assert!(g == 0 || g == 255);
      assert!(b == 0 || b == 255);
      assert_eq!(r, g); // Should be grayscale
      assert_eq!(g, b);
    }
  }

  #[test]
  fn test_dither_modifies_buffer() {
    let mut buffer = vec![100, 150, 200, 50, 75, 25]; // 2 pixels
    let original = buffer.clone();

    dither(&mut buffer, DitherMethod::FloydSteinberg, ColorPalette::COLOR8, 2, 1);

    assert_ne!(buffer, original, "Dithering should modify the buffer");
  }

  #[test]
  fn test_buffer_bounds_safety() {
    // Test with minimal buffer to ensure no out-of-bounds access
    let mut buffer = vec![128, 128, 128]; // 1x1 pixel

    // This should not panic
    dither(&mut buffer, DitherMethod::FloydSteinberg, ColorPalette::Monochrome, 1, 1);

    assert_eq!(buffer.len(), 3); // Should still be RGB
  }

  #[test]
  fn test_apply_error_diffusion_handles_edges() {
    // Test edge case handling in error diffusion
    let mut buffer = vec![
      100, 100, 100, // (0,0)
      200, 200, 200, // (1,0)
    ];

    apply_error_diffusion(&mut buffer, DitherMethod::FloydSteinberg, &PALETTE_MONOCHROME, 2, 1);

    // Should not panic and buffer should be modified
    assert_eq!(buffer.len(), 6);
  }

  #[test]
  fn test_apply_bayer_dithering() {
    let mut buffer = vec![
      100, 100, 100, // (0,0)
      150, 150, 150, // (1,0)
      200, 200, 200, // (0,1)
      75, 75, 75, // (1,1)
    ];

    apply_bayer_dithering(&mut buffer, DitherMethod::Bayer2x2, &PALETTE_8C, 2, 2);

    // Should not panic and buffer should be modified
    assert_eq!(buffer.len(), 12);
  }

  #[test]
  fn test_all_algorithms_dont_panic() {
    let buffer = vec![128, 64, 192, 32, 160, 96]; // 2x1 image

    let algorithms = [
      DitherMethod::None,
      DitherMethod::FloydSteinberg,
      DitherMethod::Simple2D,
      DitherMethod::Jarvis,
      DitherMethod::Atkinson,
      DitherMethod::Stucki,
      DitherMethod::Burkes,
      DitherMethod::Sierra,
      DitherMethod::TwoRowSierra,
      DitherMethod::SierraLite,
      DitherMethod::Bayer2x2,
      DitherMethod::Bayer4x4,
      DitherMethod::Bayer8x8,
    ];

    for algorithm in algorithms {
      let mut test_buffer = buffer.clone();

      // None of these should panic
      dither(&mut test_buffer, algorithm, ColorPalette::COLOR8, 2, 1);

      assert_eq!(test_buffer.len(), 6, "Buffer size should remain consistent for {:?}", algorithm);
    }
  }
}
