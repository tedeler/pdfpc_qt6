import QtQuick
import QtQuick.Window
import QtQuick.Pdf
import org.pdfpc.app

Window {
    id: root
    required property var ctrl
    property bool windowed: false

    title: "pdfpc — presenter"
    color: "#1a1a1a"
    // Start mapped as a normal window; fullscreen is requested after show on Wayland.
    visible: false
    width: windowed ? 1280 : (screen ? Math.max(800, screen.width) : 1280)
    height: windowed ? 800 : (screen ? Math.max(600, screen.height) : 800)
    visibility: Window.Windowed
    x: 0
    y: 0

    function applyPlacement() {
        if (root.windowed) {
            root.visibility = Window.Windowed
            root.width = 1280
            root.height = 800
            root.x = 0
            root.y = 0
        } else if (root.screen) {
            root.visibility = Window.FullScreen
        }
        root.visible = true
        root.raise()
        root.requestActivate()
    }

    onScreenChanged: {
        if (root.visible && !root.windowed && root.screen)
            root.visibility = Window.FullScreen
    }

    onWindowedChanged: {
        if (root.visible)
            root.applyPlacement()
    }

    FocusScope {
        id: focusScope
        anchors.fill: parent
        focus: true

        Keys.onPressed: function (event) {
            root.ctrl.handleKey(event.key, event.modifiers, event.text)
            event.accepted = true
        }

        Column {
            anchors.fill: parent
            anchors.margins: 16
            spacing: 12

            Row {
                id: topRow
                width: parent.width
                height: parent.height - bottomBar.height - 12
                spacing: 12

                SlideView {
                    id: currentSlide
                    width: parent.width * 0.72
                    height: parent.height
                    pdfSource: root.ctrl.pdf_url
                    page: root.ctrl.current_page
                    strokesJson: root.ctrl.current_strokes_json
                    pointerVisible: root.ctrl.pointer_visible && root.ctrl.mode === 1
                    pointerX: root.ctrl.pointer_x
                    pointerY: root.ctrl.pointer_y
                    interactive: true
                    ctrl: root.ctrl
                }

                SlideView {
                    id: nextSlide
                    width: parent.width * 0.28
                    height: parent.height * 0.45
                    anchors.verticalCenter: parent.verticalCenter
                    pdfSource: root.ctrl.pdf_url
                    page: root.ctrl.next_page
                    strokesJson: "[]"
                    pointerVisible: false
                    interactive: false
                    ctrl: root.ctrl
                    opacity: root.ctrl.next_page >= 0 ? 1.0 : 0.35
                }
            }

            Rectangle {
                id: bottomBar
                width: parent.width
                height: 48
                color: "#111111"
                radius: 4

                Row {
                    anchors.fill: parent
                    anchors.margins: 12
                    spacing: 24

                    Text {
                        text: root.ctrl.page_label
                        color: "#f0f0f0"
                        font.pixelSize: 22
                        font.family: "DejaVu Sans"
                        verticalAlignment: Text.AlignVCenter
                        height: parent.height
                    }

                    Text {
                        text: root.ctrl.timer_text + (root.ctrl.timer_paused ? " (paused)" : "")
                        color: root.ctrl.timer_paused ? "#f0c040" : "#f0f0f0"
                        font.pixelSize: 22
                        font.family: "DejaVu Sans Mono"
                        verticalAlignment: Text.AlignVCenter
                        height: parent.height
                    }

                    Text {
                        visible: root.ctrl.goto_active
                        text: "Go to: " + root.ctrl.goto_buffer
                        color: "#7ec8ff"
                        font.pixelSize: 22
                        font.family: "DejaVu Sans"
                        verticalAlignment: Text.AlignVCenter
                        height: parent.height
                    }

                    Text {
                        visible: root.ctrl.mode !== 0
                        text: root.ctrl.mode === 1 ? "Laser" : "Pen"
                        color: "#ff6666"
                        font.pixelSize: 20
                        font.family: "DejaVu Sans"
                        verticalAlignment: Text.AlignVCenter
                        height: parent.height
                    }

                    Text {
                        visible: root.ctrl.frozen
                        text: "Frozen"
                        color: "#66ccff"
                        font.pixelSize: 20
                        verticalAlignment: Text.AlignVCenter
                        height: parent.height
                    }

                    Text {
                        visible: root.ctrl.blank
                        text: "Blank"
                        color: "#aaaaaa"
                        font.pixelSize: 20
                        verticalAlignment: Text.AlignVCenter
                        height: parent.height
                    }
                }
            }
        }
    }

    Component.onCompleted: focusScope.forceActiveFocus()
    onActiveChanged: if (active) focusScope.forceActiveFocus()
}
