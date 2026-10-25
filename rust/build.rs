use cxx_qt_build::{CxxQtBuilder, QmlModule};

fn main() {
    CxxQtBuilder::new_qml_module(
        QmlModule::new("org.pdfpc.app")
            .qml_files([
                "qml/Main.qml",
                "qml/Presenter.qml",
                "qml/Audience.qml",
                "qml/SlideView.qml",
            ]),
    )
    .qt_module("Network")
    .qt_module("Quick")
    .qt_module("Pdf")
    .qt_module("DBus")
    .files(["src/session.rs"])
    .cpp_file("cpp/ink_canvas.h")
    .cpp_file("cpp/kwin_placer.h")
    .build();

    // cxx-qt-lib only links the Qt modules of its own features.
    println!("cargo:rustc-link-arg=-lQt6DBus");
}
