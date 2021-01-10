// #![windows_subsystem = "windows"] // it is "console" by default
use core::time::Duration;
use std::sync::mpsc::TryRecvError;
use env_logger::Env;
use log::debug;
use minifb::{Key, Window, WindowOptions};
use std::process::{Command, Stdio};

mod opts;
mod images;
mod remote;
mod utils;

use utils::{Result, ExitError, err_code};
use opts::*;

fn run() -> Result<()> {
    env_logger::Builder::from_env(Env::default().default_filter_or("error")).init();

    let matches = clap::App::new("Royal Image Viewer")
        .version("2.0")
        .author("Rafał Michalski")
        .about("Displays a centered image in a window of a size and position of your choosing.")
        .app_args()
        .after_help(
            "RIV acts as a server and listens on a UDP socket when displaying a window.\n\
            Before doing so, RIV sends a command to the UDP socket to load an image into\n\
            an existing window. A new window appears only if the server doesn't respond\n\
            within the timeout. To disable sending a command at all, set the timeout to 0.")
        .get_matches();
    let cfg = Config::new(&matches)?;
    let Config { opt_name, color, width, height, .. } = cfg;

    debug!("{:?}", cfg);

    if cfg.mswin_free_console {
        utils::free_console_window();
    }

    // check remote if file
    if let Some(name) = opt_name {
        let timeout = Duration::from_secs(cfg.timeout);
        if let Some(res) = remote::send((cfg.remote, cfg.port),
                                        (cfg.bind, 0),
                                        timeout, cfg.color, name)? {
            return if res {
                Ok(())
            }
            else {
                err_code("the remote process failed to load the image", 2)
            }
        }
        if cfg.fail {
            return err_code("the remote process failed to respond in time", 3)
        }
    }
    else if cfg.fail {
        return Err("no image file name was specified".into())
    }

    if cfg.detach {
        let pid = daemonize_with(cfg)?;
        println!("{}", pid);
        return Ok(())
    }

    // allocate buffer
    let mut buffer: Vec<u32> = vec![color; width * height];

    // bind socket
    let recv = remote::bind((cfg.bind, cfg.port), width as u32, height as u32, cfg.info)?;

    // load image if file
    if let Some(name) = opt_name {
        images::load_image_center_into(
            name, color, width as u32, height as u32, buffer.as_mut(), cfg.info)?;
    }

    // open window
    utils::set_dpi_awareness()?;

    let mut winopts = WindowOptions::default();
    winopts.none = true;

    let mut window = Window::new(
        "Royal Image Viewer",
        width,
        height,
        winopts,
    )?;

    window.set_position(cfg.xwin, cfg.ywin);
    window.set_cursor_visibility(false);
    // Limit to max ~60 fps update rate
    window.limit_update_rate(Some(std::time::Duration::from_micros(1_000_000 / 60)));
    // Draw a buffer with preloaded image
    window.update_with_buffer(&buffer, width, height)?;

    while window.is_open() && (cfg.nkey || !window.is_key_down(Key::Escape)) {
        match recv.try_recv() {
            Ok((color, img)) => {
                debug!("drawing image with: {:06x}", color);
                images::center_image_into(&img, color, width as u32, height as u32, &mut buffer);
                window.update_with_buffer(&buffer, width, height)?;
            }
            Err(TryRecvError::Empty) => window.update(),
            Err(TryRecvError::Disconnected) => break
        }
    }

    Ok(())
}

fn daemonize_with(mut cfg: Config) -> Result<u32> {
    cfg.timeout = 0;
    cfg.detach = false;
    cfg.mswin_free_console = true;
    Command::new(std::env::args().next().unwrap())
       .args_from(&cfg)
       .stdin(Stdio::null())
       .stdout(Stdio::inherit())
       .stderr(Stdio::inherit())
       .spawn()
       .map_err(|e| e.into())
       .map(|child| child.id())
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
