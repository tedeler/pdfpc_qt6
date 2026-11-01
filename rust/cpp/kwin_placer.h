#pragma once

#include <QCoreApplication>
#include <QDBusConnection>
#include <QDBusConnectionInterface>
#include <QDBusMessage>
#include <QDBusReply>
#include <QDir>
#include <QFile>
#include <QJsonDocument>
#include <QJsonValue>
#include <QObject>
#include <QRect>
#include <QTimer>
#include <QtQml/qqmlregistration.h>

// Wayland clients cannot position their own normal windows. Under KDE the
// presenter window is moved by a one-shot KWin script loaded over D-Bus.
class KWinPlacer : public QObject
{
    Q_OBJECT
    QML_ELEMENT
    Q_PROPERTY(bool available READ available CONSTANT)

public:
    explicit KWinPlacer(QObject *parent = nullptr)
        : QObject(parent)
    {
        auto *iface = QDBusConnection::sessionBus().interface();
        m_available = iface && iface->isServiceRegistered(QStringLiteral("org.kde.KWin"));
    }

    bool available() const { return m_available; }

    // Sets the frame geometry (global logical coordinates) of this process'
    // window whose caption equals `caption`.
    Q_INVOKABLE bool setFrameGeometry(const QString &caption, const QRect &geometry)
    {
        if (!m_available)
            return false;

        const QString name = QStringLiteral("pdfpc-place-%1-%2")
                                 .arg(QCoreApplication::applicationPid())
                                 .arg(++m_counter);
        const QString path = QDir::temp().filePath(name + QStringLiteral(".js"));
        const QString captionJson = QString::fromUtf8(
            QJsonDocument::fromVariant(QVariantList{caption}).toJson(QJsonDocument::Compact));
        const QString script = QStringLiteral(
            "var caption = %1[0];\n"
            "var list = workspace.windowList();\n"
            "for (var i = 0; i < list.length; ++i) {\n"
            "    var w = list[i];\n"
            "    if (w.pid === %2 && w.caption === caption) {\n"
            "        if (w.fullScreen) w.fullScreen = false;\n"
            "        w.frameGeometry = { x: %3, y: %4, width: %5, height: %6 };\n"
            "    }\n"
            "}\n")
                                   .arg(captionJson)
                                   .arg(QCoreApplication::applicationPid())
                                   .arg(geometry.x())
                                   .arg(geometry.y())
                                   .arg(geometry.width())
                                   .arg(geometry.height());

        return runScript(name, script);
    }

    // Number of KDE virtual desktops, or 0 when unknown.
    Q_INVOKABLE int desktopCount() const
    {
        if (!m_available)
            return 0;
        QDBusMessage get = QDBusMessage::createMethodCall(
            QStringLiteral("org.kde.KWin"), QStringLiteral("/VirtualDesktopManager"),
            QStringLiteral("org.freedesktop.DBus.Properties"), QStringLiteral("Get"));
        get << QStringLiteral("org.kde.KWin.VirtualDesktopManager") << QStringLiteral("count");
        QDBusReply<QVariant> r = QDBusConnection::sessionBus().call(get);
        return r.isValid() ? r.value().toInt() : 0;
    }

    // Moves all windows of this process to virtual desktop `number` (1-based)
    // and makes that desktop current.
    Q_INVOKABLE bool moveToDesktop(int number)
    {
        if (!m_available || number < 1)
            return false;
        const QString name = QStringLiteral("pdfpc-desktop-%1-%2")
                                 .arg(QCoreApplication::applicationPid())
                                 .arg(++m_counter);
        const QString script = QStringLiteral(
            "var target = workspace.desktops[%1];\n"
            "if (target) {\n"
            "    var list = workspace.windowList();\n"
            "    for (var i = 0; i < list.length; ++i) {\n"
            "        if (list[i].pid === %2) list[i].desktops = [target];\n"
            "    }\n"
            "    workspace.currentDesktop = target;\n"
            "}\n")
                                   .arg(number - 1)
                                   .arg(QCoreApplication::applicationPid());
        return runScript(name, script);
    }

private:
    bool runScript(const QString &name, const QString &script)
    {
        const QString path = QDir::temp().filePath(name + QStringLiteral(".js"));
        QFile file(path);
        if (!file.open(QIODevice::WriteOnly | QIODevice::Truncate))
            return false;
        file.write(script.toUtf8());
        file.close();

        auto bus = QDBusConnection::sessionBus();
        QDBusMessage load = QDBusMessage::createMethodCall(
            QStringLiteral("org.kde.KWin"), QStringLiteral("/Scripting"),
            QStringLiteral("org.kde.kwin.Scripting"), QStringLiteral("loadScript"));
        load << path << name;
        QDBusReply<int> id = bus.call(load);
        if (!id.isValid() || id.value() < 0) {
            QFile::remove(path);
            return false;
        }

        QDBusMessage run = QDBusMessage::createMethodCall(
            QStringLiteral("org.kde.KWin"), QStringLiteral("/Scripting/Script%1").arg(id.value()),
            QStringLiteral("org.kde.kwin.Script"), QStringLiteral("run"));
        bus.call(run);

        QTimer::singleShot(1000, this, [name, path]() {
            QDBusMessage unload = QDBusMessage::createMethodCall(
                QStringLiteral("org.kde.KWin"), QStringLiteral("/Scripting"),
                QStringLiteral("org.kde.kwin.Scripting"), QStringLiteral("unloadScript"));
            unload << name;
            QDBusConnection::sessionBus().call(unload, QDBus::NoBlock);
            QFile::remove(path);
        });
        return true;
    }

    bool m_available = false;
    int m_counter = 0;
};
