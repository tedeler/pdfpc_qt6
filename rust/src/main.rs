//! pdfpc — slim dual-monitor PDF presenter.

mod laser_trail;
mod recent;
mod session;

use cxx_qt_lib::{QGuiApplication, QQmlApplicationEngine, QUrl};

fn main() {
    let mut app = QGuiApplication::new();
    // Before any window exists, so KDE binds the surface to pdfpc-qt6.desktop.
    session::qobject::configure_app_identity();
    let mut engine = QQmlApplicationEngine::new();

    if let Some(engine) = engine.as_mut() {
        engine.load(&QUrl::from("qrc:/qt/qml/org/pdfpc/app/qml/Main.qml"));
    }
    session::qobject::apply_window_icons();

    if let Some(app) = app.as_mut() {
        app.exec();
    }
}
