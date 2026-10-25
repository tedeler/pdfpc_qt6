import QtQuick
import QtQuick.Window
import QtQuick.Pdf
import org.pdfpc.app

Window {
    id: root
    required property var ctrl
    property bool windowed: false

    title: "pdfpc — audience"
    color: "#000000"
    // Start mapped as a normal window; fullscreen is requested after show on Wayland.
    visible: false
    width: windowed ? 640 : (screen ? Math.max(640, screen.width) : 960)
    height: windowed ? 480 : (screen ? Math.max(480, screen.height) : 540)
    visibility: Window.Windowed
    x: windowed ? 1280 : 0
    y: 0

    function applyPlacement() {
        if (root.windowed) {
            root.visibility = Window.Windowed
            root.width = 640
            root.height = 480
            root.x = 1280
            root.y = 0
        } else if (root.screen) {
            root.visibility = Window.FullScreen
        }
        root.visible = true
        root.raise()
    }

    onScreenChanged: {
        if (root.visible && !root.windowed && root.screen)
            root.visibility = Window.FullScreen
    }

    onWindowedChanged: {
        if (root.visible)
            root.applyPlacement()
    }

    SlideView {
        id: slide
        anchors.fill: parent
        pdfSource: root.ctrl.pdf_url
        page: root.ctrl.audience_page
        strokesJson: root.ctrl.audience_strokes_json
        pointerVisible: root.ctrl.pointer_visible && root.ctrl.mode === 1 && !root.ctrl.blank && !root.ctrl.frozen
        pointerX: root.ctrl.pointer_x
        pointerY: root.ctrl.pointer_y
        interactive: false
        ctrl: root.ctrl
        // Opacity (not visible) so blank overlays reliably under Qt Quick / Wayland.
        opacity: root.ctrl.blank ? 0 : 1
    }

    Rectangle {
        anchors.fill: parent
        color: "#000000"
        opacity: root.ctrl.blank ? 1 : 0
        z: 100
    }
}
