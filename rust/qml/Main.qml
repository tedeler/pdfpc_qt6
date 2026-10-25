import QtQuick
import QtQuick.Window
import QtQuick.Pdf
import org.pdfpc.app

Item {
    id: root

    Session {
        id: session
    }

    // Keeps pageCount in sync; slide views load the PDF themselves.
    PdfDocument {
        id: document
        source: session.pdf_url

        onStatusChanged: function () {
            if (document.status === PdfDocument.Ready)
                session.applyPageCount(document.pageCount)
        }
        onPageCountChanged: function () {
            if (document.status === PdfDocument.Ready)
                session.applyPageCount(document.pageCount)
        }
    }

    function screenAt(index) {
        var screens = Qt.application.screens
        if (!screens || screens.length === 0)
            return null
        if (index < 0 || index >= screens.length)
            return screens[0]
        return screens[index]
    }

    Presenter {
        id: presenterWindow
        ctrl: session
        windowed: session.windowed
        screen: root.screenAt(session.presenter_screen)
    }

    Audience {
        id: audienceWindow
        ctrl: session
        windowed: session.windowed
        screen: root.screenAt(session.audience_screen)
    }

    Timer {
        interval: 250
        running: true
        repeat: true
        onTriggered: session.tick()
    }

    Component.onCompleted: {
        var args = Qt.application.arguments
        var pdfPath = ""
        var windowed = false
        for (var i = 1; i < args.length; ++i) {
            if (args[i] === "--windowed" || args[i] === "-w")
                windowed = true
            else if (args[i] === "--help" || args[i] === "-h") {
                console.log("Usage: pdfpc [--windowed] <file.pdf>")
                Qt.quit()
                return
            } else if (!args[i].startsWith("-") && pdfPath === "") {
                pdfPath = args[i]
            }
        }
        session.configureWindowed(windowed)
        if (pdfPath === "") {
            console.log("Usage: pdfpc [--windowed] <file.pdf>")
            Qt.quit()
            return
        }
        session.loadPdf(pdfPath)

        var screens = Qt.application.screens
        if (!screens || screens.length < 2) {
            session.configureWindowed(true)
            session.presenter_screen = 0
            session.audience_screen = 0
        }
    }
}
