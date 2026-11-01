import QtQuick

QtObject {
    readonly property int desktopCount: 0

    function needsDeferredMapping(screenCount) {
        return false
    }

    // Return "native", "placed", or "fullscreen" when a compositor needs a
    // temporary fullscreen request to move the presenter between outputs.
    function preparePresenterPlacement(window, screen, geometry, screenCount) {
        return "native"
    }

    function moveToDesktop(number) {
        return false
    }
}
