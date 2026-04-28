#![allow(dead_code)]
use image::{DynamicImage, GenericImageView, GrayImage, Luma};

pub fn apply_floyd_steinberg(img: &DynamicImage) -> DynamicImage {
    let (width, height) = img.dimensions();
    let gray = img.to_luma8();
    let mut pixels = gray.to_vec();
    let w = width as usize;
    let h = height as usize;

    for y in 0..h {
        for x in 0..w {
            let idx = y * w + x;
            let old = pixels[idx];
            let new = if old > 128 { 255 } else { 0 };
            let error = (old as i16) - (new as i16);
            pixels[idx] = new;

            if x + 1 < w {
                pixels[y * w + x + 1] = ((pixels[y * w + x + 1] as i16) + (error * 7 / 16))
                    .max(0)
                    .min(255) as u8;
            }
            if y + 1 < h {
                if x > 0 {
                    pixels[(y + 1) * w + x - 1] = ((pixels[(y + 1) * w + x - 1] as i16)
                        + (error * 3 / 16))
                        .max(0)
                        .min(255) as u8;
                }
                pixels[(y + 1) * w + x] = ((pixels[(y + 1) * w + x] as i16) + (error * 5 / 16))
                    .max(0)
                    .min(255) as u8;
                if x + 1 < w {
                    pixels[(y + 1) * w + x + 1] = ((pixels[(y + 1) * w + x + 1] as i16)
                        + (error * 1 / 16))
                        .max(0)
                        .min(255) as u8;
                }
            }
        }
    }

    let mut out = GrayImage::new(width, height);
    for y in 0..height {
        for x in 0..width {
            out.put_pixel(x, y, Luma([pixels[(y as usize) * w + (x as usize)]]));
        }
    }
    DynamicImage::ImageLuma8(out)
}

pub fn apply_ordered_dither(img: &DynamicImage) -> DynamicImage {
    let (width, height) = img.dimensions();
    let gray = img.to_luma8();
    let mut out = GrayImage::new(width, height);
    let matrix = [[0, 8, 2, 10], [12, 4, 14, 6], [3, 11, 1, 9], [15, 7, 13, 5]];

    for y in 0..height {
        for x in 0..width {
            let val = gray.get_pixel(x, y)[0];
            let threshold = (matrix[(y as usize) % 4][(x as usize) % 4] * 256) / 16;
            let new_val = if val > threshold as u8 { 255 } else { 0 };
            out.put_pixel(x, y, Luma([new_val]));
        }
    }
    DynamicImage::ImageLuma8(out)
}

pub fn apply_bayer_dither(img: &DynamicImage, level: u8) -> DynamicImage {
    let (width, height) = img.dimensions();
    let gray = img.to_luma8();
    let mut out = GrayImage::new(width, height);
    let threshold = (level as f32 * 255.0 / 100.0) as u8;

    for y in 0..height {
        for x in 0..width {
            let val = gray.get_pixel(x, y)[0];
            let new_val = if val > threshold { 255 } else { 0 };
            out.put_pixel(x, y, Luma([new_val]));
        }
    }
    DynamicImage::ImageLuma8(out)
}

pub fn apply_stucki_dither(img: &DynamicImage) -> DynamicImage {
    let (width, height) = img.dimensions();
    let gray = img.to_luma8();
    let mut pixels = gray.to_vec();
    let w = width as usize;
    let h = height as usize;

    for y in 0..h {
        for x in 0..w {
            let idx = y * w + x;
            let old = pixels[idx];
            let new = if old > 128 { 255 } else { 0 };
            let error = (old as i16) - (new as i16);
            pixels[idx] = new;

            let coeffs = [
                (1, 0, 8),
                (2, 0, 8),
                (-2, 1, 8),
                (-1, 1, 8),
                (0, 1, 8),
                (1, 1, 8),
                (2, 1, 8),
                (-2, 2, 8),
                (-1, 2, 8),
                (0, 2, 8),
                (1, 2, 8),
                (2, 2, 8),
            ];
            for (dx, dy, c) in coeffs {
                let nx = (x as i32 + dx) as usize;
                let ny = (y as i32 + dy) as usize;
                if nx < w && ny < h {
                    let nidx = ny * w + nx;
                    pixels[nidx] = ((pixels[nidx] as i16) + (error * c / 8)).max(0).min(255) as u8;
                }
            }
        }
    }

    let mut out = GrayImage::new(width, height);
    for y in 0..height {
        for x in 0..width {
            out.put_pixel(x, y, Luma([pixels[(y as usize) * w + (x as usize)]]));
        }
    }
    DynamicImage::ImageLuma8(out)
}
