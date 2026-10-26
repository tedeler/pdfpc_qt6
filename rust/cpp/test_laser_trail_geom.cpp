// Continuity + style self-test for laser afterglow painting.
// Build via scripts/test-laser-trail-continuity.sh
#include "laser_trail_geom.h"

#include <cstdio>

int main()
{
    const qreal pointerSize = 0.022 / 3.0;
    const QImage img = renderLaserTrailSample(800, 240, pointerSize);

    int whiteGaps = -1;
    int redGaps = -1;
    if (!laserTrailImageIsContinuous(img, &whiteGaps, &redGaps)) {
        std::fprintf(stderr,
                     "laser trail continuity failed: whiteGaps=%d redGaps=%d\n", whiteGaps,
                     redGaps);
        img.save(QStringLiteral("laser_trail_fail.png"));
        return 1;
    }

    if (!laserTrailImageHasWhiteInterior(img)) {
        std::fprintf(stderr, "expected white interior along the polyline center\n");
        img.save(QStringLiteral("laser_trail_fail.png"));
        return 2;
    }

    if (!laserTrailImageHasRedBorder(img)) {
        std::fprintf(stderr, "expected red glowing border around the white interior\n");
        img.save(QStringLiteral("laser_trail_fail.png"));
        return 3;
    }

    // Regression: drawing only short RoundCap dashes (no connecting path) must
    // not look like a continuous white interior.
    {
        QImage bad(800, 240, QImage::Format_ARGB32_Premultiplied);
        bad.fill(Qt::transparent);
        QPainter p(&bad);
        p.setRenderHint(QPainter::Antialiasing, true);
        // Isolated dots — the old "bead" failure mode.
        const QPointF dots[] = {{80, 120}, {280, 120}, {480, 120}, {680, 120}};
        for (const QPointF &c : dots) {
            p.setPen(Qt::NoPen);
            p.setBrush(QColor(255, 0, 0, 120));
            p.drawEllipse(c, 14, 14);
            p.setBrush(QColor(255, 255, 255, 240));
            p.drawEllipse(c, 3, 3);
        }
        p.end();
        if (laserTrailImageIsContinuous(bad, nullptr, nullptr)
            || laserTrailImageHasWhiteInterior(bad)) {
            std::fprintf(stderr, "expected isolated beads to fail continuous white interior\n");
            return 4;
        }
    }

    std::printf("ok: polyline ribbon with white interior + red border "
                "(whiteGaps=%d redGaps=%d)\n",
                whiteGaps, redGaps);
    return 0;
}
