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

const PACKAGE_NAME: &str = "rache";

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
    #[command(about = "Check rache under its feature configurations.")]
    Check(CommandCheck),
    #[command(about = "Run workspace quality checks.")]
    Lint(CommandLint),
    #[command(about = "Run workspace tests.")]
    Test(CommandTest),
}

#[derive(Parser)]
struct CommandBench {
    #[arg(long, help = "Compile benchmarks without running them.")]
    no_run: bool,
}

impl CommandBench {
    fn run(self) {
        let mut cmd = cargo();
        cmd.args(["bench", "--package", "benchmarks"]);
        if self.no_run {
            cmd.arg("--no-run");
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
        cmd.args([
            "build",
            "--workspace",
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
struct CommandCheck;

impl CommandCheck {
    fn run(self) {
        run_command(make_check_cmd(false));
        run_command(make_check_cmd(true));
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

fn make_check_cmd(all_features: bool) -> StdCommand {
    let mut cmd = cargo();
    cmd.env("RUSTFLAGS", "-D warnings");
    cmd.args([
        "check",
        "--package",
        PACKAGE_NAME,
        "--all-targets",
        "--no-default-features",
    ]);
    if all_features {
        cmd.arg("--all-features");
    }
    cmd
}

fn make_format_cmd(fix: bool) -> StdCommand {
    let mut cmd = cargo();
    cmd.args(["fmt", "--all"]);
    if !fix {
        cmd.args(["--", "--check"]);
    }
    cmd
}

fn make_clippy_cmd(fix: bool) -> StdCommand {
    let mut cmd = cargo();
    cmd.args(["clippy", "--workspace", "--all-targets", "--all-features"]);
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
        "doc",
        "--package",
        PACKAGE_NAME,
        "--all-features",
        "--no-deps",
    ]);
    cmd
}

fn main() {
    Command::parse().run();
}
