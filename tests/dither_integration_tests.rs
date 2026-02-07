use dithers::dither::{DitherMethod, dither, open_image, save_image};
use dithers::palette::ColorPalette;
use std::fs;
use std::path::PathBuf;

const TEST_IMAGE: &str = "test/in/glace-1280_853.jpg";

#[test]
fn test_image_exists() {
  assert!(PathBuf::from(TEST_IMAGE).exists(), "Test image {} not found", TEST_IMAGE);
}

#[test]
fn test_open_image() {
  let (buffer, width, height) = open_image(&PathBuf::from(TEST_IMAGE));

  assert!(width > 0, "Image width should be greater than 0");
  assert!(height > 0, "Image height should be greater than 0");
  assert_eq!(
    buffer.len(),
    (width * height * 3) as usize,
    "Buffer size should match width * height * 3 for RGB"
  );
}

#[test]
fn test_floyd_steinberg_all_palettes() {
  let (buffer, width, height) = open_image(&PathBuf::from(TEST_IMAGE));
  let original_buffer = buffer.clone();

  // Test with monochrome
  let mut test_buffer = original_buffer.clone();
  dither(&mut test_buffer, DitherMethod::FloydSteinberg, ColorPalette::Monochrome, width, height);
  assert_ne!(test_buffer, original_buffer, "Buffer should be modified by dithering");

  // Test with 8-color
  let mut test_buffer = original_buffer.clone();
  dither(&mut test_buffer, DitherMethod::FloydSteinberg, ColorPalette::COLOR8, width, height);
  assert_ne!(test_buffer, original_buffer, "Buffer should be modified by dithering");

  // Test with 16-color
  let mut test_buffer = original_buffer.clone();
  dither(&mut test_buffer, DitherMethod::FloydSteinberg, ColorPalette::COLOR16, width, height);
  assert_ne!(test_buffer, original_buffer, "Buffer should be modified by dithering");
}

#[test]
fn test_error_diffusion_algorithms() {
  let (buffer, width, height) = open_image(&PathBuf::from(TEST_IMAGE));
  let original_buffer = buffer.clone();

  let algorithms = [
    DitherMethod::FloydSteinberg,
    DitherMethod::Simple2D,
    DitherMethod::Jarvis,
    DitherMethod::Atkinson,
    DitherMethod::Stucki,
    DitherMethod::Burkes,
    DitherMethod::Sierra,
    DitherMethod::TwoRowSierra,
    DitherMethod::SierraLite,
  ];

  for algorithm in algorithms {
    let mut test_buffer = original_buffer.clone();
    dither(&mut test_buffer, algorithm, ColorPalette::COLOR8, width, height);
    assert_ne!(test_buffer, original_buffer, "Algorithm {:?} should modify the buffer", algorithm);
  }
}

#[test]
fn test_bayer_algorithms() {
  let (buffer, width, height) = open_image(&PathBuf::from(TEST_IMAGE));
  let original_buffer = buffer.clone();

  let algorithms = [DitherMethod::Bayer2x2, DitherMethod::Bayer4x4, DitherMethod::Bayer8x8];

  for algorithm in algorithms {
    let mut test_buffer = original_buffer.clone();
    dither(&mut test_buffer, algorithm, ColorPalette::COLOR8, width, height);
    assert_ne!(test_buffer, original_buffer, "Bayer algorithm {:?} should modify the buffer", algorithm);
  }
}

#[test]
fn test_no_dithering() {
  let (buffer, width, height) = open_image(&PathBuf::from(TEST_IMAGE));
  let original_buffer = buffer.clone();

  let mut test_buffer = original_buffer.clone();
  dither(&mut test_buffer, DitherMethod::None, ColorPalette::COLOR8, width, height);

  // Should still modify buffer due to palette quantization
  assert_ne!(test_buffer, original_buffer, "Even 'None' dithering should quantize colors");
}

#[test]
fn test_monochrome_palette_output() {
  let (buffer, width, height) = open_image(&PathBuf::from(TEST_IMAGE));
  let mut test_buffer = buffer;

  dither(&mut test_buffer, DitherMethod::FloydSteinberg, ColorPalette::Monochrome, width, height);

  // Check that all pixels are either black (0,0,0) or white (255,255,255)
  for chunk in test_buffer.chunks_exact(3) {
    let (r, g, b) = (chunk[0], chunk[1], chunk[2]);
    assert!(
      (r == 0 && g == 0 && b == 0) || (r == 255 && g == 255 && b == 255),
      "Monochrome pixel should be black (0,0,0) or white (255,255,255), got ({},{},{})",
      r,
      g,
      b
    );
  }
}

#[test]
fn test_save_and_cleanup() {
  let (mut buffer, width, height) = open_image(&PathBuf::from(TEST_IMAGE));

  dither(&mut buffer, DitherMethod::FloydSteinberg, ColorPalette::Monochrome, width, height);

  let output_path = PathBuf::from("test_output_integration.png");
  save_image(buffer, output_path.clone(), width, height);

  assert!(output_path.exists(), "Output image should be created");

  // Cleanup
  fs::remove_file(output_path).expect("Should be able to clean up test file");
}

#[test]
fn test_all_algorithms_with_all_palettes() {
  let (buffer, width, height) = open_image(&PathBuf::from(TEST_IMAGE));

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

  let palettes = [ColorPalette::Monochrome, ColorPalette::COLOR8, ColorPalette::COLOR16];

  for algorithm in algorithms {
    for palette in palettes {
      let mut test_buffer = buffer.clone();

      // This should not panic
      dither(&mut test_buffer, algorithm, palette, width, height);

      // Buffer should be valid RGB data
      assert_eq!(
        test_buffer.len(),
        (width * height * 3) as usize,
        "Buffer size should remain consistent for {:?} with {:?}",
        algorithm,
        palette
      );

      // All values should be valid RGB (0-255) - u8 is always <= 255, but test for completeness
      for &_value in &test_buffer {
        // u8 values are always 0-255, so this is implicitly satisfied
      }
    }
  }
}

#[test]
fn test_buffer_bounds() {
  let (buffer, width, height) = open_image(&PathBuf::from(TEST_IMAGE));
  let mut test_buffer = buffer;

  // Test with edge case: 1x1 image would be too small, so test with actual image
  // but verify no out-of-bounds access occurs
  dither(&mut test_buffer, DitherMethod::FloydSteinberg, ColorPalette::Monochrome, width, height);

  // If we get here without panicking, bounds checking worked
  assert_eq!(test_buffer.len(), (width * height * 3) as usize);
}

#[test]
fn test_different_algorithms_produce_different_results() {
  let (buffer, width, height) = open_image(&PathBuf::from(TEST_IMAGE));

  let mut floyd_buffer = buffer.clone();
  let mut atkinson_buffer = buffer.clone();
  let mut bayer_buffer = buffer;

  dither(&mut floyd_buffer, DitherMethod::FloydSteinberg, ColorPalette::COLOR8, width, height);
  dither(&mut atkinson_buffer, DitherMethod::Atkinson, ColorPalette::COLOR8, width, height);
  dither(&mut bayer_buffer, DitherMethod::Bayer4x4, ColorPalette::COLOR8, width, height);

  // Different algorithms should produce different results
  assert_ne!(floyd_buffer, atkinson_buffer, "Floyd-Steinberg and Atkinson should produce different results");
  assert_ne!(floyd_buffer, bayer_buffer, "Floyd-Steinberg and Bayer should produce different results");
  assert_ne!(atkinson_buffer, bayer_buffer, "Atkinson and Bayer should produce different results");
}
