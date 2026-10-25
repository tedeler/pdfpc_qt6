import QtQuick
import QtQuick.Window
import QtQuick.Pdf
import org.pdfpc.app

// Root is a plain QtObject: a Window declared inside an Item waits for that
// Item's window to become visible before it shows itself, and the root Item
// of a QQmlApplicationEngine never has one.
QtObject {
    id: root

    property Session session: Session {}

    // --windowed from the command line; otherwise windowed only while fewer
    // than two screens are connected.
    property bool userWindowed: false
    property bool started: false

    // Keeps pageCount in sync; slide views load the PDF themselves.
    property PdfDocument document: PdfDocument {
        source: root.session.pdf_url

        onStatusChanged: function () {
            if (status === PdfDocument.Ready)
                root.session.applyPageCount(pageCount)
        }
        onPageCountChanged: function () {
            if (status === PdfDocument.Ready)
                root.session.applyPageCount(pageCount)
        }
    }

    property Presenter presenterWindow: Presenter {
        ctrl: root.session
    }

    property Audience audienceWindow: Audience {
        ctrl: root.session
    }

    property Timer tickTimer: Timer {
        interval: 250
        running: true
        repeat: true
        onTriggered: root.session.tick()
    }

    property Timer placeTimer: Timer {
        interval: 50
        onTriggered: root.placeWindows()
    }

    // KWin assigns a fullscreen surface to the output of its current screen
    // only once the surface is mapped, so fullscreen is requested in a second
    // step after the windows were shown (or moved) as normal windows.
    property Timer fullScreenTimer: Timer {
        interval: 150
        onTriggered: root.enterFullScreen()
    }

    property Connections sessionConnections: Connections {
        target: root.session
        function onPresenter_screenChanged() { root.schedulePlacement() }
        function onAudience_screenChanged() { root.schedulePlacement() }
        function onWindowedChanged() { root.schedulePlacement() }
    }

    property Connections screenConnections: Connections {
        target: Qt.application
        function onScreensChanged() {
            if (root.started)
                root.updateScreenLayout()
        }
    }

    function screenCount() {
        var screens = Qt.application.screens
        return screens ? screens.length : 0
    }

    function screenAt(index) {
        var screens = Qt.application.screens
        if (!screens || screens.length === 0)
            return null
        if (index < 0 || index >= screens.length)
            return screens[0]
        return screens[index]
    }

    function updateScreenLayout() {
        var n = screenCount()
        var windowed = userWindowed || n < 2
        if (n >= 2) {
            var p = session.presenter_screen
            var a = session.audience_screen
            if (p < 0 || p >= n || a < 0 || a >= n || p === a) {
                session.presenter_screen = 0
                session.audience_screen = 1
            }
        } else {
            session.presenter_screen = 0
            session.audience_screen = 0
        }
        console.log("pdfpc: screens=" + n + " windowed=" + windowed
                    + " presenterScreen=" + session.presenter_screen
                    + " audienceScreen=" + session.audience_screen)
        session.configureWindowed(windowed)
        schedulePlacement()
    }

    function schedulePlacement() {
        if (started)
            placeTimer.restart()
    }

    function showWindowed(win, screen, rx, ry, rw, rh) {
        if (win.visibility === Window.FullScreen || win.visibility === Window.Maximized)
            win.visibility = Window.Windowed
        if (screen) {
            win.screen = screen
            win.x = screen.virtualX + Math.round(screen.width * rx)
            win.y = screen.virtualY + Math.round(screen.height * ry)
            win.width = Math.round(screen.width * rw)
            win.height = Math.round(screen.height * rh)
        }
        win.visibility = Window.Windowed
    }

    function placeWindows() {
        var ps = screenAt(session.presenter_screen)
        var aus = screenAt(session.audience_screen)
        console.log("pdfpc: placing presenter on " + (ps ? ps.name : "?")
                    + ", audience on " + (aus ? aus.name : "?")
                    + (session.windowed ? " (windowed)" : " (fullscreen)"))
        fullScreenTimer.stop()
        if (session.windowed) {
            showWindowed(presenterWindow, ps, 0.02, 0.04, 0.6, 0.7)
            showWindowed(audienceWindow, aus, 0.64, 0.04, 0.34, 0.34)
        } else {
            showWindowed(presenterWindow, ps, 0.1, 0.1, 0.8, 0.8)
            showWindowed(audienceWindow, aus, 0.1, 0.1, 0.8, 0.8)
            fullScreenTimer.start()
        }
        presenterWindow.raise()
        presenterWindow.requestActivate()
    }

    function enterFullScreen() {
        // KWin maps new normal windows on the active output and Qt then
        // reports that output as the window's screen; restore the target.
        var ps = screenAt(session.presenter_screen)
        var aus = screenAt(session.audience_screen)
        if (ps)
            presenterWindow.screen = ps
        if (aus)
            audienceWindow.screen = aus
        presenterWindow.visibility = Window.FullScreen
        audienceWindow.visibility = Window.FullScreen
        presenterWindow.raise()
        presenterWindow.requestActivate()
    }

    Component.onCompleted: {
        var args = Qt.application.arguments
        var pdfPath = ""
        for (var i = 1; i < args.length; ++i) {
            if (args[i] === "--windowed" || args[i] === "-w")
                userWindowed = true
            else if (args[i] === "--help" || args[i] === "-h") {
                console.log("Usage: pdfpc [--windowed] <file.pdf>")
                Qt.quit()
                return
            } else if (!args[i].startsWith("-") && pdfPath === "") {
                pdfPath = args[i]
            }
        }

        if (pdfPath === "") {
            console.log("Usage: pdfpc [--windowed] <file.pdf>")
            Qt.quit()
            return
        }
        session.loadPdf(pdfPath)
        started = true
        updateScreenLayout()
    }
}
