import QtQuick
import org.pdfpc.app

QtObject {
    property KWinPlacer kwin: KWinPlacer {}

    readonly property int desktopCount: kwin.available ? kwin.desktopCount() : 0
    readonly property bool wayland: Qt.platform.pluginName.indexOf("wayland") === 0

    // Wayland clients cannot position normal windows. KWin can do so with a
    // short-lived script; other compositors need the fullscreen fallback.
    function needsDeferredMapping(screenCount) {
        return wayland && screenCount > 1
    }

    function preparePresenterPlacement(window, screen, geometry, screenCount) {
        if (!wayland || !screen || screenCount < 2)
            return "native"

        if (kwin.available && kwin.setFrameGeometry(window.title, geometry))
            return "placed"

        return "fullscreen"
    }

    function moveToDesktop(number) {
        return kwin.moveToDesktop(number)
    }
}
