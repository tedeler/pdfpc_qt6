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
            // Remount once Ready so currentFrame is applied at creation time
            // (PdfPageImage often ignores a pre-set frame before Ready).
            if (status === PdfDocument.Ready)
                root.hardResetRaster()
        }
        onPageCountChanged: function () {
            if (status === PdfDocument.Ready && pageCount > 0 && root.ctrl)
                root.ctrl.applyPageCount(pageCount)
        }
        onSourceChanged: root.hardResetRaster()
    }

    readonly property bool pageValid: root.page >= 0
            && localDocument.status === PdfDocument.Ready
            && localDocument.pageCount > 0
    readonly property int frameIndex: root.pageValid
            ? Math.max(0, Math.min(root.page, localDocument.pageCount - 1))
            : 0

    // Ping-pong raster slots: keep the front page visible while the back slot
    // remounts for the new frame — avoids the blank flash of a single Loader.
    property int frontSlot: 0
    property int loadSeq: 0

    onFrameIndexChanged: {
        if (root.pageValid)
            root.loadFrame(root.frameIndex, false)
    }

    onPageValidChanged: {
        if (root.pageValid)
            root.hardResetRaster()
        else
            root.teardownSlots()
    }

    function teardownSlots() {
        slot0.active = false
        slot1.active = false
    }

    function hardResetRaster() {
        root.loadSeq += 1
        root.teardownSlots()
        root.frontSlot = 0
        if (root.pageValid)
            root.loadIntoSlot(0, root.frameIndex, root.loadSeq, true)
    }

    function loadFrame(frame, forceFront) {
        if (!root.pageValid)
            return
        var front = root.frontItem()
        if (!forceFront && front
                && front.currentFrame === frame
                && front.status === Image.Ready
                && front.paintedWidth > 0) {
            return
        }
        // An in-flight load (e.g. hardReset) already targets this frame.
        if ((slot0.seq === root.loadSeq && slot0.wantedFrame === frame)
                || (slot1.seq === root.loadSeq && slot1.wantedFrame === frame)) {
            return
        }
        var back = 1 - root.frontSlot
        root.loadSeq += 1
        root.loadIntoSlot(back, frame, root.loadSeq, true)
    }

    function loadIntoSlot(slot, frame, seq, promoteWhenReady) {
        var loader = slot === 0 ? slot0 : slot1
        loader.wantedFrame = frame
        loader.seq = seq
        loader.promoteWhenReady = promoteWhenReady
        loader.active = false
        Qt.callLater(function () {
            if (loader.seq !== seq)
                return
            if (!root.pageValid)
                return
            loader.active = true
        })
    }

    function frontItem() {
        var loader = root.frontSlot === 0 ? slot0 : slot1
        return loader.item
    }

    function onSlotReady(slot) {
        var loader = slot === 0 ? slot0 : slot1
        if (!loader.item)
            return
        if (loader.seq !== root.loadSeq)
            return
        if (loader.item.currentFrame !== root.frameIndex)
            return
        if (loader.item.status !== Image.Ready || loader.item.paintedWidth <= 0)
            return
        if (loader.promoteWhenReady)
            root.frontSlot = slot
    }

    readonly property size pointSize: root.pageValid
                                      ? localDocument.pagePointSize(root.frameIndex)
                                      : Qt.size(4, 3)
    readonly property real fitScale: Math.min(width / Math.max(1, pointSize.width),
                                              height / Math.max(1, pointSize.height))

    readonly property var frontPage: root.frontItem()
    readonly property real pageX: frontPage
            ? frontPage.x + (frontPage.width - frontPage.paintedWidth) / 2
            : 0
    readonly property real pageY: frontPage
            ? frontPage.y + (frontPage.height - frontPage.paintedHeight) / 2
            : 0
    readonly property real pageW: frontPage ? Math.max(1, frontPage.paintedWidth) : 1
    readonly property real pageH: frontPage ? Math.max(1, frontPage.paintedHeight) : 1

    Rectangle {
        id: paper
        x: root.pageX
        y: root.pageY
        width: root.pageW
        height: root.pageH
        color: "#ffffff"
        visible: root.frontPage && root.frontPage.visible && root.frontPage.paintedWidth > 0
        z: 0
    }

    Loader {
        id: slot0
        property int wantedFrame: 0
        property int seq: 0
        property bool promoteWhenReady: false
        readonly property int slotIndex: 0

        anchors.fill: parent
        active: false
        z: root.frontSlot === 0 ? 2 : 1
        opacity: root.frontSlot === 0 ? 1 : 0
        sourceComponent: pageComponent

        onStatusChanged: {
            if (status === Loader.Ready)
                root.onSlotReady(0)
        }
    }

    Loader {
        id: slot1
        property int wantedFrame: 0
        property int seq: 0
        property bool promoteWhenReady: false
        readonly property int slotIndex: 1

        anchors.fill: parent
        active: false
        z: root.frontSlot === 1 ? 2 : 1
        opacity: root.frontSlot === 1 ? 1 : 0
        sourceComponent: pageComponent

        onStatusChanged: {
            if (status === Loader.Ready)
                root.onSlotReady(1)
        }
    }

    Component {
        id: pageComponent
        PdfPageImage {
            // parent is the Loader that created this item.
            anchors.fill: parent
            document: localDocument
            // Freeze the frame at remount time so the back buffer does not jump
            // when root.frameIndex advances again mid-decode.
            currentFrame: parent.wantedFrame
            fillMode: Image.PreserveAspectFit
            sourceSize.width: Math.max(1, Math.round(root.pointSize.width * root.fitScale * Screen.devicePixelRatio))
            sourceSize.height: Math.max(1, Math.round(root.pointSize.height * root.fitScale * Screen.devicePixelRatio))
            asynchronous: false
            visible: status === Image.Ready && paintedWidth > 0

            Component.onCompleted: {
                if ("retainWhileLoading" in this)
                    retainWhileLoading = true
                // Loader.Ready can fire before Image.Ready; re-check promote.
                root.onSlotReady(parent.slotIndex)
            }

            onStatusChanged: root.onSlotReady(parent.slotIndex)
            onPaintedWidthChanged: root.onSlotReady(parent.slotIndex)
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
        z: 3
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

        function laserStraight(modifiers) {
            return (modifiers & Qt.ShiftModifier) !== 0
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
                    var straight = laserStraight(mouse.modifiers)
                    if (!laserDragged
                            && pageDist(pressX, pressY, p.x, p.y) >= clickSlop) {
                        laserDragged = true
                        root.ctrl.laserTrailBegin(pressX, pressY, straight)
                        root.ctrl.laserTrailAdd(p.x, p.y, straight)
                    } else if (laserDragged) {
                        root.ctrl.laserTrailAdd(p.x, p.y, straight)
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
