use std::env;
use std::path::{Path, PathBuf};

use image::ImageReader;

fn main() {
    process_images();

    linker_be_nice();
    // make sure linkall.x is the last linker script (otherwise might cause problems with flip-link)
    println!("cargo:rustc-link-arg=-Tlinkall.x");
}

fn process_images() {
    println!("cargo:rerun-if-changed=assets/");

    let out_folder = PathBuf::from(env::var("OUT_DIR").unwrap());
    for entry in std::fs::read_dir(Path::new("assets/")).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        eprintln!("Processing {:?}", path);
        if path.extension().and_then(|ext| ext.to_str()) == Some("png") {
            let out_path = out_folder.join(format!("{}.raw", path.file_stem().unwrap().to_str().unwrap()));
            eprintln!("Creating {:?}", out_path);
            convert_to_raw_image(path, out_path);
        }
    }
}

fn convert_to_raw_image<P: AsRef<Path>>(in_path: P, out_path: P) {
    let in_path = in_path.as_ref();
    let image = ImageReader::open(in_path).unwrap().decode().unwrap().to_rgb8();

    let image_byte_size = ((image.width() * image.height()) * 2) as usize;
    let mut data = Vec::with_capacity(image_byte_size);
    eprintln!("Size {:?}", image_byte_size);
    for pixel in image.pixels() {
        let r = pixel.0[0];
        let g = pixel.0[1];
        let b = pixel.0[2];

        // Map to Rgb565
        let r2 = ((r as f32 / 255.0) * 31.0) as u16;
        let g2 = ((g as f32 / 255.0) * 63.0) as u16;
        let b2 = ((b as f32 / 255.0) * 31.0) as u16;

        let pixel_rgb565 = (b2 & 31) | ((g2 & 63) << 5) | (r2 & 31) << 11;

        data.push(((pixel_rgb565 >> 8) & 0xFF) as u8);
        data.push((pixel_rgb565 & 0xFF) as u8);
    }
    /*let image_byte_size = ((image.width() as f32 * image.height() as f32) / 8.0).ceil() as usize;
    eprintln!("Size {:?}", image_byte_size);
    let mut data = vec![0u8; image_byte_size];
    let mut bit = 7;
    let mut i = 0;
    for pixel in image.as_raw() {
        let value = if *pixel > 127 { 1 } else { 0 };
        data[i] |= value << bit;
        if bit > 0 {
            bit -= 1;
        } else {
            bit = 7;
            i += 1;
        }
    }*/

    std::fs::write(out_path, &data).unwrap();
}

fn linker_be_nice() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 {
        let kind = &args[1];
        let what = &args[2];

        match kind.as_str() {
            "undefined-symbol" => match what.as_str() {
                "_defmt_timestamp" => {
                    eprintln!();
                    eprintln!("💡 `defmt` not found - make sure `defmt.x` is added as a linker script and you have included `use defmt_rtt as _;`");
                    eprintln!();
                }
                "_stack_start" => {
                    eprintln!();
                    eprintln!("💡 Is the linker script `linkall.x` missing?");
                    eprintln!();
                }
                "esp_wifi_preempt_enable" | "esp_wifi_preempt_yield_task" | "esp_wifi_preempt_task_create" => {
                    eprintln!();
                    eprintln!("💡 `esp-wifi` has no scheduler enabled. Make sure you have the `builtin-scheduler` feature enabled, or that you provide an external scheduler.");
                    eprintln!();
                }
                "embedded_test_linker_file_not_added_to_rustflags" => {
                    eprintln!();
                    eprintln!("💡 `embedded-test` not found - make sure `embedded-test.x` is added as a linker script for tests");
                    eprintln!();
                }
                _ => (),
            },
            // we don't have anything helpful for "missing-lib" yet
            _ => {
                std::process::exit(1);
            }
        }

        std::process::exit(0);
    }

    println!(
        "cargo:rustc-link-arg=-Wl,--error-handling-script={}",
        std::env::current_exe().unwrap().display()
    );
}
