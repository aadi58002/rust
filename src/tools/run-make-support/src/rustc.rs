use std::env;
use std::ffi::{OsStr, OsString};
use std::path::Path;
use std::process::{Command, Output};

use crate::{handle_failed_output, set_host_rpath, tmp_dir};

/// Construct a new `rustc` invocation.
pub fn rustc() -> Rustc {
    Rustc::new()
}

/// Construct a new `rustc` aux-build invocation.
pub fn aux_build() -> Rustc {
    Rustc::new_aux_build()
}

/// A `rustc` invocation builder.
#[derive(Debug)]
pub struct Rustc {
    cmd: Command,
}

fn setup_common() -> Command {
    let rustc = env::var("RUSTC").unwrap();
    let mut cmd = Command::new(rustc);
    set_host_rpath(&mut cmd);
    cmd.arg("--out-dir").arg(tmp_dir()).arg("-L").arg(tmp_dir());
    cmd
}

impl Rustc {
    // `rustc` invocation constructor methods

    /// Construct a new `rustc` invocation.
    pub fn new() -> Self {
        let cmd = setup_common();
        Self { cmd }
    }

    /// Construct a new `rustc` invocation with `aux_build` preset (setting `--crate-type=lib`).
    pub fn new_aux_build() -> Self {
        let mut cmd = setup_common();
        cmd.arg("--crate-type=lib");
        Self { cmd }
    }

    // Argument provider methods

    /// Configure the compilation environment.
    pub fn cfg(&mut self, s: &str) -> &mut Self {
        self.cmd.arg("--cfg");
        self.cmd.arg(s);
        self
    }

    /// Specify default optimization level `-O` (alias for `-C opt-level=2`).
    pub fn opt(&mut self) -> &mut Self {
        self.cmd.arg("-O");
        self
    }

    /// Specify type(s) of output files to generate.
    pub fn emit(&mut self, kinds: &str) -> &mut Self {
        self.cmd.arg(format!("--emit={kinds}"));
        self
    }

    /// Specify where an external library is located.
    pub fn extern_<P: AsRef<Path>>(&mut self, crate_name: &str, path: P) -> &mut Self {
        assert!(
            !crate_name.contains(|c: char| c.is_whitespace() || c == '\\' || c == '/'),
            "crate name cannot contain whitespace or path separators"
        );

        let path = path.as_ref().to_string_lossy();

        self.cmd.arg("--extern");
        self.cmd.arg(format!("{crate_name}={path}"));

        self
    }

    /// Specify path to the input file.
    pub fn input<P: AsRef<Path>>(&mut self, path: P) -> &mut Self {
        self.cmd.arg(path.as_ref());
        self
    }

    /// This flag defers LTO optimizations to the linker.
    pub fn linker_plugin_lto(&mut self, option: &str) -> &mut Self {
        self.cmd.arg(format!("-Clinker-plugin-lto={option}"));
        self
    }

    /// Specify what happens when the code panics.
    pub fn panic(&mut self, option: &str) -> &mut Self {
        self.cmd.arg(format!("-Cpanic={option}"));
        self
    }

    /// Specify number of codegen units
    pub fn codegen_units(&mut self, units: usize) -> &mut Self {
        self.cmd.arg(format!("-Ccodegen-units={units}"));
        self
    }

    /// Specify directory path used for incremental cache
    pub fn incremental<P: AsRef<Path>>(&mut self, path: P) -> &mut Self {
        let mut arg = OsString::new();
        arg.push("-Cincremental=");
        arg.push(path.as_ref());
        self.cmd.arg(&arg);
        self
    }

    /// Specify error format to use
    pub fn error_format(&mut self, format: &str) -> &mut Self {
        self.cmd.arg(format!("--error-format={format}"));
        self
    }

    /// Specify json messages printed by the compiler
    pub fn json(&mut self, items: &str) -> &mut Self {
        self.cmd.arg(format!("--json={items}"));
        self
    }

    /// Specify target triple.
    pub fn target(&mut self, target: &str) -> &mut Self {
        assert!(!target.contains(char::is_whitespace), "target triple cannot contain spaces");
        self.cmd.arg(format!("--target={target}"));
        self
    }

    /// Generic command argument provider. Use `.arg("-Zname")` over `.arg("-Z").arg("arg")`.
    pub fn arg<S: AsRef<OsStr>>(&mut self, arg: S) -> &mut Self {
        self.cmd.arg(arg);
        self
    }

    /// Specify the crate type.
    pub fn crate_type(&mut self, crate_type: &str) -> &mut Self {
        self.cmd.arg("--crate-type");
        self.cmd.arg(crate_type);
        self
    }

    /// Specify the edition year.
    pub fn edition(&mut self, edition: &str) -> &mut Self {
        self.cmd.arg("--edition");
        self.cmd.arg(edition);
        self
    }

    /// Generic command arguments provider. Use `.arg("-Zname")` over `.arg("-Z").arg("arg")`.
    pub fn args<S: AsRef<OsStr>>(&mut self, args: &[S]) -> &mut Self {
        self.cmd.args(args);
        self
    }

    pub fn env(&mut self, name: impl AsRef<OsStr>, value: impl AsRef<OsStr>) -> &mut Self {
        self.cmd.env(name, value);
        self
    }

    // Command inspection, output and running helper methods

    /// Get the [`Output`][std::process::Output] of the finished `rustc` process.
    pub fn output(&mut self) -> Output {
        self.cmd.output().unwrap()
    }

    /// Run the constructed `rustc` command and assert that it is successfully run.
    #[track_caller]
    pub fn run(&mut self) -> Output {
        let caller_location = std::panic::Location::caller();
        let caller_line_number = caller_location.line();

        let output = self.cmd.output().unwrap();
        if !output.status.success() {
            handle_failed_output(&format!("{:#?}", self.cmd), output, caller_line_number);
        }
        output
    }

    #[track_caller]
    pub fn run_fail(&mut self) -> Output {
        let caller_location = std::panic::Location::caller();
        let caller_line_number = caller_location.line();

        let output = self.cmd.output().unwrap();
        if output.status.success() {
            handle_failed_output(&format!("{:#?}", self.cmd), output, caller_line_number);
        }
        output
    }

    #[track_caller]
    pub fn run_fail_assert_exit_code(&mut self, code: i32) -> Output {
        let caller_location = std::panic::Location::caller();
        let caller_line_number = caller_location.line();

        let output = self.cmd.output().unwrap();
        if output.status.code().unwrap() != code {
            handle_failed_output(&format!("{:#?}", self.cmd), output, caller_line_number);
        }
        output
    }

    /// Inspect what the underlying [`Command`] is up to the current construction.
    pub fn inspect(&mut self, f: impl FnOnce(&Command)) -> &mut Self {
        f(&self.cmd);
        self
    }
}
