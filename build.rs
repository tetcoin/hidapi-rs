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

// Copy-pasted from `https://github.com/paritytech/parity-ethereum/blob/cb03f380ab2bb37ff18771e6886c42098ad8b15a/docker/android/Dockerfile#L44-L62`
const ANDROID_CFG: [&str; 5] = [
	"git checkout 83d918449f22720d84a341a05e24b6d109e6d3ae",
	"git apply ../libudev.patch",
	"./autogen.sh",
	"./configure --disable-introspection --disable-programs --disable-hwdb --host=arm-linux-androideabi --prefix=/opt/ndk-standalone/sysroot/usr/ --enable-shared=false",
	"make"
];

const LINUX_CFG: [&str; 3] = [
	"./autogen.sh",
	"./configure",
	"make"
];

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
		env::set_var("CFLAGS", "-D LINE_MAX=2048 -D RLIMIT_NLIMITS=15 -D IPTOS_LOWCOST=2 -std=gnu99");
		env::set_var("CXX", "arm-linux-androideabi-clang++");
		execute_shell_cmd(&ANDROID_CFG);
		let mut config = cc::Build::new();
		config.file("etc/hidapi/linux/hid.c").include("etc/hidapi/hidapi");
		config.compile("libhidapi.a");

		println!("cargo:rustc-link-search=native=./etc/eudev/src/libudev/.libs");
		println!("cargo:rustc-link-lib=static=udev");
	} else if target.contains("linux") {
		execute_shell_cmd(&LINUX_CFG);
		let mut config = cc::Build::new();
		config.file("etc/hidapi/linux/hid.c").include("etc/hidapi/hidapi");
		config.compile("libhidapi.a");

		println!("cargo:rustc-link-search=native=./etc/eudev/src/libudev/.libs");
		println!("cargo:rustc-link-lib=static=udev");
	}
}

fn execute_shell_cmd(commands: &[&str]) {
	let start = std::env::current_dir().expect("Couldn't fetch current directory");
	let target = std::path::Path::new(&start).join("etc/eudev");
	env::set_current_dir(target).expect("Could not find the directory: \"etc\\eudev\"");

	for full_cmd in commands.iter() {
		let ignore_error = full_cmd.starts_with("git") && full_cmd.contains("apply");

		let mut it = full_cmd.split_whitespace();
		let cmd = it.next().expect("A command should have at least one element; qed");

		let this_cmd = Command::new(cmd)
			.args(it)
			.status()
			.expect(&format!("Command {} failed", cmd));

	if !this_cmd.success() && !ignore_error {
			panic!("{}", this_cmd);
	} else if ignore_error {
			println!("IGNORED \"git apply error\" {}", this_cmd);
		}
	}

	env::set_current_dir(start).expect("Couldn't go back to start directory");
}
