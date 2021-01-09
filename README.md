Royal Image Viewer
==================

This is the image viewer we need and deserve but never have before (probably).

```
Royal Image Viewer 2.0
Rafał Michalski
Displays a centered image in a window of a size and position of your choosing.

USAGE:
    riv [FLAGS] [OPTIONS] [FILE]

FLAGS:
    -f, --fail       Exits after failing to contact the remote instance
        --help       Prints help information
    -K, --no-key     Do not exit after pressing ESC key
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

The RIV window is displayed until the program is terminated or ESC key is pressed.

* Without a `FILE` argument, displays a window filled with a black or a provided `color`.
* While displaying a window, listens to UDP commands on localhost or a provided `bind` IP address.
* When called with a `FILE` argument, attempts to send a command to an existing RIV process via UDP messages to load and display another image and change the background color on an already opened window.
* After a failed attempt to contact another RIV process opens a window and displays a centered image.
* To load and display an image immediately without attempting to contact another process set the `timeout` to 0.
* To prevent RIV from displaying its window after a failed attempt to contact another RIV process use the `-f` switch.

### Examples

```
# displays image.jpg on a 1920x1080 window
# does not send a command to another instance before attempting to load an image
# listens on 9990 UDP port for commands
riv path/to/image.jpg -t 0

# opens 800x800 window positioned at 100x100, with olive background
# listens on 3333 UDP port for commands
riv --color olive -w 800 -h 800 -p 3333 -x 100 -y 100

# attempts to command another instance of RIV to show provided image on a #623 background
# exits after 4 seconds if another instance is not up and listening on port 9990
riv path/to/another/image.jpg -c '#623' -t 4 -f
```

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
RIV_TIMEOUT=1
```

To see debug messages set `RUST_LOG=debug`.


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
