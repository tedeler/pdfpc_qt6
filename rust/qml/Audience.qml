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
    x: windowed ? 1300 : 0
    y: windowed ? 0 : 0

    onScreenChanged: {
        if (!windowed && screen)
            visibility = Window.FullScreen
    }

    SlideView {
        anchors.fill: parent
        pdfSource: root.ctrl.pdf_url
        page: root.ctrl.audience_page
        strokesJson: root.ctrl.audience_strokes_json
        pointerVisible: root.ctrl.pointer_visible && root.ctrl.mode === 1 && !root.ctrl.blank && !root.ctrl.frozen
        pointerX: root.ctrl.pointer_x
        pointerY: root.ctrl.pointer_y
        interactive: false
        ctrl: root.ctrl
        visible: !root.ctrl.blank
    }

    Rectangle {
        anchors.fill: parent
        color: "#000000"
        visible: root.ctrl.blank
        z: 10
    }
}
