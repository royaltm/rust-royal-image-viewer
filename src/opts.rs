use clap::Arg;
use css_color_parser::Color as CssColor;
use std::process::Command;
use crate::images;
use crate::utils::Result;

const DEFAULT_ADDRESS: &str = "localhost";

pub trait AppArgs {
    fn app_args(self) -> Self;
}

impl<'a, 'b> AppArgs for clap::App<'a, 'b> {
    fn app_args(self) -> Self {
        self.arg(Arg::with_name("color").short("c").long("color").value_name("css")
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
        .arg(Arg::with_name("detach").short("d").long("detach")
            .help("Run window process in the background and print its PID"))
        .arg(Arg::with_name("nkey").short("K").long("no-key")
            .help("Do not exit after pressing ESC key"))
        .arg(Arg::with_name("info").short("i").long("info")
            .help("Prints information about the image"))
        .arg(Arg::with_name("mswinfreecons").long("mswin-free-console")
            .hidden(true))
        .arg(Arg::with_name("FILE")
            .help("An image file to display")
            .required(false))
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Config<'a> {
    pub opt_name: Option<&'a str>,
    pub xwin: isize,
    pub ywin: isize,
    pub height: usize,
    pub width: usize,
    pub color: u32,
    pub port: u16,
    pub timeout: u64,
    pub remote: &'a str,
    pub bind: &'a str,
    pub nkey: bool,
    pub fail: bool,
    pub detach: bool,
    pub info: bool,
    pub mswin_free_console: bool,
}

impl<'a> Config<'a> {
    pub fn new<'b: 'a>(matches: &'b clap::ArgMatches<'a>) -> Result<Config<'a>> {
        let fail = matches.is_present("fail");
        Ok(Config {
            width: matches.value_of("width").map(|v| v.parse()).transpose()
                                      .map_err(|_| "width must be a positive integer")?
                                      .unwrap(),
            height: matches.value_of("height").map(|v| v.parse()).transpose()
                                       .map_err(|_| "height must be a positive integer")?
                                       .unwrap(),
            xwin: matches.value_of("xwin").map(|v| v.parse()).transpose()
                                     .map_err(|_| "xwin must be an integer")?
                                     .unwrap_or(0),
            ywin: matches.value_of("ywin").map(|v| v.parse()).transpose()
                                     .map_err(|_| "ywin must be an integer")?
                                     .unwrap_or(0),
            color: matches.value_of("color")
                               .map(|c| c.parse::<CssColor>()).transpose()
                               .map_err(|_| "couldn't recognize a color name")?
                               .map(|CssColor { r, g, b, .. }| images::from_u8_rgb(r, g, b))
                               .unwrap_or(0),
            port: matches.value_of("port").map(|v| v.parse()).transpose()
                               .map_err(|_| "port must be an integer: 0 - 65535")?
                               .unwrap(),
            remote: matches.value_of("remote").unwrap_or_else(|| DEFAULT_ADDRESS),
            bind: matches.value_of("bind").unwrap_or_else(|| DEFAULT_ADDRESS),
            fail,
            nkey: matches.is_present("nkey"),
            detach: matches.is_present("detach"),
            info: matches.is_present("info"),
            mswin_free_console: matches.is_present("mswinfreecons"),
            timeout: matches.value_of("timeout").map(|v| v.parse()).transpose()
                                      .map_err(|_| "timeout must be a positive integer")?
                                      .unwrap_or_else(|| if fail { 5 } else { 1 }),
            opt_name: matches.value_of("FILE")
        })
    }
}

pub trait ArgsFrom {
    fn args_from(self, opts: &Config) -> Self;
}

impl ArgsFrom for Command {
    fn args_from(mut self, opts: &Config) -> Self {
        let mut arg_val = |opt, prop: &str| {
            self.arg(opt);
            self.arg(prop);
        };

        arg_val("-w", &opts.width.to_string());
        arg_val("-h", &opts.height.to_string());
        arg_val("-t", &opts.timeout.to_string());
        arg_val("-p", &opts.port.to_string());
        if opts.xwin != 0 {
            arg_val("-x", &opts.xwin.to_string());
        }
        if opts.ywin != 0 {
            arg_val("-y", &opts.ywin.to_string());
        }
        if opts.color != 0 {
            arg_val("-c", &format!("{:06x}", opts.color));
        }
        if opts.remote != DEFAULT_ADDRESS {
            arg_val("-r", opts.remote);
        }
        if opts.bind != DEFAULT_ADDRESS {
            arg_val("-b", opts.bind);
        }
        if opts.fail {
            self.arg("-f");
        }
        if opts.nkey {
            self.arg("-K");
        }
        if opts.detach {
            self.arg("-d");
        }
        if opts.info {
            self.arg("-i");
        }
        #[cfg(windows)]
        if opts.mswin_free_console {
            self.arg("--mswin-free-console");
        }
        if let Some(name) = opts.opt_name {
            self.arg(name);
        }
        self
    }
}
