use std::{env, path::PathBuf, process::Command};
fn main() {
    println!("cargo:rerun-if-changed=assets");
    println!("cargo:rerun-if-env-changed=GLIB_COMPILE_RESOURCES");
    println!("cargo:rerun-if-env-changed=WINDRES");
    let out = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    let status = Command::new(
        env::var_os("GLIB_COMPILE_RESOURCES").unwrap_or_else(|| "glib-compile-resources".into()),
    )
    .arg("--sourcedir=assets")
    .arg("--target")
    .arg(out.join("leditor.gresource"))
    .arg("assets/leditor.gresource.xml")
    .status()
    .expect("Install GLib development tools (glib-compile-resources)");
    assert!(status.success(), "Icon resource compilation failed");
    if env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        let msvc = env::var("CARGO_CFG_TARGET_ENV").as_deref() == Ok("msvc");
        let object = out.join(if msvc {
            "leditor.res"
        } else {
            "leditor-icon.o"
        });
        let mut command = Command::new(
            env::var_os("WINDRES").unwrap_or_else(|| if msvc { "rc" } else { "windres" }.into()),
        );
        if msvc {
            command.arg("/fo").arg(&object).arg("assets/leditor.rc");
        } else {
            command
                .args(["-i", "assets/leditor.rc", "-O", "coff", "-o"])
                .arg(&object);
        }
        assert!(
            command
                .status()
                .expect("Install Windows resource compiler; set WINDRES for cross-compilation")
                .success(),
            "Windows icon compilation failed"
        );
        println!("cargo:rustc-link-arg-bin=leditor={}", object.display());
    }
}
