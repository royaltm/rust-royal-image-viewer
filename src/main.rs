use minifb::{Key, Window, WindowOptions};
use clap::clap_app;
use css_color_parser::Color as CssColor;

mod images;
mod utils;

use utils::Result;

const DEFAULT_WIDTH: usize = 1920;
const DEFAULT_HEIGHT: usize = 1080;

fn run() -> Result<()> {
    utils::set_dpi_awareness()?;

    let matches = clap_app!(RoyalImageViewer =>
        (version: "1.0")
        (author: "Rafał Michalski")
        // (about: HEAD)
        (@arg xwin: -x --xwin +takes_value "horizontal window position")
        (@arg ywin: -y --ywin +takes_value "vertical window position")
        (@arg width: -w --width +takes_value "window width")
        (@arg height: -h --height +takes_value "window height")
        (@arg color: -c --color +takes_value "background color")
        (@arg FILE: [file] "An image file to display")
    ).get_matches();

    let width: usize = matches.value_of("width").map(|v| v.parse()).transpose()
                            .map_err(|_| "width must be a positive integer")?
                            .unwrap_or(DEFAULT_WIDTH);
    let height: usize = matches.value_of("height").map(|v| v.parse()).transpose()
                            .map_err(|_| "height must be a positive integer")?
                            .unwrap_or(DEFAULT_HEIGHT);
    let xwin: isize = matches.value_of("xwin").map(|v| v.parse()).transpose()
                            .map_err(|_| "xwin must be an integer")?
                            .unwrap_or(0);
    let ywin: isize = matches.value_of("ywin").map(|v| v.parse()).transpose()
                            .map_err(|_| "ywin must be an integer")?
                            .unwrap_or(0);

    let color = matches.value_of("color")
                       .map(|c| c.parse::<CssColor>()).transpose()
                       .map_err(|_| "couldn't recognize a color name")?
                       .map(|CssColor { r, g, b, .. }| images::from_u8_rgb(r, g, b))
                       .unwrap_or(0);

    let mut buffer: Vec<u32> = vec![color; width * height];

    if let Some(name) = matches.value_of("FILE") {
        images::load_image_center_into(name, width as u32, height as u32, buffer.as_mut())?;
    }

    let mut opts = WindowOptions::default();
    opts.none = true;

    let mut window = Window::new(
        "Royal Image Viewer",
        width,
        height,
        opts,
    )?;

    window.set_position(xwin, ywin);
    window.set_cursor_visibility(false);
    // Limit to max ~60 fps update rate
    window.limit_update_rate(Some(std::time::Duration::from_micros(16600)));
    window.update_with_buffer(&buffer, width, height)?;

    while window.is_open() && !window.is_key_down(Key::Escape) {
        window.update();
    }

    Ok(())
}

fn main() -> Result<()> {
    std::process::exit(match run() {
        Ok(..) => 0,
        Err(err) => {
            eprintln!("Sorry, but {}.", err);
            1
        }
    });
}
