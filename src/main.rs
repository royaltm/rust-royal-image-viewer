use core::time::Duration;
use std::sync::mpsc::TryRecvError;
use clap::Arg;
use css_color_parser::Color as CssColor;
use env_logger::Env;
use log::debug;
use minifb::{Key, Window, WindowOptions};

mod images;
mod remote;
mod utils;

use utils::{Result, ExitError, err_code};

fn run() -> Result<()> {
    env_logger::Builder::from_env(Env::default().default_filter_or("error")).init();
    let matches = clap::App::new("Royal Image Viewer")
                .version("2.0")
                .author("Rafał Michalski")
                .about("Displays a centered image in a window of a size and position of your choosing.")
                .arg(Arg::with_name("color").short("c").long("color").value_name("css")
                         .env("RIV_WINDOW_COLOR")
                         .help("Window background color")
                         .takes_value(true))
                .arg(Arg::with_name("xwin").short("x").long("xwin")
                         .env("RIV_WINDOW_X")
                         .help("Horizontal window position")
                         .takes_value(true))
                .arg(Arg::with_name("ywin").short("y").long("ywin")
                         .env("RIV_WINDOW_Y")
                         .help("Vertical window position")
                         .takes_value(true))
                .arg(Arg::with_name("width").short("w").long("width")
                         .env("RIV_WINDOW_WIDTH")
                         .help("Window width")
                         .default_value("1920"))
                .arg(Arg::with_name("height").short("h").long("height")
                         .env("RIV_WINDOW_HEIGH")
                         .help("Window height")
                         .default_value("1080"))
                .arg(Arg::with_name("port").short("p").long("port")
                         .env("RIV_PORT")
                         .help("Specify UDP port")
                         .default_value("9990"))
                .arg(Arg::with_name("bind").short("b").long("bind").value_name("ipaddr")
                         .env("RIV_BIND_ADDR")
                         .help("Specify UDP bind IP address")
                         .takes_value(true))
                .arg(Arg::with_name("remote").short("r").long("remote").value_name("ipaddr")
                         .env("RIV_REMOTE_ADDR")
                         .help("Remote process IP address")
                         .takes_value(true))
                .arg(Arg::with_name("timeout").short("t").long("timeout").value_name("seconds")
                         .env("RIV_TIMEOUT")
                         .help("Remote process respond timeout")
                         .takes_value(true))
                .arg(Arg::with_name("fail").short("f").long("fail")
                         .help("Exits after failing to contact the remote process"))
                .arg(Arg::with_name("nkey").short("K").long("no-key")
                         .help("Do not exit after pressing ESC key"))
                .arg(Arg::with_name("FILE")
                               .help("An image file to display")
                               .required(false))
                .get_matches();

    let width: usize = matches.value_of("width").map(|v| v.parse()).transpose()
                              .map_err(|_| "width must be a positive integer")?
                              .unwrap();
    let height: usize = matches.value_of("height").map(|v| v.parse()).transpose()
                               .map_err(|_| "height must be a positive integer")?
                               .unwrap();
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
    let port: u16 = matches.value_of("port").map(|v| v.parse()).transpose()
                           .map_err(|_| "port must be an integer: 0 - 65535")?
                           .unwrap();
    let remote = (matches.value_of("remote").unwrap_or_else(|| "localhost"), port);
    let bind = (matches.value_of("bind").unwrap_or_else(|| "localhost"), port);
    let fail = matches.is_present("fail");
    let nkey = matches.is_present("nkey");
    let timeout: u64 = matches.value_of("timeout").map(|v| v.parse()).transpose()
                              .map_err(|_| "timeout must be a positive integer")?
                              .unwrap_or_else(|| if fail { 5 } else { 1 });
    let opt_name = matches.value_of("FILE");

    // check remote if file
    if let Some(name) = opt_name {
        let timeout = Duration::from_secs(timeout);
        if let Some(res) = remote::send(remote, (bind.0, 0), timeout, color, name)? {
            return if res {
                Ok(())
            }
            else {
                err_code("the remote process failed to load the image", 2)
            }
        }
        if fail {
            return err_code("the remote process failed to respond in time", 3)
        }
    }
    else if fail {
        return Err("no image file name was specified".into())
    }

    // allocate buffer
    let mut buffer: Vec<u32> = vec![color; width * height];

    // bind socket
    let recv = remote::bind(bind, width as u32, height as u32)?;

    // load image if file
    if let Some(name) = opt_name {
        images::load_image_center_into(name, color, width as u32, height as u32, buffer.as_mut())?;
    }

    // open window
    utils::set_dpi_awareness()?;

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
    window.limit_update_rate(Some(std::time::Duration::from_micros(1_000_000 / 60)));
    // Draw a buffer with preloaded image
    window.update_with_buffer(&buffer, width, height)?;

    while window.is_open() && (nkey || !window.is_key_down(Key::Escape)) {
        match recv.try_recv() {
            Ok((color, img)) => {
                debug!("drawing image");
                images::center_image_into(&img, color, width as u32, height as u32, &mut buffer);
                window.update_with_buffer(&buffer, width, height)?;
            }
            Err(TryRecvError::Empty) => window.update(),
            Err(TryRecvError::Disconnected) => break
        }
    }

    Ok(())
}

fn main() -> Result<()> {
    std::process::exit(match run() {
        Ok(..) => 0,
        Err(err) => {
            eprintln!("Sorry, but {}.", err);
            if let Some(eerr) = err.downcast_ref::<ExitError>() {
                eerr.exit_code()
            }
            else {
                1
            }
        }
    });
}
