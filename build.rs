use anyhow::Result;
use std::env;
use std::process::Command;
use vergen::{vergen, Config, ShaKind};

fn main() -> Result<()> {
    println!("cargo:rerun-if-changed=package.json");
    println!("cargo:rerun-if-changed=package-lock.json");
    let proc =  Command::new("npm").arg("install").spawn();
    if let Err(_proc) = proc {
        println!("Build requires npm, but it was not found.  Please install Node >= 16.16.0");
        std::process::exit(1);
    }
    if let Ok(mut child) = proc {
        let status = child.wait()?;
        if !status.success() {
            std::process::exit(1);
        }
    }

    let jsdir = format!("{}/js", env::var("OUT_DIR").unwrap());
    println!("cargo:rustc-env=JS_DIR={}", jsdir);
    println!("cargo:rerun-if-changed=src/html_files/");
    let status = Command::new("npm")
        .arg("exec")
        .arg("--")
        .arg("tsc")
        .arg("-p")
        .arg("src/html_files/")
        .arg("--outDir")
        .arg(jsdir)
        .spawn()?
        .wait()?;
    if !status.success() {
        std::process::exit(1);
    }

    let mut config = Config::default();
    *config.git_mut().sha_kind_mut() = ShaKind::Short;
    vergen(config)
}
