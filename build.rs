// **************************************************************************
// Copyright (c) 2015 Roland Ruckerbauer All Rights Reserved.
//
// This file is part of hidapi_rust.
//
// hidapi_rust is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// hidapi_rust is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with hidapi_rust.  If not, see <http://www.gnu.org/licenses/>.
// *************************************************************************

extern crate cc;

use std::process::Command;
use std::env;

const ANDROID_COMMANDS: [&str; 5] = [
    "git checkout 83d918449f22720d84a341a05e24b6d109e6d3ae",
    "./autogen.sh",
    "./configure --disable-introspection --disable-programs --disable-hwdb --host=arm-linux-androideabi --prefix=/opt/ndk-standalone/sysroot/usr/ --enable-shared=false CC=arm-linux-androideabi-clang CFLAGS=\"-D LINE_MAX=2048 -D RLIMIT_NLIMITS=15 -D IPTOS_LOWCOST=2 -std=gnu99\" CXX=arm-linux-androideabi-clang++",
    "git apply - < ../libudev.patch",
    "make"
];

const OTHER_LINUX: [&str; 3] = ["./autogen.sh", "./configure", "make"];

fn main() {
    let target = env::var("TARGET").unwrap();

    if target.contains("windows") {
        cc::Build::new()
            .file("etc/hidapi/windows/hid.c")
            .include("etc/hidapi/hidapi")
            .compile("libhidapi.a");
        println!("cargo:rustc-link-lib=setupapi");

    } else if target.contains("darwin") {
        cc::Build::new()
            .file("etc/hidapi/mac/hid.c")
            .include("etc/hidapi/hidapi")
            .compile("libhidapi.a");
        println!("cargo:rustc-link-lib=framework=IOKit");
        println!("cargo:rustc-link-lib=framework=CoreFoundation");

    } else if target.contains("android") {
        gnu_make(&ANDROID_COMMANDS);
        let mut config = cc::Build::new();
        config.file("etc/hidapi/linux/hid.c").include("etc/hidapi/hidapi");
        config.compile("libhidapi.a");

        println!("cargo:rustc-link-search=native=./etc/eudev/src/libudev/.libs");
        println!("cargo:rustc-link-lib=static=udev");
    } else if target.contains("linux") {
        gnu_make(&OTHER_LINUX);
        let mut config = cc::Build::new();
        config.file("etc/hidapi/linux/hid.c").include("etc/hidapi/hidapi");
        config.compile("libhidapi.a");

        println!("cargo:rustc-link-search=native=./etc/eudev/src/libudev/.libs");
        println!("cargo:rustc-link-lib=static=udev");
    }
}


fn gnu_make(commands: &[&str]) {
    let start = std::env::current_dir().expect("Couldn't fetch current directory");
    let target = std::path::Path::new(&start).join("etc/eudev");
    env::set_current_dir(target).expect("Could not find the directory: \"etc\\eudev\"");

    for c in commands.iter() {
        let success = Command::new(c)
            .status()
            .expect("command failed")
            .success();
        assert!(success, "Command: {:?} failed");
    }

    env::set_current_dir(start).expect("Couldn't go back to start directory");
}
