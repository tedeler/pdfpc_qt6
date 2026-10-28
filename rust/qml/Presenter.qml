import QtQuick
import QtQuick.Controls
import QtQuick.Dialogs
import QtQuick.Window
import QtQuick.Pdf
import org.pdfpc.app

ApplicationWindow {
    id: root
    required property var ctrl

    // app_version is read from ../VERSION at build time (PDFPC_VERSION).
    title: "pdfpc-qt6 " + ctrl.app_version + " — presenter"
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
        // Pin size so org.kde.breeze Dialog.qml does not bind
        // implicitWidth ↔ contentItem in a loop.
        implicitWidth: 360
        width: Math.min(implicitWidth, Overlay.overlay ? Overlay.overlay.width - 48 : implicitWidth)
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
            if (root.ctrl.recent_visible) {
                if (event.key === Qt.Key_Down) {
                    if (recentList.count > 0)
                        recentList.currentIndex = Math.min(recentList.count - 1, recentList.currentIndex + 1)
                    event.accepted = true
                    return
                }
                if (event.key === Qt.Key_Up) {
                    if (recentList.count > 0)
                        recentList.currentIndex = Math.max(0, recentList.currentIndex - 1)
                    event.accepted = true
                    return
                }
                if (event.key === Qt.Key_Return || event.key === Qt.Key_Enter) {
                    if (recentList.currentIndex >= 0 && recentList.model[recentList.currentIndex])
                        root.ctrl.loadPdf(recentList.model[recentList.currentIndex].path)
                    event.accepted = true
                    return
                }
            }
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
        ["Ctrl+G", "Jump to frozen page (while frozen)"],
        ["b", "Blank audience screen"],
        ["f", "Freeze audience screen"],
        ["1", "Normal mouse"],
        ["2", "Laser pointer (drag = afterglow; Shift+drag = straight)"],
        ["+  /  -", "Larger / smaller laser pointer"],
        ["3", "Pen"],
        ["c", "Clear drawings on this slide"],
        ["s", "Start timer"],
        ["p", "Pause / resume timer"],
        ["Ctrl+T", "Reset timer"],
        ["Shift+S", "Swap presenter and audience screens"],
        ["F3", "Open recent files / choose PDF"],
        ["h", "Show / hide this help"],
        ["Escape", "Close recent/help, cancel go-to, unblank, unfreeze, resume timer, normal mouse"]
    ]

    function recentEntries() {
        try {
            return JSON.parse(root.ctrl.recent_files_json)
        } catch (e) {
            return []
        }
    }

    FileDialog {
        id: openPdfDialog
        title: qsTr("Open PDF")
        nameFilters: [qsTr("PDF files (*.pdf)"), qsTr("All files (*)")]
        onAccepted: {
            root.ctrl.loadPdf("" + selectedFile)
            focusScope.forceActiveFocus()
        }
        onRejected: focusScope.forceActiveFocus()
    }

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

    Rectangle {
        id: recentOverlay
        anchors.fill: parent
        visible: root.ctrl.recent_visible
        color: "#e0101010"
        z: 1001

        MouseArea {
            anchors.fill: parent
            acceptedButtons: Qt.AllButtons
            hoverEnabled: true
        }

        Rectangle {
            anchors.centerIn: parent
            width: Math.min(parent.width * 0.72, 640)
            height: Math.min(parent.height * 0.78, 480)
            color: "#202020"
            border.color: "#505050"
            radius: 8

            Column {
                anchors.fill: parent
                anchors.margins: 20
                spacing: 12

                Text {
                    text: qsTr("Open PDF")
                    color: "#ffffff"
                    font.pixelSize: Math.max(16, Math.round(root.height / 36))
                    font.bold: true
                }

                Text {
                    width: parent.width
                    text: qsTr("Recently opened files")
                    color: "#a0a0a0"
                    font.pixelSize: Math.max(12, Math.round(root.height / 48))
                }

                Rectangle {
                    width: parent.width
                    height: parent.height - 120
                    color: "#161616"
                    radius: 4
                    border.color: "#404040"

                    ListView {
                        id: recentList
                        anchors.fill: parent
                        anchors.margins: 4
                        clip: true
                        model: root.ctrl.recent_visible ? root.recentEntries() : []
                        currentIndex: count > 0 ? 0 : -1
                        // Keep keyboard focus on focusScope so F3/Escape/arrows
                        // reach Keys.onPressed (ListView focus would swallow them).
                        focus: false
                        activeFocusOnTab: false

                        delegate: Item {
                            width: recentList.width
                            height: Math.max(40, Math.round(root.height / 22))

                            required property var modelData
                            required property int index

                            Rectangle {
                                anchors.fill: parent
                                anchors.margins: 1
                                color: recentList.currentIndex === index ? "#2a4a6a" : "transparent"
                                radius: 3

                                Column {
                                    anchors.verticalCenter: parent.verticalCenter
                                    anchors.left: parent.left
                                    anchors.leftMargin: 10
                                    anchors.right: pageLabel.left
                                    anchors.rightMargin: 8
                                    spacing: 2

                                    Text {
                                        width: parent.width
                                        text: modelData.name
                                        color: "#f0f0f0"
                                        elide: Text.ElideMiddle
                                        font.pixelSize: Math.max(13, Math.round(root.height / 48))
                                    }
                                    Text {
                                        width: parent.width
                                        text: modelData.path
                                        color: "#808080"
                                        elide: Text.ElideMiddle
                                        font.pixelSize: Math.max(10, Math.round(root.height / 58))
                                        font.family: "DejaVu Sans Mono"
                                    }
                                }

                                Text {
                                    id: pageLabel
                                    anchors.right: parent.right
                                    anchors.rightMargin: 12
                                    anchors.verticalCenter: parent.verticalCenter
                                    text: qsTr("p. %1").arg(modelData.page + 1)
                                    color: "#7ec8ff"
                                    font.pixelSize: Math.max(12, Math.round(root.height / 50))
                                }

                                MouseArea {
                                    anchors.fill: parent
                                    onClicked: recentList.currentIndex = index
                                    onDoubleClicked: root.ctrl.loadPdf(modelData.path)
                                }
                            }
                        }

                        Text {
                            anchors.centerIn: parent
                            visible: recentList.count === 0
                            text: qsTr("No recent files yet")
                            color: "#707070"
                            font.pixelSize: 14
                        }
                    }
                }

                Row {
                    spacing: 12

                    Rectangle {
                        width: openSelectedText.implicitWidth + 24
                        height: 36
                        radius: 4
                        color: recentList.currentIndex >= 0 ? "#2f8fd8" : "#404040"

                        Text {
                            id: openSelectedText
                            anchors.centerIn: parent
                            text: qsTr("Open")
                            color: "#ffffff"
                            font.pixelSize: 14
                        }

                        MouseArea {
                            anchors.fill: parent
                            enabled: recentList.currentIndex >= 0
                            onClicked: {
                                var item = recentList.model[recentList.currentIndex]
                                if (item)
                                    root.ctrl.loadPdf(item.path)
                            }
                        }
                    }

                    Rectangle {
                        width: browseText.implicitWidth + 24
                        height: 36
                        radius: 4
                        color: "#3a3a3a"

                        Text {
                            id: browseText
                            anchors.centerIn: parent
                            text: qsTr("Browse…")
                            color: "#ffffff"
                            font.pixelSize: 14
                        }

                        MouseArea {
                            anchors.fill: parent
                            onClicked: openPdfDialog.open()
                        }
                    }

                    Text {
                        anchors.verticalCenter: parent.verticalCenter
                        text: ("" + root.ctrl.pdf_url).length > 0
                              ? qsTr("Escape continues the current PDF")
                              : qsTr("Choose a file to start")
                        color: "#808080"
                        font.pixelSize: 12
                    }
                }
            }
        }
    }

    Connections {
        target: root.ctrl
        function onRecent_visibleChanged() {
            if (root.ctrl.recent_visible)
                focusScope.forceActiveFocus()
        }
    }

    Component.onCompleted: focusScope.forceActiveFocus()
    onActiveChanged: if (active) focusScope.forceActiveFocus()
}
