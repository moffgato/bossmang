use {
    crate::error::Error,
    image::{DynamicImage, GenericImageView, ImageBuffer, Rgba},
    rand::seq::SliceRandom,
    rand::Rng,
    std::path::PathBuf,
};

pub fn hide_data(img_path: &PathBuf, data: &[u8]) -> Result<(), Error> {
    let img = image::open(img_path).map_err(Error::Image)?;
    let (width, height) = img.dimensions();

    if (width * height * 3) < (data.len() * 8) as u32 {
        return Err(Error::Storage(format!(
            "Image too small for data: needs {} bits, has {} bits available",
            data.len() * 8,
            width * height * 3
        )));
    }

    let mut rng = rand::thread_rng();
    let mut positions: Vec<(u32, u32)> = (0..width)
        .flat_map(|x| (0..height).map(move |y| (x, y)))
        .collect();
    positions.shuffle(&mut rng);

    let data_bits = data.iter()
        .flat_map(|&byte| (0..8).map(move |i| (byte >> i) & 1))
        .collect::<Vec<_>>();

    let mut img_buf = img.to_rgba8();

    for ((x, y), &bit) in positions.iter().zip(data_bits.iter()) {
        let pixel = img_buf.get_pixel_mut(*x, *y);

        let channel = rng.gen_range(0..3);
        pixel[channel] &= 0xFE;
        pixel[channel] |= bit;
    }

    let output_path = img_path.with_extension("png");
    img_buf.save(&output_path).map_err(|e| Error::Storage(format!("Failed to save image: {}", e)))?;

    Ok(())
}

pub fn extract_data(img_path: &PathBuf, data_len: usize) -> Result<Vec<u8>, Error> {
    let img = image::open(img_path).map_err(Error::Image)?;
    let (width, height) = img.dimensions();

    if (width * height * 3) < (data_len * 8) as u32 {
        return Err(Error::Storage(format!(
            "Image too small for data length: needs {} bits, has {} bits available",
            data_len * 8,
            width * height * 3
        )));
    }

    let mut rng = rand::thread_rng();
    let mut positions: Vec<(u32, u32)> = (0..width)
        .flat_map(|x| (0..height).map(move |y| (x, y)))
        .collect();
    positions.shuffle(&mut rng);

    let mut result = Vec::with_capacity(data_len);
    let mut current_byte = 0u8;
    let mut bit_count = 0;

    for (i, (x, y)) in positions.iter().enumerate() {
        if i >= data_len * 8 {
            break;
        }

        let pixel = img.get_pixel(*x, *y);
        let channel = rng.gen_range(0..3);
        let bit = pixel[channel] & 1;

        current_byte |= bit << bit_count;
        bit_count += 1;

        if bit_count == 8 {
            result.push(current_byte);
            current_byte = 0;
            bit_count = 0;
        }
    }

    Ok(result)
}


#[cfg(test)]
mod tests {
    use super::*;
    use image::{RgbaImage, Rgba};
//    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_hide_and_extract() -> Result<(), Error> {
        let dir = tempdir().map_err(|e| Error::Storage(e.to_string()))?;
        let img_path = dir.path().join("test.png");

        let mut img = RgbaImage::new(100, 100);
        for pixel in img.pixels_mut() {
            *pixel = Rgba([255, 255, 255, 255]);
        }
        img.save(&img_path).map_err(|e| Error::Storage(e.to_string()))?;

        let test_data = b"Hello, World!";

        hide_data(&img_path, test_data)?;

        let extracted = extract_data(&img_path, test_data.len())?;

        assert_eq!(test_data, extracted.as_slice());

        Ok(())
    }

    #[test]
    fn test_image_too_small() {
        let dir = tempdir().unwrap();
        let img_path = dir.path().join("small.png");

        let img = RgbaImage::new(2, 2);
        img.save(&img_path).unwrap();

        let result = hide_data(&img_path, &vec![0; 100]);
        assert!(result.is_err());
    }
}
