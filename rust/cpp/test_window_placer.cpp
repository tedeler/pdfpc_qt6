// Layout and command tests for the compositor placement backends.
// Build via scripts/test-window-placer.sh
#include "window_placer_proto.h"

#include <cstdio>

static int g_failed = 0;

static void expect(bool cond, const char *message)
{
    if (!cond) {
        std::fprintf(stderr, "FAIL: %s\n", message);
        ++g_failed;
    }
}

int main()
{
    const double scale = 0.7071067811865476; // 1/sqrt(2)
    PlacerOutput primary;
    primary.name = QStringLiteral("eDP-1");
    primary.w = 1920;
    primary.h = 1080;
    PlacerOutput side;
    side.name = QStringLiteral("HDMI-A-1");
    side.x = 1920;
    side.y = 0;
    side.w = 1920;
    side.h = 1080;

    const PlacerRect centered = rectForOutput(primary, scale, 0, 0, 0, 0, false);
    expect(centered.ok, "centered rect is valid");
    expect(centered.w == qRound(1920 * scale), "presenter width is screen/sqrt(2)");
    expect(centered.h == qRound(1080 * scale), "presenter height is screen/sqrt(2)");
    expect(centered.x == qRound((1920 - centered.w) / 2.0), "presenter is centered horizontally");
    expect(centered.y == qRound((1080 - centered.h) / 2.0), "presenter is centered vertically");

    const PlacerRect global = rectForOutput(side, scale, 0, 0, 0, 0, true);
    expect(global.x == 1920 + centered.x, "hyprland coordinates include the output origin");
    expect(global.y == centered.y, "hyprland y includes the output origin");

    const PlacerRect parked = rectForOutput(side, 0, 100, 40, 640, 360, true);
    expect(parked.x == 2020 && parked.y == 40 && parked.w == 640 && parked.h == 360,
           "explicit rect is offset by the output origin");

    const QByteArray swayJson = R"([
        {"name":"eDP-1","active":true,"rect":{"x":0,"y":0,"width":1920,"height":1080}},
        {"name":"HDMI-A-1","active":true,"rect":{"x":1920,"y":0,"width":1280,"height":720}},
        {"name":"DP-2","active":false,"rect":{"x":0,"y":0,"width":800,"height":600}}
    ])";
    const auto swayOuts = parseSwayOutputs(swayJson);
    expect(swayOuts.size() == 2, "inactive sway outputs are skipped");
    const auto hdmi = matchOutput(swayOuts, QStringLiteral("HDMI-A-1"), 99);
    expect(hdmi.name == QStringLiteral("HDMI-A-1") && hdmi.w == 1280 && hdmi.x == 1920,
           "sway output is matched by name");
    const auto byIndex = matchOutput(swayOuts, QStringLiteral("missing"), 1);
    expect(byIndex.name == QStringLiteral("HDMI-A-1"), "output index is the fallback");
    expect(!matchOutput(swayOuts, QString(), 5).valid(), "a bad index yields no output");

    const QByteArray hyprJson = R"([
        {"name":"DP-1","x":0,"y":0,"width":2560,"height":1440,"disabled":false},
        {"name":"DP-2","x":2560,"y":0,"width":1920,"height":1080,"disabled":true}
    ])";
    const auto hyprOuts = parseHyprMonitors(hyprJson);
    expect(hyprOuts.size() == 1 && hyprOuts.at(0).w == 2560, "disabled hyprland monitors are skipped");

    const QByteArray frame = i3IpcFrame(0, QByteArray("floating enable"));
    expect(frame.left(6) == "i3-ipc", "i3 frame magic");
    expect(quint8(frame.at(6)) == 15 && frame.at(7) == 0, "i3 payload length is little-endian");
    expect(frame.mid(14) == "floating enable", "i3 payload follows the header");
    expect(i3CommandSucceeded(R"([{"success":true},{"success":true}])"), "i3 success array");
    expect(!i3CommandSucceeded(R"([{"success":true},{"success":false}])"), "one failed i3 command fails the batch");
    expect(!i3CommandSucceeded(QByteArray("not json")), "non-json i3 reply fails");

    const QString title = QStringLiteral("pdfpc-qt6 — presenter");
    const auto place = i3PlaceCommands(4242, title, hdmi.name, rectForOutput(hdmi, scale, 0, 0, 0, 0, false));
    expect(place.size() == 4, "sway place is four commands");
    expect(place.at(0).contains(QStringLiteral("pid=4242")) && place.at(0).contains(title),
           "sway commands select this process and the presenter title");
    expect(place.at(0).contains(QStringLiteral("move container to output \"HDMI-A-1\"")),
           "sway moves onto the named output");
    expect(place.at(1).endsWith(QStringLiteral("floating enable")), "sway floats the presenter");
    expect(place.at(2).contains(QStringLiteral("resize set")) && place.at(3).contains(QStringLiteral("move position")),
           "sway resizes then positions relative to the output");
    expect(i3PlaceCommands(4242, QStringLiteral("bad]title"), hdmi.name, centered).isEmpty(),
           "a criteria-breaking title is refused");
    expect(i3MoveToOutputCommand(4242, title, QStringLiteral("DP-1;exec malicious")).isEmpty(),
           "an output name cannot append another i3 command");
    expect(i3Quote(QStringLiteral("a\"b\\c")) == QStringLiteral("a\\\"b\\\\c"), "i3 quotes are escaped");

    const auto hypr = hyprPlaceCommands(title, rectForOutput(side, scale, 0, 0, 0, 0, true));
    expect(hypr.size() == 3, "hyprland place is three dispatches");
    expect(hypr.at(0).startsWith(QStringLiteral("dispatch setfloating title:^")), "hyprland floats by title");
    expect(hypr.at(0).contains(QStringLiteral("pdfpc-qt6 — presenter")), "hyprland title keeps the em dash");
    expect(hypr.at(0).endsWith(QLatin1Char('$')), "hyprland title match is anchored");
    expect(hypr.at(2).contains(QStringLiteral("movewindowpixel exact")), "hyprland moves in global pixels");
    expect(hypr.at(2).contains(QString::number(global.x)), "hyprland move uses the global x");
    const auto claim = hyprClaimCommands(QStringLiteral("pdfpc-qt6 — audience"), QStringLiteral("DP-1"));
    expect(claim.size() == 2 && claim.at(1) == QStringLiteral("dispatch movewindow mon:DP-1"),
           "hyprland claim focuses then moves to the monitor");
    expect(hyprClaimCommands(title, QStringLiteral("bad name")).isEmpty(), "hyprland refuses output names with spaces");
    expect(hyprDispatchOk("ok\n"), "hyprland ok");
    expect(!hyprDispatchOk("Window not found"), "hyprland errors are not success");

    const QString script = kwinPlaceScript(title, 4242, QStringLiteral("HDMI-A-1"), 1, 10, 20, 800, 600, 1930, 20,
                                           scale, QStringLiteral(":1.7"), QStringLiteral("/pdfpc/place3"));
    expect(script.contains(QStringLiteral("workspace.windowList")) && script.contains(QStringLiteral("workspace.clientList")),
           "kwin script supports Plasma 6 and Plasma 5 window lists");
    expect(script.contains(QStringLiteral("Object.assign")) && script.contains(QStringLiteral("w.frameGeometry = q")),
           "kwin script assigns the whole frame rect");
    expect(script.contains(QStringLiteral("w.geometry = rect")), "kwin script falls back to Plasma 5 geometry");
    expect(script.contains(QStringLiteral("KWin.ScreenArea")), "kwin script can ask Plasma 5 for the output rect");
    expect(script.contains(QStringLiteral("pdfpcPid === 4242")) == false && script.contains(QStringLiteral("w.pid === pdfpcPid")),
           "kwin pid is a variable, not a placeholder");
    expect(script.contains(QStringLiteral("pdfpcPid = 4242")), "kwin script carries the pid");
    expect(script.contains(QStringLiteral("\"HDMI-A-1\"")) && script.contains(QStringLiteral("\"pdfpc-qt6 — presenter\"")),
           "kwin strings are JSON-encoded");
    expect(script.contains(QStringLiteral("callDBus")) && script.contains(QStringLiteral("org.pdfpc.Place")),
           "kwin script reports whether it found the window");
    expect(!script.contains(QStringLiteral("@@")), "kwin script has no unsubstituted tokens");
    expect(script.contains(QStringLiteral(":1.7")) && script.contains(QStringLiteral("/pdfpc/place3")),
           "kwin script calls back on the unique bus name");

    const QString hostile = kwinPlaceScript(QStringLiteral("a\"b\\c"), 1, QStringLiteral("DP-1"), 0, 0, 0, 1, 1, 0, 0,
                                            0, QStringLiteral(":1.1"), QStringLiteral("/pdfpc/place1"));
    expect(hostile.contains(QStringLiteral("\"a\\\"b\\\\c\"")), "kwin caption quotes are escaped");

    if (g_failed) {
        std::fprintf(stderr, "%d window-placer checks failed\n", g_failed);
        return 1;
    }
    std::printf("window placer checks passed\n");
    return 0;
}
