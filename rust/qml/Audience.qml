import QtQuick
import QtQuick.Window
import QtQuick.Pdf
import org.pdfpc.app

Window {
    id: root
    required property var ctrl

    // app_version is read from ../VERSION at build time (PDFPC_VERSION).
    title: "pdfpc-qt6 " + ctrl.app_version + " — audience"
    color: "#000000"

    Item {
        id: keyTarget
        anchors.fill: parent
        focus: true

        Keys.onPressed: function (event) {
            root.ctrl.handleKey(event.key, event.modifiers, event.text)
            event.accepted = true
        }
    }

    SlideView {
        id: slide
        anchors.fill: parent
        pdfSource: root.ctrl.pdf_url
        page: root.ctrl.audience_page
        strokesJson: root.ctrl.audience_strokes_json
        laserTrailsJson: (!root.ctrl.blank && !root.ctrl.frozen) ? root.ctrl.laser_trails_json : "[]"
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

    onActiveChanged: if (active) keyTarget.forceActiveFocus()
}
