import QtQuick
import QtQuick.Window
import QtQuick.Pdf
import org.pdfpc.app

Window {
    id: root
    required property var ctrl

    title: "pdfpc — presenter"
    color: "#1a1a1a"

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

    // Keep in sync with handleKey in session.rs and the README key table.
    readonly property var shortcuts: [
        ["→  ↓  Space  Enter  Page Down", "Next slide (also left click)"],
        ["←  ↑  Backspace  Page Up", "Previous slide (also right click)"],
        ["Home / End", "First / last slide"],
        ["g, digits, Enter", "Go to slide"],
        ["b", "Blank audience screen"],
        ["f", "Freeze audience screen"],
        ["1", "Normal mouse"],
        ["2", "Laser pointer"],
        ["+  /  -", "Larger / smaller laser pointer"],
        ["3", "Pen"],
        ["c", "Clear drawings on this slide"],
        ["p", "Pause / resume timer"],
        ["Ctrl+T", "Reset timer"],
        ["Shift+S", "Swap presenter and audience screens"],
        ["h", "Show / hide this help"],
        ["Escape", "Close help, cancel go-to, unblank, unfreeze, resume timer, normal mouse"],
        ["q", "Quit"]
    ]

    Rectangle {
        id: helpOverlay
        anchors.fill: parent
        visible: root.ctrl.help_visible
        color: "#e0101010"
        z: 1000

        // Swallow clicks so they do not reach the slide underneath.
        MouseArea {
            anchors.fill: parent
            acceptedButtons: Qt.AllButtons
            hoverEnabled: true
        }

        Rectangle {
            anchors.centerIn: parent
            width: helpColumn.width + 64
            height: helpColumn.height + 48
            color: "#202020"
            border.color: "#505050"
            radius: 8

            Column {
                id: helpColumn
                anchors.centerIn: parent
                spacing: 14

                Text {
                    text: "Keyboard shortcuts"
                    color: "#ffffff"
                    font.pixelSize: helpGrid.fontSize + 6
                    font.bold: true
                }

                Grid {
                    id: helpGrid
                    readonly property int fontSize: Math.max(12, Math.min(20, Math.round(root.height / 42)))
                    columns: 2
                    columnSpacing: 32
                    rowSpacing: 6

                    Repeater {
                        model: root.shortcuts.length * 2
                        Text {
                            readonly property var entry: root.shortcuts[Math.floor(index / 2)]
                            text: entry[index % 2]
                            color: index % 2 === 0 ? "#7ec8ff" : "#e8e8e8"
                            font.pixelSize: helpGrid.fontSize
                            font.family: index % 2 === 0 ? "DejaVu Sans Mono" : "DejaVu Sans"
                            width: index % 2 === 1 ? Math.min(implicitWidth, root.width * 0.45) : implicitWidth
                            wrapMode: Text.WordWrap
                        }
                    }
                }

                Text {
                    text: "h or Escape closes this help"
                    color: "#909090"
                    font.pixelSize: helpGrid.fontSize - 2
                }
            }
        }
    }

    Component.onCompleted: focusScope.forceActiveFocus()
    onActiveChanged: if (active) focusScope.forceActiveFocus()
}
