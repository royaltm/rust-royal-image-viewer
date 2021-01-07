Royal Image Viewer
==================

This is the image viewer we need and deserve but never have before (probably).

```
RustImageViewer 1.0
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


Cross compiling
---------------

Install Rust with [rustup](https://rustup.rs/) and all cross-compiling tools you need with systemroot.

### Raspbian

Required raspbian packages in `$SYSROOT`:

* `libxkbcommon-dev`
* `libwayland-dev`

```
export XKBCOMMON_LIB_DIR=$SYSROOT/usr/lib/arm-linux-gnueabihf
```

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

* xkbcommon
* wayland

```
export XKBCOMMON_LIB_DIR=$SYSROOT/usr/lib/$arch
```

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
