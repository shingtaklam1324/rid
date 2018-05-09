extern crate clap;
extern crate image;
extern crate md5;
extern crate palette;
extern crate rand;

use image::{ImageBuffer, Rgb};
use rand::{Rng, SeedableRng, XorShiftRng};

use std::mem::transmute_copy;

use clap::{App, Arg};

use palette::FromColor;

fn main() {
    let matches = App::new("rid")
        .version("0.1.0")
        .author("Shing Tak Lam")
        .about("A quick identicon generator")
        .arg(Arg::with_name("USERNAME").required(true).index(1))
        .arg(Arg::with_name("OUT DIR").short("o").takes_value(true))
        .arg(
            Arg::with_name("SEED")
                .long("seed")
                .takes_value(true)
                .help("Seed for the RNG, any valid 32-bit unsigned integer"),
        )
        .arg(
            Arg::with_name("SALT")
                .long("salt")
                .takes_value(true)
                .help("Salt for the hashing, any string"),
        )
        .arg(
            Arg::with_name("HUE")
                .long("hue")
                .takes_value(true)
                .help("Hue for the output image, unit: degree"),
        )
        .arg(
            Arg::with_name("SATURATION")
                .long("sat")
                .takes_value(true)
                .help("Saturation for the output image, unit: %"),
        )
        .arg(
            Arg::with_name("MULTI COLORED")
                .short("m")
                .long("multi")
                .multiple(true)
                .help("Allows for multi-colored image output"),
        )
        .get_matches();

    let username = matches.value_of("USERNAME").unwrap();
    let hash_username = match matches.value_of("SALT") {
        Some(s) => format!("{}{}", username, s),
        None => username.to_owned(),
    };
    let out_dir = matches
        .value_of("OUT DIR")
        .map(str::to_owned)
        .unwrap_or(format!("{}.png", username));

    let hash = md5::compute(&hash_username);

    let seed = match matches.value_of("SEED") {
        None => unsafe { transmute_copy(&hash.0) },
        Some(s) => [0, 0, 0, s.parse().unwrap()],
    };

    let hue = matches
        .value_of("HUE")
        .map(|s| s.parse().unwrap())
        .unwrap_or((hash.0)[1] as f32 * (360.0 / 256.0));
    let sat = matches
        .value_of("SATURATION")
        .map(|s| s.parse().unwrap())
        .unwrap_or((hash.0)[6] as f32 * (100.0 / 256.0));

    let sat = sat + (100.0 * matches.occurrences_of("MULTI COLORED") as f32);

    let (mut h, mut v, ca, cb) = match hash.0 {
        [a, b, c, d, e, f, g, h, i, j, k, l, m, n, o, p] => (
            [0, a, b, c, d, e, f, g, 255],
            [0, h, i, j, k, l, m, n, 255],
            o as f32 / 255.0,
            p as f32 / 255.0,
        ),
    };

    let (black, gray, white) = (0.0, 0.5, 1.0);

    h.sort();
    v.sort();

    let colors = [black, ca, gray, cb, white]
        .iter()
        .map(|x| {
            palette::rgb::Rgb::from_hsv(palette::Hsv::new(
                palette::RgbHue::from(hue),
                sat / 100.0,
                *x,
            ))
        })
        .map(|c: palette::rgb::Rgb| {
            Rgb([
                (c.red * 255.0) as u8,
                (c.green * 255.0) as u8,
                (c.blue * 255.0) as u8,
            ])
        })
        .collect::<Vec<_>>();

    let mut rng: XorShiftRng = SeedableRng::from_seed(seed);

    let mut img = image::ImageBuffer::new(256, 256);
    for x in 0..8 {
        for y in 0..8 {
            let c = rng.choose(&colors).unwrap();
            draw_rect(&mut img, v[y], h[x], v[y + 1] - v[y], h[x + 1] - h[x], *c)
        }
    }
    img.save(out_dir).unwrap();
}

fn draw_rect(img: &mut ImageBuffer<Rgb<u8>, Vec<u8>>, x: u8, y: u8, w: u8, h: u8, pixel: Rgb<u8>) {
    for x in x..(x + w) {
        for y in y..(y + h) {
            img.put_pixel(x as u32, y as u32, pixel)
        }
    }
}
