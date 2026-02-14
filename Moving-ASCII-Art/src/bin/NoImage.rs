use noise::{NoiseFn, Perlin};
use std::{fs, thread, time::Duration};
use std::time::Instant;

fn main()
{
    let perlin = Perlin::new(1);
    let mut t: f64 = 0.0;

    let tick = Duration::from_millis(16);
    let write_interval = Duration::from_millis(33);

    let mut last_tick = Instant::now();
    let mut last_write = Instant::now();
    let mut last_ascii = String::new();

    loop
    {
        let frame_start = Instant::now();

        let dt = (frame_start - last_tick).as_secs_f64();
        last_tick = frame_start;
        t += dt;

        if last_write.elapsed() >= write_interval
        {
            last_write = frame_start;

            let ascii = dither_without_image(&perlin, t);

            if ascii != last_ascii
            {
                last_ascii = ascii;
                fs::write("ascii.tmp", &last_ascii).unwrap();
                fs::rename("ascii.tmp", "ascii.txt").unwrap();
            }
        }
        

        let elapsed = frame_start.elapsed();
        if elapsed < tick
        {
            thread::sleep(tick - elapsed);
        }
    }
}

fn pixel_to_ascii(value: u8) -> char
{
    let chars: &[u8] = b".:-=+*%#";
    let t = (value as f32) / 255.0;
    let idx = (t * (chars.len() as f32 - 1.0)) as usize;
    chars[idx] as char
}

fn hash01(x: u32, y: u32) -> f32 
{
    let mut v = x.wrapping_mul(374761393) ^ y.wrapping_mul(668265263);
    v = (v ^ (v >> 13)).wrapping_mul(1274126177);
    v ^= v >> 16;
    (v as f32) / (u32::MAX as f32)
}

fn create_lerping_perlin(seed: u32) -> Perlin
{
    let mut p = Perlin::new(seed);
    for i in 0..256
    {
        seed.wrapping_mul(374761393) ^ (i as u32).wrapping_mul(668265263);
    }
    p
}

fn dither_without_image(perlin: &Perlin, t: f64) -> String
{
    let width = 128;
    let height = 96;

    let mut pixels: Vec<f32> = Vec::with_capacity(width * height);

    let scale = 0.09;
    let strength = 10.0;
    let speed = 0.05;

    for y in 0..height
    {
        for x in 0..width
        {
            let base = 0.5;
            let phase = hash01(x as u32, y as u32) as f64 * 10.0;
            let n = perlin.get([x as f64 * scale, y as f64 * scale, (t + phase) * speed]) as f32;
            let amp = 0.20_f32;
            let mut v = (base + n * amp).clamp(0.0, 1.0);
            let contrast: f32 = 4.0;
            v = ((v - 0.5) * contrast + 0.5).clamp(0.0, 1.0);
            pixels.push(v * 255.0);
        }
    }

    let w = 128;
    let h = 96;

    let width = w as usize;
    let height = h as usize;

    let mut out = String::with_capacity((width + 1) * height);

    let flow_freq: f64  = 0.02;
    let flow_speed: f64 = 0.10;
    let warp_amp: f64   = 5.0;

    for y in 0..height
    {
        for x in 0..width
        {
            let xf = x as f64;
            let yf = y as f64;

            let drift_x: f64 = t * 0.35;
            let drift_y: f64 = t * 0.15;

            let dx = perlin.get([xf * flow_freq + drift_x, yf * flow_freq + drift_y, t * flow_speed]);
            let dy = perlin.get([xf * flow_freq + 19.7 + drift_x, yf * flow_freq - 31.3 + drift_y, t * flow_speed]);

            let sx = (xf + dx * warp_amp).round() as i32;
            let sy = (yf + dy * warp_amp).round() as i32;

            let sx = sx.clamp(0, w - 1) as u32;
            let sy = sy.clamp(0, h - 1) as u32;

            let value = pixels[sy as usize * width + sx as usize];
            out.push(pixel_to_ascii(value.round() as u8));
        }
        out.push('\n');
    }
    out
}