#pragma once

#include <QGuiApplication>
#include <QIcon>
#include <QImage>
#include <QPixmap>
#include <QTimer>
#include <QWindow>

// KDE matches the Wayland app id (and the X11 window class) to
// pdfpc-qt6.desktop. The embedded PNG is the title-bar icon; Plasma also
// reads Icon= from the desktop entry once the app id matches.
inline QIcon pdfpc_app_icon()
{
    const QString resource = QStringLiteral(":/qt/qml/org/pdfpc/app/icons/pdfpc-qt6.png");
    const QImage image(resource);
    if (image.isNull())
        return QIcon::fromTheme(QStringLiteral("pdfpc-qt6"));

    // A single 256px image exceeds the X11 request-size limit, so the XCB
    // backend drops it and the title bar stays empty. Ship the sizes a
    // window manager actually uses.
    QIcon icon;
    const int sizes[] = {16, 22, 32, 48, 64, 128};
    for (int size : sizes) {
        icon.addPixmap(QPixmap::fromImage(
            image.scaled(size, size, Qt::IgnoreAspectRatio, Qt::SmoothTransformation)));
    }
    return icon;
}

inline void configure_app_identity()
{
    QGuiApplication::setApplicationName(QStringLiteral("pdfpc-qt6"));
    QGuiApplication::setDesktopFileName(QStringLiteral("pdfpc-qt6"));
    const QIcon icon = pdfpc_app_icon();
    if (!icon.isNull())
        QGuiApplication::setWindowIcon(icon);
}

// Qt Quick creates the native surface when the window is shown, and that
// surface does not pick up QGuiApplication::windowIcon on its own. Apply the
// icon to every window now and again on the next event-loop turn, after a
// visibility change has created the platform window.
inline void apply_window_icons()
{
    const QIcon icon = QGuiApplication::windowIcon();
    if (icon.isNull())
        return;
    const auto apply = [icon]() {
        const auto windows = QGuiApplication::allWindows();
        for (QWindow *window : windows)
            window->setIcon(icon);
    };
    apply();
    if (QObject *app = QGuiApplication::instance())
        QTimer::singleShot(0, app, apply);
}
