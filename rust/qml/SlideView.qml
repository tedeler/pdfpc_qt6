import QtQuick
import QtQuick.Pdf
import org.pdfpc.app

Item {
    id: root

    required property var ctrl
    property url pdfSource: ""
    property int page: 0
    property string strokesJson: "[]"
    property bool pointerVisible: false
    property real pointerX: -1
    property real pointerY: -1
    property bool interactive: false

    // Per-view document so currentFrame (page) does not fight a shared renderer.
    PdfDocument {
        id: localDocument
        source: root.pdfSource
    }

    PdfPageImage {
        id: pageImage
        anchors.fill: parent
        document: localDocument
        currentFrame: Math.max(0, root.page)
        fillMode: Image.PreserveAspectFit
        asynchronous: true
        visible: root.page >= 0 && localDocument.status === PdfDocument.Ready
    }

    InkCanvas {
        id: ink
        x: pageImage.x + (pageImage.width - pageImage.paintedWidth) / 2
        y: pageImage.y + (pageImage.height - pageImage.paintedHeight) / 2
        width: Math.max(1, pageImage.paintedWidth)
        height: Math.max(1, pageImage.paintedHeight)
        strokesJson: root.strokesJson
        pointerVisible: root.pointerVisible
        pointerX: root.pointerX
        pointerY: root.pointerY
        visible: root.page >= 0
    }

    MouseArea {
        anchors.fill: ink
        enabled: root.interactive
        acceptedButtons: Qt.LeftButton | Qt.RightButton
        hoverEnabled: root.interactive && root.ctrl.mode === 1

        function toPage(mx, my) {
            return Qt.point(mx / Math.max(1, ink.width), my / Math.max(1, ink.height))
        }

        onClicked: function (mouse) {
            if (root.ctrl.mode === 0)
                root.ctrl.mouseClick(mouse.button)
        }

        onPositionChanged: function (mouse) {
            var p = toPage(mouse.x, mouse.y)
            if (root.ctrl.mode === 1)
                root.ctrl.pointerMove(p.x, p.y)
            else if (root.ctrl.mode === 2 && (mouse.buttons & Qt.LeftButton))
                root.ctrl.strokeAdd(p.x, p.y)
        }

        onPressed: function (mouse) {
            var p = toPage(mouse.x, mouse.y)
            if (root.ctrl.mode === 2 && mouse.button === Qt.LeftButton)
                root.ctrl.strokeBegin(p.x, p.y)
            else if (root.ctrl.mode === 1)
                root.ctrl.pointerMove(p.x, p.y)
        }

        onReleased: function (mouse) {
            if (root.ctrl.mode === 2)
                root.ctrl.strokeEnd()
        }

        onExited: {
            if (root.ctrl.mode === 1)
                root.ctrl.pointerHide()
        }
    }
}
