# fpr-cli-util

Additional utilities for building command-line tools with
[`fpr-cli`](https://github.com/shinjitumala/fpr-cli): the entry points (`run`,
`rund`), the `version!` macro, and TOML config helpers. It re-exports
`fpr-cli`, so a binary needs one dependency.

# Philosophy

- **The exit code is part of the interface.** A run that failed exits non-zero,
  so a shell script can test it, and a failing command cannot be mistaken for a
  successful one.
- **`main` is two lines.** The entry point owns the config path, `--version`,
  `--help`, and the error report; a tool writes only its commands.
- **One place decides the config path.** `$FPR_<PROJECT>_CONFIG_PATH` when set,
  otherwise `$HOME/<rel>`, which `run`/`rund` build and pass to `Ctx::new`.

# Motivating example

The `com` crate binds the framework and the config path once:

```rust
pub use fpr_cli_util::*;

const ENV: &str = "MYAPP_CONFIG_PATH";
const DEF: &str = ".config/myapp";

pub fn run_acts<M: Acts<C>, V: Args<C>, C: Ctx>() {
    fpr_cli_util::run::<_, M, V, _, _>(ENV, DEF);
}
```

The binary's `main` is the whole front end:

```rust
version!(C);

fn main() {
    myapp_com::run_acts::<Cmds, Version, C>();
}
```

A command that cannot read its config fails, and the shell sees it:

```console
$ myapp list
Failed to read '/home/u/.config/myapp/myapp.toml' because 'No such file or directory (os error 2)'.
$ echo $?
1
```

The success path, and the two invocations the framework answers itself:

```console
$ myapp list
item-a
item-b
$ echo $?
0
$ myapp --version >/dev/null; echo $?
0
$ myapp --help >/dev/null; echo $?
0
```

# Exit codes

| code | meaning |
| ---- | ------- |
| `0` | the command ran and returned `Ok`; also `--version` and `--help` |
| `1` | the run failed |

`1` covers every failure this crate can see: `Ctx::new` returned `Err` (a
missing or unparsable config), an argument failed to parse, an act was unknown
or missing, or a command returned `Err`. `run` and `rund` print the message to
stderr and then exit, so a tool must not print the same error in its command as
well.

A usage error is not distinguished from a runtime failure: both are `1`, and
the message names the cause. A tool that needs a distinct code has to exit
itself, before returning the `Err`.

Two consequences follow from where the exit happens:

- The process exits immediately, so destructors do not run. A command that owns
  a lock, a temporary file, or a child process must release it before returning
  `Err`.
- `--version` and `--help` still exit `1` when the context cannot be built:
  `Ctx::new` runs first, and both handlers take `&C`. A tool whose `--version`
  must work without a config cannot fail in `Ctx::new` when the config is
  absent — it has to fall back to a default context.

# Getting started

## Prerequisites

A Rust toolchain with edition 2024 support (`rustc` 1.85 or later) and
`cargo`. This is a library: it installs no binary.

## Install

```toml
[dependencies]
fpr-cli-util = "0.4"
```

# Public API

| item | what it does |
| ---- | ------------ |
| `run<C, M: Acts<C>, V: Args<C>, K: AsRef<OsStr>, P: AsRef<str>>(env, default_home_rel)` | sub-command entry point: resolves the config path, calls `C::new`, handles `--version`, dispatches the act tree, prints `--help`, reports a failure and exits |
| `rund<M: Args<C>, V: Args<C>, C: Ctx, K, P>(env, default_home_rel)` | single-command entry point: the same, for one `Args` struct plus `--help` |
| `trait Ctx` | `fn new(s: &str) -> Result<Self, String>`; `s` is the config path the entry point resolved, file or directory by the tool's own convention |
| `version!(C)` | defines `pub struct Version {}` from the package and git metadata, so `--version` exists; needs a `build.rs` calling `built::write_built_file()` |
| `reader(p)`, `from_toml(p)`, `write_toml(o, p)`, `write_ctoml(o, p)` | config file helpers: open, deserialize, and write TOML. Each `Err` names the path and the cause |
| re-exports | `pub use fpr_cli::*` — the `Args`/`Acts`/`Actions` derives and their types |

# Links

- [`fpr-cli`](https://github.com/shinjitumala/fpr-cli) — the derive-based
  `Args`/`Acts` model this crate re-exports and builds on.
- [`fpr-cli-test`](https://github.com/shinjitumala/fpr-cli-test) — a sample
  binary that exercises the derives.

# License

Apache-2.0.
