#pragma once

// Pure helpers for placing pdfpc windows. The compositor backends in
// window_placer.h turn these into socket or D-Bus calls. No I/O here so the
// layout can be tested without a session.

#include <QByteArray>
#include <QJsonArray>
#include <QJsonDocument>
#include <QJsonObject>
#include <QJsonValue>
#include <QList>
#include <QString>
#include <QStringList>
#include <QtGlobal>

struct PlacerOutput {
    QString name;
    int x = 0;
    int y = 0;
    int w = 0;
    int h = 0;

    bool valid() const { return w > 0 && h > 0; }
};

// x/y are relative to the output when global is false, and in the compositor's
// global layout when global is true.
struct PlacerRect {
    int x = 0;
    int y = 0;
    int w = 0;
    int h = 0;
    bool ok = false;
};

// centeredScale > 0 sizes the window to that fraction of each output edge and
// centers it (the presenter uses 1/sqrt(2), so the area is half the output).
// Otherwise relX/relY/relW/relH are pixels measured from the output origin.
inline PlacerRect rectForOutput(const PlacerOutput &output, double centeredScale, int relX, int relY,
                                int relW, int relH, bool global)
{
    PlacerRect rect;
    if (!output.valid())
        return rect;
    if (centeredScale > 0) {
        rect.w = qRound(output.w * centeredScale);
        rect.h = qRound(output.h * centeredScale);
        rect.x = qRound((output.w - rect.w) / 2.0);
        rect.y = qRound((output.h - rect.h) / 2.0);
    } else {
        rect.x = relX;
        rect.y = relY;
        rect.w = relW;
        rect.h = relH;
    }
    if (rect.w <= 0 || rect.h <= 0)
        return rect;
    if (global) {
        rect.x += output.x;
        rect.y += output.y;
    }
    rect.ok = true;
    return rect;
}

inline PlacerOutput matchOutput(const QList<PlacerOutput> &outputs, const QString &name, int index)
{
    if (!name.isEmpty()) {
        for (const auto &output : outputs) {
            if (output.name == name)
                return output;
        }
    }
    if (index >= 0 && index < outputs.size())
        return outputs.at(index);
    return {};
}

inline QList<PlacerOutput> parseSwayOutputs(const QByteArray &json)
{
    QList<PlacerOutput> outputs;
    const auto doc = QJsonDocument::fromJson(json);
    if (!doc.isArray())
        return outputs;
    for (const auto &value : doc.array()) {
        if (!value.isObject())
            continue;
        const auto obj = value.toObject();
        if (obj.contains(QStringLiteral("active")) && !obj.value(QStringLiteral("active")).toBool())
            continue;
        const auto rect = obj.value(QStringLiteral("rect")).toObject();
        PlacerOutput output;
        output.name = obj.value(QStringLiteral("name")).toString();
        output.x = rect.value(QStringLiteral("x")).toInt();
        output.y = rect.value(QStringLiteral("y")).toInt();
        output.w = rect.value(QStringLiteral("width")).toInt();
        output.h = rect.value(QStringLiteral("height")).toInt();
        if (output.valid() && !output.name.isEmpty())
            outputs.append(output);
    }
    return outputs;
}

inline QList<PlacerOutput> parseHyprMonitors(const QByteArray &json)
{
    QList<PlacerOutput> outputs;
    const auto doc = QJsonDocument::fromJson(json);
    if (!doc.isArray())
        return outputs;
    for (const auto &value : doc.array()) {
        if (!value.isObject())
            continue;
        const auto obj = value.toObject();
        if (obj.value(QStringLiteral("disabled")).toBool())
            continue;
        PlacerOutput output;
        output.name = obj.value(QStringLiteral("name")).toString();
        output.x = obj.value(QStringLiteral("x")).toInt();
        output.y = obj.value(QStringLiteral("y")).toInt();
        output.w = obj.value(QStringLiteral("width")).toInt();
        output.h = obj.value(QStringLiteral("height")).toInt();
        if (output.valid() && !output.name.isEmpty())
            outputs.append(output);
    }
    return outputs;
}

inline QByteArray i3IpcFrame(quint32 type, const QByteArray &payload)
{
    QByteArray msg;
    msg.append("i3-ipc", 6);
    const quint32 len = quint32(payload.size());
    char header[8];
    header[0] = char(len & 0xff);
    header[1] = char((len >> 8) & 0xff);
    header[2] = char((len >> 16) & 0xff);
    header[3] = char((len >> 24) & 0xff);
    header[4] = char(type & 0xff);
    header[5] = char((type >> 8) & 0xff);
    header[6] = char((type >> 16) & 0xff);
    header[7] = char((type >> 24) & 0xff);
    msg.append(header, 8);
    msg.append(payload);
    return msg;
}

inline bool i3CommandSucceeded(const QByteArray &payload)
{
    const auto doc = QJsonDocument::fromJson(payload);
    if (!doc.isArray() || doc.array().isEmpty())
        return false;
    for (const auto &value : doc.array()) {
        if (!value.isObject() || !value.toObject().value(QStringLiteral("success")).toBool(false))
            return false;
    }
    return true;
}

// i3 criteria end at ']' and quote strings with '"'. Titles we place are
// fixed, but keep the command from being rewritten by a surprising caption.
inline QString i3Quote(const QString &text)
{
    QString out;
    out.reserve(text.size());
    for (const QChar ch : text) {
        if (ch == QLatin1Char('\\') || ch == QLatin1Char('"'))
            out += QLatin1Char('\\');
        out += ch;
    }
    return out;
}

// Output names are inserted into an i3/Sway command outside of quotes.
// Connector names are like HDMI-A-1; anything that could add a command is refused.
inline bool i3TokenSafe(const QString &text)
{
    if (text.isEmpty())
        return false;
    for (const QChar ch : text) {
        if (ch.unicode() < 32)
            return false;
        switch (ch.unicode()) {
        case '"':
        case '\'':
        case ']':
        case ';':
        case '|':
        case '&':
        case '`':
        case '$':
        case '\\':
            return false;
        default:
            break;
        }
    }
    return true;
}

inline QString i3Criteria(qint64 pid, const QString &title)
{
    if (pid <= 0 || title.contains(QLatin1Char(']')) || title.contains(QLatin1Char('\n'))
        || title.contains(QLatin1Char('\r')))
        return {};
    return QStringLiteral("[pid=%1 title=\"%2\"]").arg(pid).arg(i3Quote(title));
}

inline QString i3MoveToOutputCommand(qint64 pid, const QString &title, const QString &output)
{
    if (!i3TokenSafe(output))
        return {};
    const QString criteria = i3Criteria(pid, title);
    if (criteria.isEmpty())
        return {};
    return criteria + QStringLiteral(" move container to output \"") + i3Quote(output) + QLatin1Char('"');
}

inline QStringList i3PlaceCommands(qint64 pid, const QString &title, const QString &output, const PlacerRect &rect)
{
    const QString criteria = i3Criteria(pid, title);
    const QString move = i3MoveToOutputCommand(pid, title, output);
    if (criteria.isEmpty() || move.isEmpty() || !rect.ok)
        return {};
    return {move,
            criteria + QStringLiteral(" floating enable"),
            criteria + QStringLiteral(" resize set %1 px %2 px").arg(rect.w).arg(rect.h),
            criteria + QStringLiteral(" move position %1 px %2 px").arg(rect.x).arg(rect.y)};
}

inline QString regexEscape(const QString &text)
{
    QString out;
    out.reserve(text.size());
    for (const QChar ch : text) {
        if (QStringLiteral("\\^$.|?*+()[]{}").contains(ch))
            out += QLatin1Char('\\');
        out += ch;
    }
    return out;
}

inline QString hyprWindow(const QString &title)
{
    return QStringLiteral("title:^") + regexEscape(title) + QLatin1Char('$');
}

inline bool hyprOutputSafe(const QString &output)
{
    return i3TokenSafe(output) && !output.contains(QLatin1Char(',')) && !output.contains(QLatin1Char(' '));
}

inline QStringList hyprPlaceCommands(const QString &title, const PlacerRect &globalRect)
{
    if (title.isEmpty() || !globalRect.ok)
        return {};
    const QString window = hyprWindow(title);
    return {QStringLiteral("dispatch setfloating ") + window,
            QStringLiteral("dispatch resizewindowpixel exact %1 %2,%3")
                .arg(globalRect.w)
                .arg(globalRect.h)
                .arg(window),
            QStringLiteral("dispatch movewindowpixel exact %1 %2,%3")
                .arg(globalRect.x)
                .arg(globalRect.y)
                .arg(window)};
}

inline QStringList hyprClaimCommands(const QString &title, const QString &output)
{
    if (title.isEmpty() || !hyprOutputSafe(output))
        return {};
    const QString window = hyprWindow(title);
    return {QStringLiteral("dispatch focuswindow ") + window,
            QStringLiteral("dispatch movewindow mon:") + output};
}

inline bool hyprDispatchOk(const QByteArray &reply)
{
    const QString text = QString::fromUtf8(reply).trimmed();
    return text == QLatin1String("ok") || text.startsWith(QLatin1String("ok "))
        || text.startsWith(QLatin1String("ok\n"));
}

inline QString jsonJsString(const QString &text)
{
    QJsonArray array;
    array.append(text);
    const QByteArray json = QJsonDocument(array).toJson(QJsonDocument::Compact);
    // Compact form is ["..."]. The element is a JSON string literal.
    return QString::fromUtf8(json.mid(1, json.size() - 2));
}

// One-shot KWin script. Plasma 6 lists windows with windowList(); Plasma 5
// with clientList(). Geometry is assigned as a whole rect because the x/y
// fields are read-only on current KWin. Coordinates come from the output KWin
// itself knows, with the Qt rect only as a fallback.
inline QString kwinPlaceScript(const QString &caption, qint64 pid, const QString &outputName, int outputIndex,
                               int relX, int relY, int relW, int relH, int globalX, int globalY,
                               double centeredScale, const QString &dbusService, const QString &dbusPath)
{
    QString script = QStringLiteral(
               "var caption = @@CAPTION@@;\n"
               "var outputName = @@OUTPUT@@;\n"
               "var outputIndex = @@INDEX@@;\n"
               "var relX = @@REL_X@@;\n"
               "var relY = @@REL_Y@@;\n"
               "var relW = @@REL_W@@;\n"
               "var relH = @@REL_H@@;\n"
               "var globalX = @@GLOBAL_X@@;\n"
               "var globalY = @@GLOBAL_Y@@;\n"
               "var scale = @@SCALE@@;\n"
               "var dbusService = @@DBUS_SERVICE@@;\n"
               "var dbusPath = @@DBUS_PATH@@;\n"
               "var pdfpcPid = @@PID@@;\n"
               "function pdfpcWindows() {\n"
               "    if (typeof workspace.windowList === \"function\")\n"
               "        return workspace.windowList();\n"
               "    if (typeof workspace.clientList === \"function\")\n"
               "        return workspace.clientList();\n"
               "    return [];\n"
               "}\n"
               "function pdfpcOutputRect() {\n"
               "    var screens = (typeof workspace.screens !== \"undefined\") ? workspace.screens : null;\n"
               "    if (screens && screens.length) {\n"
               "        var i;\n"
               "        for (i = 0; i < screens.length; ++i) {\n"
               "            var s = screens[i];\n"
               "            if (s && outputName !== \"\" && s.name === outputName && s.geometry)\n"
               "                return s.geometry;\n"
               "        }\n"
               "        if (outputIndex >= 0 && outputIndex < screens.length && screens[outputIndex]\n"
               "            && screens[outputIndex].geometry)\n"
               "            return screens[outputIndex].geometry;\n"
               "    }\n"
               "    if (typeof workspace.clientArea === \"function\" && typeof KWin !== \"undefined\"\n"
               "        && outputIndex >= 0) {\n"
               "        var area = workspace.clientArea(KWin.ScreenArea, outputIndex, 0);\n"
               "        if (area && area.width > 0)\n"
               "            return area;\n"
               "    }\n"
               "    return null;\n"
               "}\n"
               "function pdfpcDesired() {\n"
               "    var g = pdfpcOutputRect();\n"
               "    if (g && g.width > 0 && g.height > 0) {\n"
               "        if (scale > 0) {\n"
               "            var w = Math.round(g.width * scale);\n"
               "            var h = Math.round(g.height * scale);\n"
               "            return { x: Math.round(g.x + (g.width - w) / 2),\n"
               "                     y: Math.round(g.y + (g.height - h) / 2), width: w, height: h };\n"
               "        }\n"
               "        return { x: Math.round(g.x + relX), y: Math.round(g.y + relY), width: relW, height: relH };\n"
               "    }\n"
               "    return { x: globalX, y: globalY, width: relW, height: relH };\n"
               "}\n"
               "var rect = pdfpcDesired();\n"
               "var found = false;\n"
               "var list = pdfpcWindows();\n"
               "for (var i = 0; i < list.length; ++i) {\n"
               "    var w = list[i];\n"
               "    if (w.pid === pdfpcPid && w.caption === caption) {\n"
               "        found = true;\n"
               "        if (w.fullScreen)\n"
               "            w.fullScreen = false;\n"
               "        if (w.frameGeometry) {\n"
               "            var q = Object.assign({}, w.frameGeometry);\n"
               "            q.x = rect.x; q.y = rect.y; q.width = rect.width; q.height = rect.height;\n"
               "            w.frameGeometry = q;\n"
               "        } else {\n"
               "            w.geometry = rect;\n"
               "        }\n"
               "    }\n"
               "}\n"
               "callDBus(dbusService, dbusPath, \"org.pdfpc.Place\", \"placeResult\", found ? \"ok\" : \"miss\");\n");
    script.replace(QStringLiteral("@@CAPTION@@"), jsonJsString(caption));
    script.replace(QStringLiteral("@@OUTPUT@@"), jsonJsString(outputName));
    script.replace(QStringLiteral("@@INDEX@@"), QString::number(outputIndex));
    script.replace(QStringLiteral("@@REL_X@@"), QString::number(relX));
    script.replace(QStringLiteral("@@REL_Y@@"), QString::number(relY));
    script.replace(QStringLiteral("@@REL_W@@"), QString::number(relW));
    script.replace(QStringLiteral("@@REL_H@@"), QString::number(relH));
    script.replace(QStringLiteral("@@GLOBAL_X@@"), QString::number(globalX));
    script.replace(QStringLiteral("@@GLOBAL_Y@@"), QString::number(globalY));
    script.replace(QStringLiteral("@@SCALE@@"), QString::number(centeredScale, 'f', 8));
    script.replace(QStringLiteral("@@DBUS_SERVICE@@"), jsonJsString(dbusService));
    script.replace(QStringLiteral("@@DBUS_PATH@@"), jsonJsString(dbusPath));
    script.replace(QStringLiteral("@@PID@@"), QString::number(pid));
    return script;
}
