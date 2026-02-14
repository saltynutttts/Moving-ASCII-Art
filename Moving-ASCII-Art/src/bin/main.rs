use image::{open, GrayImage, RgbaImage};
use dithereens::{InterleavedGradientNoise, simple_dither_slice_2d};
use noise::{NoiseFn, Perlin};
use std::{fs, thread, time::Duration};
use image::imageops::FilterType;
use std::time::Instant;
use std::io::{self, Write};

fn main()
{
    let img = open("test.png").unwrap();
    let rgba_img: RgbaImage = img.into_rgba8();
    let gray_img_raw = rgba_to_gray_using_alpha(&rgba_img);
    let (w, h) = gray_img_raw.dimensions();
    let aspect_fix = 1.00_f32;
    let new_w = ((w as f32) * aspect_fix).max(1.0).round() as u32;

    let down: u32 = 2;

    let gray_img = image::imageops::resize(&gray_img_raw, (new_w / down).max(1), (h / down).max(1), FilterType::Triangle);

    gray_img.save("gray_test.png").unwrap();

    let perlin = Perlin::new(1);
    let mut t: f64 = 0.0;

    let target_frame = Duration::from_millis(10);
    let mut last = Instant::now();

    loop
    {
        let frame_start = Instant::now();

        let now = frame_start;
        let dt = (now - last).as_secs_f64();
        last = now;
        t += dt;

        let ascii = dither_image(&gray_img, &perlin, t);
        fs::write("ascii.tmp", ascii).unwrap();
        fs::rename("ascii.tmp", "ascii.txt").unwrap();

        let elapsed = frame_start.elapsed();
        if elapsed < target_frame
        {
            thread::sleep(target_frame - elapsed);
        }
    }
}

fn dither_image(gray: &GrayImage, perlin: &Perlin, t: f64) -> String
{
    let (width_u32, height_u32) = gray.dimensions();
    let width = width_u32 as usize;
    let height = height_u32 as usize;

    let mut pixels: Vec<f32> = Vec::with_capacity(width * height);

    let scale = 5.0;
    let strength = 3.0;
    let speed = 100.0;

    for y in 0..height
    {
        for x in 0..width
        {
            let base = gray.get_pixel(x as u32, y as u32)[0] as f32 / 255.0;
            let phase = hash01(x as u32, y as u32) as f64 * 10.0;
            let n = perlin.get([x as f64 * scale, y as f64 * scale, (t + phase) * speed]) as f32;
            let amp = 0.20_f32;
            let warped = (base + n * amp).clamp(0.0, 1.0) * 255.0;
            pixels.push(warped);
        }
    }

    let (w_u32, h_u32) = gray.dimensions();
    let w = w_u32 as i32;
    let h = h_u32 as i32;

    let width = w_u32 as usize;
    let height = h_u32 as usize;

    let mut out = String::with_capacity((width + 1) * height);

    let flow_freq: f64  = 0.135;
    let flow_speed: f64 = 1.5;
    let warp_amp: f64   = 2.0;

    for y in 0..height
    {
        for x in 0..width
        {
            let xf = x as f64;
            let yf = y as f64;

            let dx = perlin.get([xf * flow_freq, yf * flow_freq, t * flow_speed]);
            let dy = perlin.get([xf * flow_freq + 19.7, yf * flow_freq - 31.3, t * flow_speed]);

            let sx = (xf + dx * warp_amp).round() as i32;
            let sy = (yf + dy * warp_amp).round() as i32;

            let sx = sx.clamp(0, w - 1) as u32;
            let sy = sy.clamp(0, h - 1) as u32;

            let value = gray.get_pixel(sx, sy)[0];
            out.push(pixel_to_ascii(value));
        }
        out.push('\n');
    }
    out
}

fn pixel_to_ascii(value: u8) -> char
{
    let chars: &[u8] = b".:-=+*%#";
    let t = (value as f32) / 255.0;
    let idx = (t * (chars.len() as f32 - 1.0)) as usize;
    chars[idx] as char
}

fn rgba_to_gray_using_alpha(rgba_img: &RgbaImage) -> GrayImage {
    let (width, height) = rgba_img.dimensions();
    let mut out = GrayImage::new(width, height);


    for y in 0..height {
        for x in 0..width {
            let p = rgba_img.get_pixel(x, y).0;
            let (r, g, b, a) = (p[0] as f32, p[1] as f32, p[2] as f32, p[3] as f32);

            let luma = 0.2126 * r + 0.7152 * g + 0.0722 * b;
            let luma_on_black = luma * (a / 255.0);

            out.put_pixel(x, y, image::Luma([luma_on_black.round().clamp(0.0, 255.0) as u8]));
        }
    }

    out
}

fn hash01(x: u32, y: u32) -> f32 
{
    let mut v = x.wrapping_mul(374761393) ^ y.wrapping_mul(668265263);
    v = (v ^ (v >> 13)).wrapping_mul(1274126177);
    v ^= v >> 16;
    (v as f32) / (u32::MAX as f32)
}

