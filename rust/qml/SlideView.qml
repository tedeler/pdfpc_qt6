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

        onStatusChanged: function () {
            if (status === PdfDocument.Ready && pageCount > 0 && root.ctrl)
                root.ctrl.applyPageCount(pageCount)
            root.syncFrame()
        }
        onPageCountChanged: function () {
            if (status === PdfDocument.Ready && pageCount > 0 && root.ctrl)
                root.ctrl.applyPageCount(pageCount)
            root.syncFrame()
        }
    }

    // Clamp to the loaded document so a restored page is shown once Ready.
    readonly property int frameIndex: {
        if (localDocument.status !== PdfDocument.Ready || localDocument.pageCount <= 0)
            return 0
        return Math.max(0, Math.min(root.page, localDocument.pageCount - 1))
    }

    function syncFrame() {
        if (localDocument.status !== PdfDocument.Ready || localDocument.pageCount <= 0)
            return
        frameNudgeTimer.restart()
    }

    Timer {
        id: frameNudgeTimer
        interval: 50
        repeat: false
        onTriggered: {
            if (localDocument.status !== PdfDocument.Ready || localDocument.pageCount <= 0)
                return
            var frame = root.frameIndex
            // Nudge currentFrame so Qt PDF re-rasterizes after Ready (otherwise a
            // restored page can stay stuck on the first decoded frame).
            pageImage.currentFrame = frame === 0 ? 1 : 0
            pageImage.currentFrame = frame
        }
    }

    onPageChanged: syncFrame()

    readonly property size pointSize: localDocument.status === PdfDocument.Ready
                                      ? localDocument.pagePointSize(root.frameIndex)
                                      : Qt.size(4, 3)
    readonly property real fitScale: Math.min(width / Math.max(1, pointSize.width),
                                              height / Math.max(1, pointSize.height))

    // Qt PDF leaves unpainted paper transparent (ARGB). PDFs that rely on the
    // implicit white page (datasheets, many Distiller exports) would otherwise
    // show the dark presenter/audience chrome through the slide. Match
    // QtQuick.Pdf.PdfPageView, which is itself a white Rectangle.
    readonly property real pageX: pageImage.x + (pageImage.width - pageImage.paintedWidth) / 2
    readonly property real pageY: pageImage.y + (pageImage.height - pageImage.paintedHeight) / 2
    readonly property real pageW: Math.max(1, pageImage.paintedWidth)
    readonly property real pageH: Math.max(1, pageImage.paintedHeight)

    Rectangle {
        id: paper
        x: root.pageX
        y: root.pageY
        width: root.pageW
        height: root.pageH
        color: "#ffffff"
        visible: pageImage.visible
        z: 0
    }

    PdfPageImage {
        id: pageImage
        anchors.fill: parent
        document: localDocument
        currentFrame: 0
        fillMode: Image.PreserveAspectFit
        // Without sourceSize Qt PDF rasterizes at the page size in points
        // (~360 px wide for Beamer), which is blurry when scaled to a screen.
        // Qt PDF renders exactly to sourceSize, so it must keep the page aspect.
        sourceSize.width: Math.max(1, Math.round(root.pointSize.width * root.fitScale * Screen.devicePixelRatio))
        sourceSize.height: Math.max(1, Math.round(root.pointSize.height * root.fitScale * Screen.devicePixelRatio))
        asynchronous: true
        visible: root.page >= 0 && localDocument.status === PdfDocument.Ready
        z: 1

        // retainWhileLoading is Qt 6.5+; set when the property exists.
        Component.onCompleted: {
            if ("retainWhileLoading" in pageImage)
                pageImage.retainWhileLoading = true
        }
    }

    InkCanvas {
        id: ink
        x: root.pageX
        y: root.pageY
        width: root.pageW
        height: root.pageH
        strokesJson: root.strokesJson
        pointerVisible: root.pointerVisible
        pointerX: root.pointerX
        pointerY: root.pointerY
        pointerSize: root.ctrl.pointer_size
        visible: root.page >= 0
        z: 2
    }

    MouseArea {
        anchors.fill: ink
        enabled: root.interactive
        acceptedButtons: Qt.LeftButton | Qt.RightButton
        hoverEnabled: root.interactive && root.ctrl.mode === 1
        // The laser dot replaces the mouse cursor over the slide.
        cursorShape: root.ctrl.mode === 1 ? Qt.BlankCursor : Qt.ArrowCursor

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
