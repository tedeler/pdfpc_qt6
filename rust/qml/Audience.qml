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
    visible: true
    width: windowed ? 960 : (screen ? screen.width : 960)
    height: windowed ? 540 : (screen ? screen.height : 540)
    visibility: windowed ? Window.Windowed : Window.FullScreen
    // Keep fully on-screen for single-display / windowed smoke tests.
    x: windowed ? 40 : 0
    y: windowed ? 520 : 0

    onScreenChanged: {
        if (!windowed && screen)
            visibility = Window.FullScreen
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
        opacity: root.ctrl.blank ? 0 : 1
    }

    Rectangle {
        anchors.fill: parent
        color: "#000000"
        opacity: root.ctrl.blank ? 1 : 0
        z: 100
    }
}
