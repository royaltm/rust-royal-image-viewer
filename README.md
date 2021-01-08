Royal Image Viewer
==================

This is the image viewer we need and deserve but never have before (probably).

```
RoyalImageViewer 1.0
Rafał Michalski

USAGE:
    riv [OPTIONS] [file]

FLAGS:
        --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -c, --color <color>      background color
    -h, --height <height>    window height
    -w, --width <width>      window width
    -x, --xwin <xwin>        horizontal window position
    -y, --ywin <ywin>        vertical window position

ARGS:
    <file>    An image file to display
```

The image is displayed centered until the program is killed or ESC key is pressed.

Compiling
---------

On Windows and OSX no extra packages are needed.

On Linux:

APT:

* `libxkbcommon-dev`
* `libwayland-dev`

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
