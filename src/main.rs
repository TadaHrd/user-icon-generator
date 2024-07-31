use std::io::{Error, Read};

use bmp::{Image, Pixel};
use colors_transform::{Color, Hsl};

fn bits(word: u16) -> [bool; 16] {
    [
        word & 1 != 0,
        word >> 1 & 1 != 0,
        word >> 2 & 1 != 0,
        word >> 3 & 1 != 0,
        word >> 4 & 1 != 0,
        word >> 5 & 1 != 0,
        word >> 6 & 1 != 0,
        word >> 7 & 1 != 0,
        word >> 8 & 1 != 0,
        word >> 9 & 1 != 0,
        word >> 10 & 1 != 0,
        word >> 11 & 1 != 0,
        word >> 12 & 1 != 0,
        word >> 13 & 1 != 0,
        word >> 14 & 1 != 0,
        word >> 15 & 1 != 0,
    ]
}

fn hash(text: &[u8]) -> u32 {
    let mut hash: u32 = 656_379_989;

    let mut last_byte = 0;

    for &byte in text {
        hash = hash.wrapping_mul(byte as u32);

        hash ^= (byte as u32).rotate_left(5).wrapping_neg();

        hash = hash.wrapping_sub(last_byte as u32);

        last_byte = byte;
    }

    hash.wrapping_mul(205_676_507)
}

fn get_parts(mut hash: u32) -> (Pixel, [bool; 15]) {
    let hue = hash % 360;
    hash /= 360;

    let color = Hsl::from(hue as f32, 70.0, 50.0).to_rgb();
    let (r, g, b) = (
        color.get_red() as u8,
        color.get_green() as u8,
        color.get_blue() as u8,
    );

    let bits = bits((hash & 0xFFFF) as u16)[0..15].try_into().unwrap();
    (Pixel::new(r, g, b), bits)
}

fn main() -> Result<(), Error> {
    let mut args = std::env::args();
    let arg = args.nth(1);

    match arg {
        Some(arg) if arg == "--find" => 'a: {
            let arg = match args.next() {
                Some(arg) => arg,
                None => break 'a,
            };

            let mut array = [false; 25];

            let mut sub: usize = 0;
            for (i, c) in arg.chars().enumerate() {
                if c == ' ' {
                    sub += 1;
                    continue;
                }
                if i - sub >= 25 {
                    println!("{array:?}");
                    break;
                }
                array[i.saturating_sub(sub)] = c != '0';
            }

            let printable_chars = include!("chars.rs");

            for i in 0..usize::MAX {
                use itertools::Itertools;

                'outer: for combination in printable_chars.iter().combinations_with_replacement(i) {
                    print!("\rAt length {}", i);

                    let combo: String = combination.into_iter().collect();

                    let image = make_image(&combo)?;

                    for y in 0..5 {
                        #[allow(clippy::all)]
                        for x in 0..5 {
                            let pixel = image.get_pixel(x, y);
                            // save_show_image(&combo, image)?;
                            // return Ok(());

                            let mut value = false;
                            if pixel.r != 255 || pixel.g != 255 || pixel.b != 255 {
                                value = true
                            }
                            // println!("{} {} {}", y * 5 + x, array[(y * 5 + x) as usize], value);

                            if array[(y * 5 + x) as usize] != value {
                                continue 'outer;
                            }
                        }
                    }

                    println!("\n");
                    save_show_image(&combo, image)?;
                }
            }

            println!("didn't find anything");

            return Ok(());
        }
        Some(arg) => {
            save_make_show_image(&arg)?;
            return Ok(());
        }
        None => (),
    }

    for text in include!("strings.rs") {
        save_make_show_image(text)?;
    }

    Ok(())
}

fn make_image(text: &str) -> Result<Image, Error> {
    let mut image = bmp::Image::new(5, 5);

    let hash = hash(text.as_bytes());
    let parts = get_parts(hash);

    for i in 0..15 {
        let color = match parts.1[i] {
            true => Pixel::new(255, 255, 255),
            false => parts.0,
        };

        if i < 5 {
            image.set_pixel(2, i as u32, color);
        } else if i < 10 {
            image.set_pixel(1, i as u32 - 5, color);
            image.set_pixel(3, i as u32 - 5, color);
        } else if i < 15 {
            image.set_pixel(0, i as u32 - 10, color);
            image.set_pixel(4, i as u32 - 10, color);
        }
    }

    Ok(image)
}

fn show_image(text: &str, image: Image) -> Result<(Image, bool), Error> {
    println!("Displaying: {}", text);

    for y in 0..5 {
        for x in 0..5 {
            print!("\x1B[107m");
            let pixel = image.get_pixel(x, y);
            if pixel.r == 255 && pixel.g == 255 && pixel.b == 255 {
                print!("  ");
            } else {
                print!("\x1B[38;2;{};{};{}m██\x1B[0m", pixel.r, pixel.g, pixel.b);
            }
        }

        println!("\x1B[0m")
    }
    println!("press enter to continue, save with `s`");

    let mut buf = [0; 2];
    std::io::stdin().read_exact(&mut buf)?;

    Ok((image, buf[0] == b'y'))
}

fn save_make_show_image(text: &str) -> Result<(), Error> {
    let shown = show_image(text, make_image(text)?)?;
    if shown.1 {
        shown.0.save(format!("out/{text}.bmp"))?;
    }
    Ok(())
}

fn save_show_image(text: &str, image: Image) -> Result<(), Error> {
    let shown = show_image(text, image)?;
    if shown.1 {
        shown.0.save(format!("out/{text}.bmp"))?;
    }
    Ok(())
}
