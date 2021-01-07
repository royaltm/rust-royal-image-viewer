#!/bin/bash
TARGET=target/armv7-unknown-linux-gnueabihf/release/riv
rpxc cargo build --release 
ls -l $TARGET
rpxc arm-linux-gnueabihf-strip $TARGET
ls -l $TARGET
