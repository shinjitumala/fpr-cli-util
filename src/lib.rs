use core::fmt;
pub use fpr_cli::*;
use std::{
    env::{args, var},
    ffi::OsStr,
    fs::{self, File},
    io::{BufReader, Read},
    panic::Location,
    path::Path,
};

#[macro_export]
macro_rules! version {
    ($c:ty) => {
        mod build {
            include!(concat!(env!("OUT_DIR"), "/built.rs"));
        }
        #[derive(Args)]
        #[args(f = version, desc = "Print version.")]
        pub struct Version {}
        fn version(_: &$c, _: Version) -> Result<(), String> {
            use build::*;
            println!(
                "{} v{PKG_VERSION}: {TARGET}, {PROFILE}",
                env!("CARGO_PKG_NAME"),
            );
            if !PKG_DESCRIPTION.is_empty() {
                println!("{PKG_DESCRIPTION}");
            }
            if !PKG_HOMEPAGE.is_empty() {
                println!("Homeage: {PKG_HOMEPAGE}");
            }
            if !PKG_LICENSE.is_empty() {
                println!("License: {PKG_LICENSE}");
            }
            if !PKG_AUTHORS.is_empty() {
                println!("Authors: {PKG_AUTHORS}");
            }
            if !PKG_REPOSITORY.is_empty() {
                println!(
                    "Repository: {PKG_REPOSITORY}, commit = {}, dirty = {}",
                    GIT_COMMIT_HASH_SHORT.unwrap_or("none"),
                    GIT_DIRTY
                        .map(|e| e.to_string())
                        .unwrap_or("none".to_string()),
                );
            }
            Ok(())
        }
    };
}

pub trait Ctx
where
    Self: Sized,
{
    fn new(s: &str) -> Result<Self, String>;
}

fn irun<C: Ctx, M: Acts<C>, V: Args<C>, K: AsRef<OsStr>, P: AsRef<str>>(
    config_path_override_env: K,
    config_path_default_home_rel: P,
) -> Result<(), String> {
    let c = C::new(&var(config_path_override_env).unwrap_or_else(|_| {
        let home = var("HOME").expect("Failed to get variable '$HOME'");
        format!("{home}/{}", config_path_default_home_rel.as_ref())
    }))?;
    if args().into_iter().skip(1).any(|e| e == *"--version") {
        V::next_impl(&c, &[]).map_err(|e| format!("{e}"))?;
        return Ok(());
    }
    M::run(&c).map_err(|e| format!("{e}"))?;
    Ok(())
}

pub fn run<C: Ctx, M: Acts<C>, V: Args<C>, K: AsRef<OsStr>, P: AsRef<str>>(
    config_path_override_env: K,
    config_path_default_home_rel: P,
) {
    if let Err(e) = irun::<C, M, V, K, P>(config_path_override_env, config_path_default_home_rel) {
        eprintln!("{e}");
    }
}

fn irund<C: Ctx, M: Args<C>, V: Args<C>, K: AsRef<OsStr>, P: AsRef<str>>(
    config_path_override_env: K,
    config_path_default_home_rel: P,
) -> Result<(), String> {
    let c = C::new(&var(config_path_override_env).unwrap_or_else(|_| {
        let home = var("HOME").expect("Failed to get variable '$HOME'");
        format!("{home}/{}", config_path_default_home_rel.as_ref())
    }))?;
    let args: Vec<_> = args().collect();
    let args: Vec<_> = args.iter().skip(1).map(|e| e.as_str()).collect();
    if args.iter().any(|e| *e == "--version") {
        V::next_impl(&c, &[]).map_err(|e| format!("{e}"))?;
    } else {
        M::next_impl(&c, &args).map_err(|e| format!("{e}"))?;
    }
    Ok(())
}
pub fn rund<M: Args<C>, V: Args<C>, C: Ctx, K: AsRef<OsStr>, P: AsRef<str>>(
    config_path_override_env: K,
    config_path_default_home_rel: P,
) {
    if let Err(e) = irund::<C, M, V, K, P>(config_path_override_env, config_path_default_home_rel) {
        eprintln!("{e}");
    }
}

pub fn reader<P: AsRef<Path>>(p: P) -> Result<BufReader<Box<dyn Read>>, String> {
    let f: Box<dyn Read> = Box::new(File::open(p.as_ref()).map_err(|e| {
        format!(
            "Failed to open file '{}' because '{e}'.",
            p.as_ref().to_string_lossy()
        )
    })?);
    Ok(BufReader::new(f))
}
pub fn from_toml<P: AsRef<Path>, O: serde::de::DeserializeOwned>(p: P) -> Result<O, String> {
    let mut buf = String::new();
    reader(p.as_ref())?.read_to_string(&mut buf).map_err(|e| {
        format!(
            "Error while reading '{}' because '{e}'.",
            p.as_ref().to_string_lossy()
        )
    })?;
    Ok(toml::from_str(&buf).map_err(|e| {
        format!(
            "Failed to parse '{}' because '{e}'.",
            p.as_ref().to_string_lossy()
        )
    })?)
}
#[track_caller]
pub fn write_toml<P: AsRef<Path>, O: serde::Serialize + fmt::Debug>(
    o: &O,
    p: P,
) -> Result<(), String> {
    let l = Location::caller();
    let s = toml::to_string_pretty(o)
        .map_err(|e| format!("Failed to serialize '{o:#?}' because '{e}' at {l}"))?;
    fs::write(&p, s).map_err(|e| {
        format!(
            "Failed to write to '{}' because '{e}'",
            p.as_ref().to_string_lossy()
        )
    })?;
    Ok(())
}
#[track_caller]
pub fn write_ctoml<P: AsRef<Path>, O: serde::Serialize + fmt::Debug>(
    o: &O,
    p: P,
) -> Result<(), String> {
    let l = Location::caller();
    let s = toml::to_string(o)
        .map_err(|e| format!("Failed to serialize '{o:#?}' because '{e}' at {l}"))?;
    fs::write(&p, s).map_err(|e| {
        format!(
            "Failed to write to '{}' because '{e}'",
            p.as_ref().to_string_lossy()
        )
    })?;
    Ok(())
}
