Royal Image Viewer
==================

This is the image viewer we need and deserve but never have before (probably).

```
Royal Image Viewer 2.0
Rafał Michalski
Displays a centered image in a window of a size and position of your choosing.

USAGE:
    riv.exe [FLAGS] [OPTIONS] [FILE]

FLAGS:
    -f, --fail       Exits after failing to contact the remote instance
        --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -b, --bind <ipaddr>        Specify UDP bind IP address [env: RIV_BIND_ADDR=]
    -c, --color <css>          Window background color [env: RIV_WINDOW_COLOR=]
    -h, --height <height>      Window height [env: RIV_WINDOW_HEIGH=]  [default: 1080]
    -p, --port <port>          Specify UDP port [env: RIV_PORT=]  [default: 9990]
    -r, --remote <ipaddr>      Remote instance IP address [env: RIV_REMOTE_ADDR=]
    -t, --timeout <seconds>    Remote instance respond timeout [env: RIV_TIMEOUT=]
    -w, --width <width>        Window width [env: RIV_WINDOW_WIDTH=]  [default: 1920]
    -x, --xwin <xwin>          Horizontal window position [env: RIV_WINDOW_X=]
    -y, --ywin <ywin>          Vertical window position [env: RIV_WINDOW_Y=]

ARGS:
    <FILE>    An image file to display
```

The RIV window is displayed until the program is killed or ESC key is pressed.

* Without a `FILE` argument, displays a window filled with a black or a provided color.
* When called with a `FILE` argument, displays a centered image on a window with a black (or a provided color) background.
* While displaying a window, listens to UDP commands on localhost or a provided `bind` IP address.
* Before displaying a window, attempts to send a command to an existing instance of RIV via UDP messages to load another image and change the background color. This initial attempt can be disabled by setting `timeout` to 0.
* Only after a failed attempt to contact another instance of RIV, its own window is displayed and the image is loaded.
* To prevent RIV from displaying its own window at all, provide an `-f` switch.

The following environment variables can be set to override defaults:

```
RIV_WINDOW_COLOR=black
RIV_WINDOW_WIDTH=1920
RIV_WINDOW_HEIGH=1080
RIV_WINDOW_X=0
RIV_WINDOW_Y=0
RIV_PORT=9990
RIV_REMOTE_ADDR=localhost
RIV_BIND_ADDR=localhost
RIV_TIMEOUT=2
```


Compiling
---------

On Windows and OSX no extra packages are needed.

On Linux:

APT:

* `libwayland-dev`
* `libxkbcommon-dev`

RPM:

* `wayland-devel`
* `libxkbcommon-devel`

Pacman:

* `wayland`
* `libxkbcommon`


Cross compiling
---------------

Install Rust with [rustup](https://rustup.rs/) and all cross-compiling tools you need with systemroot.

Make sure `pkgconf` package is installed in $SYSROOT.

To help guide rust [pkg_config](https://crates.io/crates/pkg-config) for cross-compiling:

```
ENV TARGET_PKG_CONFIG_PATH=
ENV TARGET_PKG_CONFIG_SYSROOT_DIR=$SYSROOT
ENV TARGET_PKG_CONFIG_LIBDIR=$SYSROOT/usr/lib/pkgconfig:$SYSROOT/usr/share/pkgconfig:$SYSROOT/usr/lib/$ARCH/pkgconfig
```

where `$ARCH` is the linux target (e.g. `arm-linux-gnueabihf`) and `$SYSROOT` is your target system image.


### Raspbian

Required arm-linux-gnueabihf packages in `$SYSROOT`:

* `libxkbcommon-dev`
* `libwayland-dev`

Add target:

```
rustup target add armv7-unknown-linux-gnueabihf
```

Compile with:

```
cargo build --release --target=armv7-unknown-linux-gnueabihf
```

Get exe file from `target/armv7-unknown-linux-gnueabihf/release/riv`.


### ARM 64-bit

Packages:

* pkgconf
* xkbcommon
* wayland

Add target:

```
rustup target add aarch64-unknown-linux-gnu
```

See https://doc.rust-lang.org/rustc/platform-support.html

Compile with:

```
cargo build --release --target=aarch64-unknown-linux-gnu
```

Get exe file from `target/aarch64-unknown-linux-gnu/release/riv`.
