// Continuity self-test for laser afterglow painting.
// Build: g++ -std=c++17 test_laser_trail_geom.cpp $(pkg-config --cflags --libs Qt6Gui) -o test_laser_trail_geom
#include "laser_trail_geom.h"

#include <cstdio>

int main()
{
    // Match the small default pointer size (0.022/3).
    const qreal pointerSize = 0.022 / 3.0;
    const QImage img = renderLaserTrailSample(800, 240, pointerSize);

    int whiteGaps = -1;
    int redGaps = -1;
    const bool ok = laserTrailImageIsContinuous(img, &whiteGaps, &redGaps);
    if (!ok) {
        std::fprintf(stderr,
                     "laser trail continuity failed: whiteGaps=%d redGaps=%d\n", whiteGaps,
                     redGaps);
        img.save(QStringLiteral("laser_trail_fail.png"));
        return 1;
    }

    // Densify unit check: sparse pixel polyline becomes fine-grained.
    QVector<QPointF> sparse;
    sparse << QPointF(10, 100) << QPointF(400, 110) << QPointF(790, 100);
    const QVector<QPointF> dense = densifyPolyline(sparse, 2.0);
    if (dense.size() <= sparse.size()) {
        std::fprintf(stderr, "densifyPolyline did not insert points\n");
        return 2;
    }
    for (int i = 1; i < dense.size(); ++i) {
        const QPointF d = dense[i] - dense[i - 1];
        if (std::hypot(d.x(), d.y()) > 2.01) {
            std::fprintf(stderr, "densifyPolyline left a gap > 2px\n");
            return 3;
        }
    }

    // Regression: old per-segment RoundCap style on sparse samples is NOT continuous.
    {
        QImage bad(800, 240, QImage::Format_ARGB32_Premultiplied);
        bad.fill(Qt::transparent);
        QPainter p(&bad);
        p.setRenderHint(QPainter::Antialiasing, true);
        QVector<QPointF> pts;
        pts << QPointF(40, 120) << QPointF(280, 125) << QPointF(520, 115) << QPointF(760, 120);
        const qreal core = 2.0;
        const qreal glow = core * 5.5;
        for (int i = 1; i < pts.size(); ++i) {
            p.setPen(QPen(QColor(255, 0, 0, 80), glow, Qt::SolidLine, Qt::RoundCap, Qt::RoundJoin));
            p.drawLine(pts[i - 1], pts[i]);
            p.setPen(QPen(QColor(255, 255, 255, 240), core, Qt::SolidLine, Qt::RoundCap, Qt::RoundJoin));
            p.drawLine(pts[i - 1], pts[i]);
        }
        p.end();
        int wg = 0;
        int rg = 0;
        const bool badOk = laserTrailImageIsContinuous(bad, &wg, &rg);
        if (badOk) {
            std::fprintf(stderr,
                         "expected sparse RoundCap segments to fail continuity, but they passed\n");
            return 4;
        }
    }

    std::printf("ok: continuous white core + red band (whiteGaps=%d redGaps=%d)\n", whiteGaps,
                redGaps);
    return 0;
}
