import QtQuick
import QtQuick.Pdf
import org.pdfpc.app

Item {
    id: root

    required property var ctrl
    property url pdfSource: ""
    property int page: 0
    property string strokesJson: "[]"
    property string laserTrailsJson: "[]"
    property bool pointerVisible: false
    property real pointerX: -1
    property real pointerY: -1
    property bool interactive: false

    // Per-view document so currentFrame (page) does not fight a shared renderer.
    PdfDocument {
        id: localDocument
        source: root.pdfSource

        onStatusChanged: function () {
            if (status === PdfDocument.Ready && pageCount > 0 && root.ctrl)
                root.ctrl.applyPageCount(pageCount)
            // Remount the rasterizer once Ready so currentFrame is applied at
            // creation time (PdfPageImage often ignores a pre-set frame).
            if (status === PdfDocument.Ready)
                root.bumpRaster()
        }
        onPageCountChanged: function () {
            if (status === PdfDocument.Ready && pageCount > 0 && root.ctrl)
                root.ctrl.applyPageCount(pageCount)
        }
        onSourceChanged: root.bumpRaster()
    }

    function bumpRaster() {
        // Tear down and recreate PdfPageImage so currentFrame is honored.
        pageLoader.active = false
        Qt.callLater(function () {
            pageLoader.active = root.pageValid
        })
    }

    readonly property bool pageValid: root.page >= 0
            && localDocument.status === PdfDocument.Ready
            && localDocument.pageCount > 0
    readonly property int frameIndex: root.pageValid
            ? Math.max(0, Math.min(root.page, localDocument.pageCount - 1))
            : 0

    onFrameIndexChanged: {
        // Page changes after Ready: recreate so next-preview and restores never
        // stick on a stale decoded frame.
        if (root.pageValid)
            root.bumpRaster()
    }

    onPageValidChanged: {
        if (root.pageValid)
            root.bumpRaster()
        else
            pageLoader.active = false
    }

    readonly property size pointSize: root.pageValid
                                      ? localDocument.pagePointSize(root.frameIndex)
                                      : Qt.size(4, 3)
    readonly property real fitScale: Math.min(width / Math.max(1, pointSize.width),
                                              height / Math.max(1, pointSize.height))

    readonly property real pageX: pageLoader.item
            ? pageLoader.item.x + (pageLoader.item.width - pageLoader.item.paintedWidth) / 2
            : 0
    readonly property real pageY: pageLoader.item
            ? pageLoader.item.y + (pageLoader.item.height - pageLoader.item.paintedHeight) / 2
            : 0
    readonly property real pageW: pageLoader.item ? Math.max(1, pageLoader.item.paintedWidth) : 1
    readonly property real pageH: pageLoader.item ? Math.max(1, pageLoader.item.paintedHeight) : 1

    Rectangle {
        id: paper
        x: root.pageX
        y: root.pageY
        width: root.pageW
        height: root.pageH
        color: "#ffffff"
        visible: pageLoader.item && pageLoader.item.visible && pageLoader.item.paintedWidth > 0
        z: 0
    }

    Loader {
        id: pageLoader
        anchors.fill: parent
        active: false
        sourceComponent: pageComponent
        z: 1
    }

    Component {
        id: pageComponent
        PdfPageImage {
            anchors.fill: parent
            document: localDocument
            currentFrame: root.frameIndex
            fillMode: Image.PreserveAspectFit
            sourceSize.width: Math.max(1, Math.round(root.pointSize.width * root.fitScale * Screen.devicePixelRatio))
            sourceSize.height: Math.max(1, Math.round(root.pointSize.height * root.fitScale * Screen.devicePixelRatio))
            asynchronous: false
            visible: status === Image.Ready && paintedWidth > 0

            Component.onCompleted: {
                if ("retainWhileLoading" in this)
                    retainWhileLoading = true
            }
        }
    }

    InkCanvas {
        id: ink
        x: root.pageX
        y: root.pageY
        width: root.pageW
        height: root.pageH
        strokesJson: root.strokesJson
        laserTrailsJson: root.laserTrailsJson
        pointerVisible: root.pointerVisible
        pointerX: root.pointerX
        pointerY: root.pointerY
        pointerSize: root.ctrl.pointer_size
        visible: root.pageValid && paper.visible
        z: 2
    }

    MouseArea {
        anchors.fill: ink
        enabled: root.interactive
        acceptedButtons: Qt.LeftButton | Qt.RightButton
        hoverEnabled: root.interactive && root.ctrl.mode === 1
        cursorShape: root.ctrl.mode === 1 ? Qt.BlankCursor : Qt.ArrowCursor

        property real pressX: 0
        property real pressY: 0
        property bool laserDragged: false
        readonly property real clickSlop: 0.012

        function toPage(mx, my) {
            return Qt.point(mx / Math.max(1, ink.width), my / Math.max(1, ink.height))
        }

        function pageDist(ax, ay, bx, by) {
            var dx = ax - bx
            var dy = ay - by
            return Math.sqrt(dx * dx + dy * dy)
        }

        onClicked: function (mouse) {
            if (root.ctrl.mode === 0)
                root.ctrl.mouseClick(mouse.button)
        }

        onPositionChanged: function (mouse) {
            var p = toPage(mouse.x, mouse.y)
            if (root.ctrl.mode === 1) {
                root.ctrl.pointerMove(p.x, p.y)
                if (mouse.buttons & Qt.LeftButton) {
                    if (!laserDragged
                            && pageDist(pressX, pressY, p.x, p.y) >= clickSlop) {
                        laserDragged = true
                        root.ctrl.laserTrailBegin(pressX, pressY)
                        root.ctrl.laserTrailAdd(p.x, p.y)
                    } else if (laserDragged) {
                        root.ctrl.laserTrailAdd(p.x, p.y)
                    }
                }
            } else if (root.ctrl.mode === 2 && (mouse.buttons & Qt.LeftButton)) {
                root.ctrl.strokeAdd(p.x, p.y)
            }
        }

        onPressed: function (mouse) {
            var p = toPage(mouse.x, mouse.y)
            if (root.ctrl.mode === 2 && mouse.button === Qt.LeftButton) {
                root.ctrl.strokeBegin(p.x, p.y)
            } else if (root.ctrl.mode === 1) {
                root.ctrl.pointerMove(p.x, p.y)
                if (mouse.button === Qt.LeftButton) {
                    pressX = p.x
                    pressY = p.y
                    laserDragged = false
                }
            }
        }

        onReleased: function (mouse) {
            if (root.ctrl.mode === 2) {
                root.ctrl.strokeEnd()
                return
            }
            if (root.ctrl.mode === 1) {
                if (mouse.button === Qt.LeftButton) {
                    if (laserDragged)
                        root.ctrl.laserTrailEnd()
                    else
                        root.ctrl.mouseClick(Qt.LeftButton)
                    laserDragged = false
                } else if (mouse.button === Qt.RightButton) {
                    root.ctrl.mouseClick(Qt.RightButton)
                }
            }
        }

        onExited: {
            if (root.ctrl.mode === 1)
                root.ctrl.pointerHide()
        }
    }
}
