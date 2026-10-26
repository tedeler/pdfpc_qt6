use cxx_qt_build::{CxxQtBuilder, QmlModule};
use std::path::Path;
use std::process::Command;

fn main() {
    let mut builder = CxxQtBuilder::new_qml_module(
        QmlModule::new("org.pdfpc.app").qml_files([
            "qml/Main.qml",
            "qml/Presenter.qml",
            "qml/Audience.qml",
            "qml/SlideView.qml",
        ]),
    )
    .qt_module("Network")
    .qt_module("Quick")
    .qt_module("Pdf");

    // KWin is reached over the session bus. macOS and Windows stay on the
    // direct placer and do not need Qt DBus.
    #[cfg(target_os = "linux")]
    {
        builder = builder.qt_module("DBus");
    }

    builder
        .files(["src/session.rs"])
        .cpp_file("cpp/ink_canvas.h")
        .cpp_file("cpp/window_placer.h")
        .build();

    println!("cargo:rerun-if-changed=cpp/laser_trail_geom.h");
    println!("cargo:rerun-if-changed=cpp/test_laser_trail_geom.cpp");
    println!("cargo:rerun-if-changed=cpp/window_placer.h");
    println!("cargo:rerun-if-changed=cpp/window_placer_proto.h");
    println!("cargo:rerun-if-changed=cpp/test_window_placer.cpp");

    // cxx-qt-lib only links the Qt modules of its own features.
    // D-Bus is the KWin channel. macOS and Windows use the direct placer and
    // do not link it.
    #[cfg(target_os = "linux")]
    println!("cargo:rustc-link-arg=-lQt6DBus");

    embed_version_info();
}

fn embed_version_info() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let version_file = manifest_dir.join("../VERSION");
    println!("cargo:rerun-if-changed={}", version_file.display());

    let version = std::fs::read_to_string(&version_file)
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| env!("CARGO_PKG_VERSION").to_string());
    println!("cargo:rustc-env=PDFPC_VERSION={version}");

    let git_dir = manifest_dir.join("../.git/HEAD");
    println!("cargo:rerun-if-changed={}", git_dir.display());
    // Packed refs / index change with checkouts; best-effort refresh of the id.
    let git_index = manifest_dir.join("../.git/index");
    if git_index.exists() {
        println!("cargo:rerun-if-changed={}", git_index.display());
    }

    let git_id = Command::new("git")
        .args(["rev-parse", "--short=12", "HEAD"])
        .current_dir(manifest_dir)
        .output()
        .ok()
        .filter(|o| o.status.success())
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "unknown".to_string());
    println!("cargo:rustc-env=PDFPC_GIT_ID={git_id}");
}
