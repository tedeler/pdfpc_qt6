#pragma once

#include "window_placer_proto.h"

#include <QCoreApplication>
#include <QElapsedTimer>
#include <QEventLoop>
#include <QFile>
#include <QFileInfo>
#include <QGuiApplication>
#include <QLocalSocket>
#include <QTimer>
#include <QtQml/qqmlregistration.h>

#if defined(Q_OS_LINUX)
#include <QDBusConnection>
#include <QDBusConnectionInterface>
#include <QDBusMessage>
#include <QDBusReply>
#include <QDir>
#endif

// Places the presenter (and, when asked, the audience) on a chosen output.
//
// direct    — X11, macOS, Windows: Qt sets the window geometry itself.
// kwin      — Plasma Wayland: one-shot KWin script. The only backend that
//             needs a compositor script.
// sway / i3 — i3 IPC (SWAYSOCK / I3SOCK): float, move to the output, size.
// hyprland  — Hyprland socket.
// output    — other Wayland compositors (GNOME, COSMIC, niri, …): a client
//             cannot position a normal window. The caller fullscreens it on
//             the output instead, which Wayland does allow.
class WindowPlacer : public QObject
{
    Q_OBJECT
    QML_ELEMENT
    Q_CLASSINFO("D-Bus Interface", "org.pdfpc.Place")
    Q_PROPERTY(QString mode READ mode CONSTANT)
    Q_PROPERTY(bool placesWindows READ placesWindows CONSTANT)
    Q_PROPERTY(bool claimsOutputs READ claimsOutputs CONSTANT)

public:
    explicit WindowPlacer(QObject *parent = nullptr)
        : QObject(parent)
        , m_mode(detectMode())
    {
        qInfo("pdfpc: placer=%s", qUtf8Printable(m_mode));
    }

    QString mode() const { return m_mode; }

    // True when this session can put a normal window at a chosen rect.
    bool placesWindows() const
    {
        return m_mode == QLatin1String("kwin") || m_mode == QLatin1String("sway")
            || m_mode == QLatin1String("i3") || m_mode == QLatin1String("hyprland");
    }

    // True when the audience window has to be moved onto its output before
    // Qt's fullscreen request. KWin does that from the fullscreen hint alone.
    bool claimsOutputs() const
    {
        return m_mode == QLatin1String("sway") || m_mode == QLatin1String("i3")
            || m_mode == QLatin1String("hyprland");
    }

    // rel* are pixels from the output origin. globalX/globalY are the same
    // point in Qt's virtual desktop (KWin fallback only). centeredScale > 0
    // ignores relX/relY and centers a window of that fraction of each edge.
    Q_INVOKABLE bool place(const QString &title, const QString &outputName, int outputIndex, int relX,
                           int relY, int relW, int relH, int globalX, int globalY, double centeredScale)
    {
        if (m_busy || !placesWindows())
            return false;
        m_busy = true;
        bool ok = false;
#if defined(Q_OS_LINUX)
        if (m_mode == QLatin1String("kwin"))
            ok = placeKwin(title, outputName, outputIndex, relX, relY, relW, relH, globalX, globalY, centeredScale);
        else if (m_mode == QLatin1String("sway") || m_mode == QLatin1String("i3"))
            ok = placeI3(title, outputName, outputIndex, relX, relY, relW, relH, centeredScale);
        else if (m_mode == QLatin1String("hyprland"))
            ok = placeHyprland(title, outputName, outputIndex, relX, relY, relW, relH, centeredScale);
#else
        Q_UNUSED(title);
        Q_UNUSED(outputName);
        Q_UNUSED(outputIndex);
        Q_UNUSED(relX);
        Q_UNUSED(relY);
        Q_UNUSED(relW);
        Q_UNUSED(relH);
        Q_UNUSED(globalX);
        Q_UNUSED(globalY);
        Q_UNUSED(centeredScale);
#endif
        m_busy = false;
        return ok;
    }

    // Move a window onto an output without resizing it. Used so fullscreen
    // lands on the audience screen under i3, Sway, and Hyprland.
    Q_INVOKABLE bool claimOutput(const QString &title, const QString &outputName, int outputIndex)
    {
        if (m_busy || !claimsOutputs())
            return false;
        m_busy = true;
        bool ok = false;
#if defined(Q_OS_LINUX)
        if (m_mode == QLatin1String("sway") || m_mode == QLatin1String("i3"))
            ok = claimI3(title, outputName, outputIndex);
        else if (m_mode == QLatin1String("hyprland"))
            ok = claimHyprland(title, outputName, outputIndex);
#else
        Q_UNUSED(title);
        Q_UNUSED(outputName);
        Q_UNUSED(outputIndex);
#endif
        m_busy = false;
        return ok;
    }

public slots:
    // KWin delivers this over the session bus. "ok" means the window was found.
    Q_SCRIPTABLE void placeResult(const QString &status)
    {
        m_placeOk = status == QLatin1String("ok");
        m_placeGot = true;
        emit placeReported();
    }

signals:
    void placeReported();

private:
    static QString detectMode()
    {
        const bool wayland = QGuiApplication::platformName().startsWith(QLatin1String("wayland"));
#if defined(Q_OS_LINUX)
        if (wayland && kwinRunning())
            return QStringLiteral("kwin");
        if (!swaySocketPath().isEmpty())
            return QStringLiteral("sway");
        if (!i3SocketPath().isEmpty())
            return QStringLiteral("i3");
        if (wayland && !hyprSocketPath().isEmpty())
            return QStringLiteral("hyprland");
#else
        Q_UNUSED(wayland);
#endif
        if (wayland)
            return QStringLiteral("output");
        return QStringLiteral("direct");
    }

#if defined(Q_OS_LINUX)
    static bool kwinRunning()
    {
        auto *iface = QDBusConnection::sessionBus().interface();
        return iface && iface->isServiceRegistered(QStringLiteral("org.kde.KWin"));
    }

    static QString existingSocket(const QString &path)
    {
        return (!path.isEmpty() && QFileInfo::exists(path)) ? path : QString();
    }

    static QString swaySocketPath() { return existingSocket(qEnvironmentVariable("SWAYSOCK")); }

    static QString i3SocketPath() { return existingSocket(qEnvironmentVariable("I3SOCK")); }

    static QString i3SocketForMode(const QString &mode)
    {
        return mode == QLatin1String("sway") ? swaySocketPath() : i3SocketPath();
    }

    static QString hyprSocketPath()
    {
        const QString signature = qEnvironmentVariable("HYPRLAND_INSTANCE_SIGNATURE");
        const QString runtime = qEnvironmentVariable("XDG_RUNTIME_DIR");
        if (signature.isEmpty() || runtime.isEmpty())
            return {};
        return existingSocket(runtime + QStringLiteral("/hypr/") + signature + QStringLiteral("/.socket.sock"));
    }

    static bool readExact(QLocalSocket &sock, int bytes, int timeoutMs, QByteArray *out, QString *error)
    {
        QElapsedTimer timer;
        timer.start();
        while (out->size() < bytes && timer.elapsed() < timeoutMs) {
            if (sock.bytesAvailable() < (bytes - out->size())) {
                if (!sock.waitForReadyRead(timeoutMs - int(timer.elapsed())))
                    break;
            }
            *out += sock.read(bytes - out->size());
        }
        if (out->size() != bytes) {
            *error = QStringLiteral("short read");
            return false;
        }
        return true;
    }

    static bool i3Call(const QString &socketPath, quint32 type, const QByteArray &payload, QByteArray *reply,
                       QString *error)
    {
        QLocalSocket sock;
        sock.connectToServer(socketPath);
        if (!sock.waitForConnected(300)) {
            *error = sock.errorString();
            return false;
        }
        const QByteArray frame = i3IpcFrame(type, payload);
        if (sock.write(frame) != frame.size() || !sock.waitForBytesWritten(300)) {
            *error = sock.errorString();
            return false;
        }
        QByteArray header;
        if (!readExact(sock, 14, 500, &header, error) || !header.startsWith("i3-ipc")) {
            *error = QStringLiteral("bad i3 header");
            return false;
        }
        const auto byte = [&header](int index) { return quint32(quint8(header.at(index))); };
        const quint32 len = byte(6) | (byte(7) << 8) | (byte(8) << 16) | (byte(9) << 24);
        if (len > 8u * 1024u * 1024u) {
            *error = QStringLiteral("i3 payload too large");
            return false;
        }
        QByteArray body;
        if (!readExact(sock, int(len), 800, &body, error))
            return false;
        *reply = body;
        return true;
    }

    static bool i3Run(const QString &socketPath, const QString &command, QString *error)
    {
        QByteArray reply;
        if (!i3Call(socketPath, 0, command.toUtf8(), &reply, error))
            return false;
        if (!i3CommandSucceeded(reply)) {
            *error = QString::fromUtf8(reply.left(180));
            return false;
        }
        return true;
    }

    bool placeI3(const QString &title, const QString &outputName, int outputIndex, int relX, int relY, int relW,
                 int relH, double centeredScale)
    {
        const QString socketPath = i3SocketForMode(m_mode);
        QString error;
        QByteArray outputsJson;
        if (!i3Call(socketPath, 3, QByteArray(), &outputsJson, &error)) {
            qInfo("pdfpc: %s outputs: %s", qUtf8Printable(m_mode), qUtf8Printable(error));
            return false;
        }
        const PlacerOutput output = matchOutput(parseSwayOutputs(outputsJson), outputName, outputIndex);
        const PlacerRect rect = rectForOutput(output, centeredScale, relX, relY, relW, relH, false);
        const QStringList commands = i3PlaceCommands(QCoreApplication::applicationPid(), title, output.name, rect);
        if (commands.isEmpty()) {
            qInfo("pdfpc: %s place: no output named '%s'", qUtf8Printable(m_mode), qUtf8Printable(outputName));
            return false;
        }
        for (const auto &command : commands) {
            if (!i3Run(socketPath, command, &error)) {
                qInfo("pdfpc: %s place failed: %s", qUtf8Printable(m_mode), qUtf8Printable(error));
                return false;
            }
        }
        qInfo("pdfpc: %s placed '%s' on %s at %d,%d %dx%d", qUtf8Printable(m_mode), qUtf8Printable(title),
              qUtf8Printable(output.name), rect.x, rect.y, rect.w, rect.h);
        return true;
    }

    bool claimI3(const QString &title, const QString &outputName, int outputIndex)
    {
        const QString socketPath = i3SocketForMode(m_mode);
        QString error;
        QByteArray outputsJson;
        QString resolved = outputName;
        if (resolved.isEmpty() && i3Call(socketPath, 3, QByteArray(), &outputsJson, &error)) {
            resolved = matchOutput(parseSwayOutputs(outputsJson), outputName, outputIndex).name;
        }
        const QString command = i3MoveToOutputCommand(QCoreApplication::applicationPid(), title, resolved);
        if (command.isEmpty() || !i3Run(socketPath, command, &error)) {
            qInfo("pdfpc: %s claim '%s' failed: %s", qUtf8Printable(m_mode), qUtf8Printable(title),
                  qUtf8Printable(error));
            return false;
        }
        return true;
    }

    static QByteArray hyprCall(const QString &socketPath, const QByteArray &request, QString *error)
    {
        QLocalSocket sock;
        sock.connectToServer(socketPath);
        if (!sock.waitForConnected(300)) {
            *error = sock.errorString();
            return {};
        }
        if (sock.write(request) != request.size() || !sock.waitForBytesWritten(300)) {
            *error = sock.errorString();
            return {};
        }
        QByteArray reply;
        QElapsedTimer timer;
        timer.start();
        while (timer.elapsed() < 400) {
            if (!sock.waitForReadyRead(100)) {
                if (!reply.isEmpty())
                    break;
                continue;
            }
            reply += sock.readAll();
        }
        if (reply.isEmpty())
            *error = QStringLiteral("empty hyprland reply");
        return reply;
    }

    bool placeHyprland(const QString &title, const QString &outputName, int outputIndex, int relX, int relY,
                       int relW, int relH, double centeredScale)
    {
        const QString socketPath = hyprSocketPath();
        QString error;
        const QByteArray monitors = hyprCall(socketPath, QByteArrayLiteral("j/monitors"), &error);
        if (monitors.isEmpty()) {
            qInfo("pdfpc: hyprland monitors: %s", qUtf8Printable(error));
            return false;
        }
        const PlacerOutput output = matchOutput(parseHyprMonitors(monitors), outputName, outputIndex);
        const PlacerRect rect = rectForOutput(output, centeredScale, relX, relY, relW, relH, true);
        const QStringList commands = hyprPlaceCommands(title, rect);
        if (commands.isEmpty()) {
            qInfo("pdfpc: hyprland place: no output named '%s'", qUtf8Printable(outputName));
            return false;
        }
        for (const auto &command : commands) {
            const QByteArray reply = hyprCall(socketPath, QByteArrayLiteral("/") + command.toUtf8(), &error);
            if (reply.isEmpty() || !hyprDispatchOk(reply)) {
                qInfo("pdfpc: hyprland '%s' failed: %s", qUtf8Printable(command.left(80)),
                      qUtf8Printable(reply.isEmpty() ? error : QString::fromUtf8(reply.left(180))));
                return false;
            }
        }
        qInfo("pdfpc: hyprland placed '%s' on %s at %d,%d %dx%d", qUtf8Printable(title),
              qUtf8Printable(output.name), rect.x, rect.y, rect.w, rect.h);
        return true;
    }

    bool claimHyprland(const QString &title, const QString &outputName, int outputIndex)
    {
        QString resolved = outputName;
        const QString socketPath = hyprSocketPath();
        QString error;
        if (resolved.isEmpty()) {
            const QByteArray monitors = hyprCall(socketPath, QByteArrayLiteral("j/monitors"), &error);
            resolved = matchOutput(parseHyprMonitors(monitors), outputName, outputIndex).name;
        }
        const QStringList commands = hyprClaimCommands(title, resolved);
        if (commands.isEmpty())
            return false;
        for (const auto &command : commands) {
            const QByteArray reply = hyprCall(socketPath, QByteArrayLiteral("/") + command.toUtf8(), &error);
            if (reply.isEmpty() || !hyprDispatchOk(reply)) {
                qInfo("pdfpc: hyprland claim failed: %s",
                      qUtf8Printable(reply.isEmpty() ? error : QString::fromUtf8(reply.left(180))));
                return false;
            }
        }
        return true;
    }

    bool placeKwin(const QString &title, const QString &outputName, int outputIndex, int relX, int relY, int relW,
                   int relH, int globalX, int globalY, double centeredScale)
    {
        auto bus = QDBusConnection::sessionBus();
        const QString objectPath = QStringLiteral("/pdfpc/place%1").arg(++m_counter);
        if (!bus.registerObject(objectPath, this, QDBusConnection::ExportScriptableSlots)) {
            qInfo("pdfpc: kwin place: cannot register %s", qUtf8Printable(objectPath));
            return false;
        }

        const QString name = QStringLiteral("pdfpc-place-%1-%2").arg(QCoreApplication::applicationPid()).arg(m_counter);
        const QString filePath = QDir::temp().filePath(name + QStringLiteral(".js"));
        const QString script = kwinPlaceScript(title, QCoreApplication::applicationPid(), outputName, outputIndex, relX,
                                               relY, relW, relH, globalX, globalY, centeredScale, bus.baseService(),
                                               objectPath);
        QFile file(filePath);
        if (!file.open(QIODevice::WriteOnly | QIODevice::Truncate)) {
            bus.unregisterObject(objectPath);
            return false;
        }
        file.write(script.toUtf8());
        file.close();

        m_placeGot = false;
        m_placeOk = false;

        QDBusMessage load = QDBusMessage::createMethodCall(
            QStringLiteral("org.kde.KWin"), QStringLiteral("/Scripting"), QStringLiteral("org.kde.kwin.Scripting"),
            QStringLiteral("loadScript"));
        load << filePath << name;
        const QDBusReply<int> id = bus.call(load, QDBus::Block, 1000);
        if (!id.isValid() || id.value() < 0) {
            qInfo("pdfpc: kwin loadScript failed");
            QFile::remove(filePath);
            bus.unregisterObject(objectPath);
            return false;
        }

        const bool ran = runKwinScript(id.value());
        if (ran && !m_placeGot) {
            QEventLoop loop;
            QTimer timer;
            timer.setSingleShot(true);
            connect(&timer, &QTimer::timeout, &loop, &QEventLoop::quit);
            connect(this, &WindowPlacer::placeReported, &loop, &QEventLoop::quit);
            timer.start(400);
            loop.exec();
        }

        QDBusMessage unload = QDBusMessage::createMethodCall(
            QStringLiteral("org.kde.KWin"), QStringLiteral("/Scripting"), QStringLiteral("org.kde.kwin.Scripting"),
            QStringLiteral("unloadScript"));
        unload << name;
        bus.call(unload, QDBus::NoBlock);
        QFile::remove(filePath);
        bus.unregisterObject(objectPath);

        if (!ran) {
            qInfo("pdfpc: kwin run failed");
            return false;
        }
        if (!m_placeGot) {
            // The script itself ran. A missing callback must not turn a move
            // that already happened into a fullscreen fallback.
            qInfo("pdfpc: kwin script ran but sent no result; assuming the move was applied");
            return true;
        }
        if (!m_placeOk)
            qInfo("pdfpc: kwin did not find '%s' yet", qUtf8Printable(title));
        return m_placeOk;
    }

    static bool runKwinScript(int id)
    {
        auto bus = QDBusConnection::sessionBus();
        const QStringList paths = {QStringLiteral("/Scripting/Script%1").arg(id), QStringLiteral("/%1").arg(id)};
        for (const auto &path : paths) {
            QDBusMessage run = QDBusMessage::createMethodCall(
                QStringLiteral("org.kde.KWin"), path, QStringLiteral("org.kde.kwin.Script"), QStringLiteral("run"));
            const QDBusMessage reply = bus.call(run, QDBus::Block, 1000);
            if (reply.type() != QDBusMessage::ErrorMessage)
                return true;
            qInfo("pdfpc: kwin run %s: %s", qUtf8Printable(path), qUtf8Printable(reply.errorMessage()));
        }
        return false;
    }
#endif

    QString m_mode;
    bool m_busy = false;
    bool m_placeGot = false;
    bool m_placeOk = false;
    int m_counter = 0;
};
