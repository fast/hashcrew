// Copyright 2026 FastLabs Developers
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-env-changed=RUSTC");
    println!("cargo:rustc-check-cfg=cfg(crc_vpclmulqdq)");

    // VPCLMULQDQ intrinsics stabilized after our MSRV. Older compilers keep
    // the PCLMULQDQ and scalar kernels without needing a build dependency.
    let version = std::env::var_os("RUSTC")
        .and_then(|rustc| {
            std::process::Command::new(rustc)
                .arg("--version")
                .output()
                .ok()
        })
        .filter(|output| output.status.success())
        .and_then(|output| String::from_utf8(output.stdout).ok())
        .and_then(|output| {
            let mut version = output.split_whitespace().nth(1)?.split('.');
            let major = version.next()?.parse::<u32>().ok()?;
            let minor = version.next()?.parse::<u32>().ok()?;
            Some((major, minor))
        });
    if version.is_some_and(|version| version >= (1, 89)) {
        println!("cargo:rustc-cfg=crc_vpclmulqdq");
    }
}
