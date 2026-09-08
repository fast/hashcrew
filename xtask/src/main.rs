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

use std::path::Path;
use std::process::Command as StdCommand;

use clap::Parser;
use clap::Subcommand;

const PACKAGE_NAME: &str = "hashcrew";

#[derive(Parser)]
struct Command {
    #[command(subcommand)]
    sub: SubCommand,
}

impl Command {
    fn run(self) {
        match self.sub {
            SubCommand::Bench(cmd) => cmd.run(),
            SubCommand::Build(cmd) => cmd.run(),
            SubCommand::Check(cmd) => cmd.run(),
            SubCommand::Lint(cmd) => cmd.run(),
            SubCommand::Miri(cmd) => cmd.run(),
            SubCommand::Test(cmd) => cmd.run(),
        }
    }
}

#[derive(Subcommand)]
enum SubCommand {
    #[command(about = "Run workspace benchmarks.")]
    Bench(CommandBench),
    #[command(about = "Compile workspace packages.")]
    Build(CommandBuild),
    #[command(about = "Check hashcrew under its feature configurations.")]
    Check(CommandCheck),
    #[command(about = "Run workspace quality checks.")]
    Lint(CommandLint),
    #[command(about = "Check memory safety with Miri.")]
    Miri(CommandMiri),
    #[command(about = "Run workspace tests.")]
    Test(CommandTest),
}

#[derive(Parser)]
struct CommandBench {
    #[arg(value_name = "NAME", help = "Run only the named benchmark target.")]
    bench: Option<String>,

    #[arg(last = true, help = "Arguments passed to the benchmark harness.")]
    args: Vec<std::ffi::OsString>,
}

impl CommandBench {
    fn run(self) {
        let mut cmd = cargo();
        cmd.args(["bench", "--package", "benchmarks"]);
        if let Some(bench) = self.bench {
            cmd.args(["--bench", &bench]);
        }
        if !self.args.is_empty() {
            cmd.arg("--").args(self.args);
        }
        run_command(cmd);
    }
}

#[derive(Parser)]
struct CommandBuild {
    #[arg(long, help = "Assert that `Cargo.lock` will remain unchanged.")]
    locked: bool,
}

impl CommandBuild {
    fn run(self) {
        let mut cmd = cargo();
        // Windows locks the running xtask executable, which is already built.
        cmd.args([
            "build",
            "--workspace",
            "--exclude",
            env!("CARGO_PKG_NAME"),
            "--all-features",
            "--tests",
            "--examples",
            "--benches",
            "--bins",
        ]);
        if self.locked {
            cmd.arg("--locked");
        }
        run_command(cmd);
    }
}

#[derive(Parser)]
struct CommandCheck {
    #[arg(
        long,
        value_name = "TRIPLE",
        help = "Check the library for this target."
    )]
    target: Option<String>,

    #[arg(long, help = "Check only configurations without std.")]
    no_std: bool,

    #[arg(
        long,
        value_name = "FLAGS",
        allow_hyphen_values = true,
        help = "Additional rustc flags for the checked library."
    )]
    rustflags: Option<String>,
}

impl CommandCheck {
    fn run(self) {
        let families = family_features();
        for with_std in [false, true] {
            if with_std && self.no_std {
                continue;
            }
            self.check(&[], with_std);
            for family in families.chunks(1) {
                self.check(family, with_std);
            }
            self.check(&families, with_std);
        }
    }

    fn check(&self, features: &[String], with_std: bool) {
        let mut cmd = cargo();
        let mut rustflags = std::env::var_os("RUSTFLAGS").unwrap_or_default();
        rustflags.push(" -D warnings");
        if let Some(flags) = &self.rustflags {
            rustflags.push(" ");
            rustflags.push(flags);
        }
        cmd.env("RUSTFLAGS", rustflags);
        cmd.args(["check", "--package", PACKAGE_NAME, "--no-default-features"]);
        if let Some(target) = &self.target {
            cmd.args(["--target", target]);
        } else {
            cmd.arg("--all-targets");
        }
        for feature in features {
            cmd.args(["--features", feature]);
        }
        if with_std {
            cmd.args(["--features", "std"]);
        }
        run_command(cmd);
    }
}

#[derive(Parser)]
struct CommandMiri;

impl CommandMiri {
    fn run(self) {
        let mut cmd = cargo();
        // Release mode keeps debug assertions from masking unsafe precondition violations.
        cmd.args([
            "+nightly",
            "miri",
            "test",
            "--package",
            PACKAGE_NAME,
            "--lib",
            "--no-default-features",
            "--release",
        ]);
        cmd.args(["--features", &family_features().join(",")]);
        run_command(cmd);
    }
}

#[derive(Parser)]
struct CommandTest {
    #[arg(long, help = "Run tests serially and do not capture output.")]
    no_capture: bool,
}

impl CommandTest {
    fn run(self) {
        let mut workspace = cargo();
        workspace.args(["test", "--workspace", "--all-features"]);
        add_test_output_args(&mut workspace, self.no_capture);
        run_command(workspace);

        let mut no_std = cargo();
        no_std.args(["test", "--package", PACKAGE_NAME, "--no-default-features"]);
        no_std.args(["--features", &family_features().join(",")]);
        add_test_output_args(&mut no_std, self.no_capture);
        run_command(no_std);

        let mut optimized = cargo();
        optimized.args(["test", "--package", "tests-integration", "--release"]);
        add_test_output_args(&mut optimized, self.no_capture);
        run_command(optimized);
    }
}

#[derive(Parser)]
#[command(name = "lint")]
struct CommandLint {
    #[arg(long, help = "Automatically apply lint and formatting suggestions.")]
    fix: bool,
}

impl CommandLint {
    fn run(self) {
        run_command(make_clippy_cmd(self.fix));
        run_command(make_format_cmd(self.fix));
        run_command(make_taplo_cmd(self.fix));
        run_command(make_typos_cmd());
        run_command(make_hawkeye_cmd(self.fix));
        run_command(make_doc_cmd());
        for family in family_features() {
            let mut cmd = make_doc_cmd();
            cmd.args(["--features", &family]);
            run_command(cmd);
        }
        let mut docsrs = make_doc_cmd();
        docsrs.env("RUSTDOCFLAGS", "-D warnings --cfg docsrs");
        docsrs.arg("--all-features");
        run_command(docsrs);
    }
}

fn find_command(command: &str) -> StdCommand {
    match which::which(command) {
        Ok(executable) => {
            let mut cmd = StdCommand::new(executable);
            cmd.current_dir(Path::new(env!("CARGO_WORKSPACE_DIR")));
            cmd
        }
        Err(err) => panic!("{command} not found: {err}"),
    }
}

fn ensure_installed(binary: &str, crate_name: &str) {
    if which::which(binary).is_err() {
        let mut cmd = cargo();
        cmd.args(["install", crate_name]);
        run_command(cmd);
    }
}

fn cargo() -> StdCommand {
    find_command("cargo")
}

fn run_command(mut cmd: StdCommand) {
    println!("{cmd:?}");
    let status = cmd.status().expect("failed to execute process");
    assert!(status.success(), "command failed: {status}");
}

fn add_test_output_args(cmd: &mut StdCommand, no_capture: bool) {
    if no_capture {
        cmd.args(["--", "--nocapture", "--test-threads=1"]);
    }
}

fn family_features() -> Vec<String> {
    let metadata = cargo_metadata::MetadataCommand::new()
        .manifest_path(Path::new(env!("CARGO_WORKSPACE_DIR")).join("Cargo.toml"))
        .no_deps()
        .exec()
        .expect("failed to read workspace metadata");
    let package = metadata
        .packages
        .into_iter()
        .find(|package| package.name == PACKAGE_NAME)
        .expect("failed to find hashcrew package");
    package
        .features
        .into_keys()
        .filter(|feature| !matches!(feature.as_str(), "default" | "std"))
        .collect()
}

fn make_format_cmd(fix: bool) -> StdCommand {
    let mut cmd = cargo();
    cmd.args(["+nightly", "fmt", "--all"]);
    if !fix {
        cmd.args(["--", "--check"]);
    }
    cmd
}

fn make_clippy_cmd(fix: bool) -> StdCommand {
    let mut cmd = cargo();
    cmd.args([
        "+nightly",
        "clippy",
        "--workspace",
        "--all-targets",
        "--all-features",
    ]);
    if fix {
        cmd.args(["--allow-staged", "--allow-dirty", "--fix"]);
    } else {
        cmd.args(["--", "-D", "warnings"]);
    }
    cmd
}

fn make_hawkeye_cmd(fix: bool) -> StdCommand {
    ensure_installed("hawkeye", "hawkeye");
    let mut cmd = find_command("hawkeye");
    if fix {
        cmd.arg("format");
    } else {
        cmd.arg("check");
    }
    cmd
}

fn make_typos_cmd() -> StdCommand {
    ensure_installed("typos", "typos-cli");
    find_command("typos")
}

fn make_taplo_cmd(fix: bool) -> StdCommand {
    ensure_installed("taplo", "taplo-cli");
    let mut cmd = find_command("taplo");
    if fix {
        cmd.arg("format");
    } else {
        cmd.args(["format", "--check"]);
    }
    cmd
}

fn make_doc_cmd() -> StdCommand {
    let mut cmd = cargo();
    cmd.env("RUSTDOCFLAGS", "-D warnings");
    cmd.args([
        "+nightly",
        "doc",
        "--package",
        PACKAGE_NAME,
        "--no-default-features",
        "--no-deps",
    ]);
    cmd
}

fn main() {
    Command::parse().run();
}
