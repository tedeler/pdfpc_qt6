import QtQuick
import QtQuick.Controls
import QtQuick.Window
import QtQuick.Pdf
import org.pdfpc.app

ApplicationWindow {
    id: root
    required property var ctrl

    title: "pdfpc-qt6 — presenter"
    color: "#1a1a1a"

    menuBar: MenuBar {
        Menu {
            title: qsTr("&Help")
            Action {
                text: qsTr("&About pdfpc-qt6")
                onTriggered: aboutDialog.open()
            }
        }
    }

    Dialog {
        id: aboutDialog
        title: qsTr("About pdfpc-qt6")
        modal: true
        anchors.centerIn: Overlay.overlay
        standardButtons: Dialog.Ok
        onClosed: focusScope.forceActiveFocus()

        contentItem: Column {
            spacing: 10
            padding: 4

            Label {
                text: "pdfpc-qt6"
                font.pixelSize: 20
                font.bold: true
            }
            Label {
                text: qsTr("Version %1").arg(root.ctrl.app_version)
            }
            Label {
                text: qsTr("Git %1").arg(root.ctrl.git_id)
                font.family: "DejaVu Sans Mono"
            }
            Label {
                text: "GPL-3.0-or-later"
                color: palette.placeholderText
            }
        }
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
                    laserTrailsJson: root.ctrl.laser_trails_json
                    pointerVisible: root.ctrl.pointer_visible && root.ctrl.mode === 1
                    pointerX: root.ctrl.pointer_x
                    pointerY: root.ctrl.pointer_y
                    interactive: true
                    ctrl: root.ctrl

                    // The audience does not see this slide: blanked or frozen.
                    Rectangle {
                        x: currentSlide.pageX - 4
                        y: currentSlide.pageY - 4
                        width: currentSlide.pageW + 8
                        height: currentSlide.pageH + 8
                        color: "transparent"
                        border.width: 4
                        border.color: root.ctrl.blank ? "#808080" : "#2f8fd8"
                        visible: root.ctrl.blank || root.ctrl.frozen
                        z: 10
                    }
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
                readonly property int fontSize: Math.round(height * 0.5)
                width: parent.width
                height: Math.max(40, Math.round(root.height * 0.08))
                color: "#111111"
                radius: 4

                Text {
                    id: pageText
                    anchors.left: parent.left
                    anchors.leftMargin: 16
                    anchors.verticalCenter: parent.verticalCenter
                    text: root.ctrl.goto_active ? "Go to: " + root.ctrl.goto_buffer + "_"
                                                : "Slide " + root.ctrl.page_label
                    color: root.ctrl.goto_active ? "#7ec8ff" : "#f0f0f0"
                    font.pixelSize: bottomBar.fontSize
                    font.family: "DejaVu Sans"
                }

                Text {
                    id: timerText
                    anchors.centerIn: parent
                    text: root.ctrl.timer_text
                    color: root.ctrl.timer_paused ? "#f0c040" : "#f0f0f0"
                    font.pixelSize: Math.round(bottomBar.fontSize * 1.25)
                    font.family: "DejaVu Sans Mono"
                    font.bold: true
                }

                Text {
                    anchors.left: timerText.right
                    anchors.leftMargin: 12
                    anchors.verticalCenter: parent.verticalCenter
                    visible: root.ctrl.timer_paused
                    text: "paused"
                    color: "#f0c040"
                    font.pixelSize: Math.round(bottomBar.fontSize * 0.7)
                }

                Row {
                    anchors.right: clockText.left
                    anchors.rightMargin: 20
                    anchors.verticalCenter: parent.verticalCenter
                    spacing: 8

                    Repeater {
                        model: [
                            { on: root.ctrl.blank, label: "BLANK", color: "#808080" },
                            { on: root.ctrl.frozen, label: "FROZEN", color: "#2f8fd8" },
                            { on: root.ctrl.mode === 1, label: "LASER", color: "#d83030" },
                            { on: root.ctrl.mode === 2, label: "PEN", color: "#d83030" }
                        ]
                        Rectangle {
                            visible: modelData.on
                            width: badgeText.implicitWidth + 16
                            height: bottomBar.height * 0.62
                            radius: 4
                            color: modelData.color
                            Text {
                                id: badgeText
                                anchors.centerIn: parent
                                text: modelData.label
                                color: "#ffffff"
                                font.pixelSize: Math.round(bottomBar.fontSize * 0.7)
                                font.bold: true
                            }
                        }
                    }
                }

                Text {
                    id: clockText
                    property string now: Qt.formatTime(new Date(), "hh:mm")
                    anchors.right: parent.right
                    anchors.rightMargin: 16
                    anchors.verticalCenter: parent.verticalCenter
                    text: now
                    color: "#a0a0a0"
                    font.pixelSize: bottomBar.fontSize
                    font.family: "DejaVu Sans Mono"

                    Timer {
                        interval: 1000
                        running: true
                        repeat: true
                        onTriggered: clockText.now = Qt.formatTime(new Date(), "hh:mm")
                    }
                }
            }
        }
    }

    // Keep in sync with handleKey in session.rs and the README key table.
    readonly property var shortcuts: [
        ["→  ↓  Space  Enter  Page Down", "Next slide (also left click; laser too)"],
        ["←  ↑  Backspace  Page Up", "Previous slide (also right click; laser too)"],
        ["Home / End", "First / last slide"],
        ["g, digits, Enter", "Go to slide"],
        ["b", "Blank audience screen"],
        ["f", "Freeze audience screen"],
        ["1", "Normal mouse"],
        ["2", "Laser pointer (drag = afterglow trail)"],
        ["+  /  -", "Larger / smaller laser pointer"],
        ["3", "Pen"],
        ["c", "Clear drawings on this slide"],
        ["s", "Start timer"],
        ["p", "Pause / resume timer"],
        ["Ctrl+T", "Reset timer"],
        ["Shift+S", "Swap presenter and audience screens"],
        ["h", "Show / hide this help"],
        ["Escape", "Close help, cancel go-to, unblank, unfreeze, resume timer, normal mouse"]
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
