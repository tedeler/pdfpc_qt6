#pragma once

#include <cmath>

#include <QColor>
#include <QImage>
#include <QPainter>
#include <QPainterPath>
#include <QPainterPathStroker>
#include <QPen>
#include <QPointF>
#include <QVector>
#include <QtGlobal>

// One polyline → one laser ribbon: white interior, soft red glowing border.
// Drawn in a single pass along the full path (no per-segment RoundCap beads).

struct LaserTrailStroke {
    QVector<QPointF> pts; // page coords 0..1
    qreal a = 1.0;        // whole-stroke opacity
};

inline QPainterPath polylinePath(const QVector<QPointF> &pixelPts)
{
    QPainterPath path;
    if (pixelPts.size() < 2)
        return path;
    path.moveTo(pixelPts.first());
    for (int i = 1; i < pixelPts.size(); ++i)
        path.lineTo(pixelPts[i]);
    return path;
}

inline QPainterPath strokeRibbon(const QPainterPath &center, qreal widthPx)
{
    QPainterPathStroker stroker;
    stroker.setWidth(qMax<qreal>(1.0, widthPx));
    stroker.setCapStyle(Qt::RoundCap);
    stroker.setJoinStyle(Qt::RoundJoin);
    stroker.setCurveThreshold(0.5);
    return stroker.createStroke(center);
}

// Paint the laser effect in one step along the polyline:
// soft red glow band (border) + solid white interior.
inline void paintLaserPolyline(QPainter *painter, const LaserTrailStroke &stroke, qreal pageW,
                               qreal pageH, qreal pointerSize)
{
    if (!painter || stroke.pts.size() < 2 || pageW <= 0 || pageH <= 0)
        return;

    const qreal alpha = qBound(0.0, stroke.a, 1.0);
    if (alpha <= 0.01)
        return;

    QVector<QPointF> pixelPts;
    pixelPts.reserve(stroke.pts.size());
    for (const QPointF &p : stroke.pts)
        pixelPts.append(QPointF(p.x() * pageW, p.y() * pageH));

    const QPainterPath center = polylinePath(pixelPts);
    if (center.isEmpty())
        return;

    const qreal base = qMax<qreal>(2.0, pointerSize * pageH);
    const qreal glowWidth = base * 5.0;                  // red glowing border band
    const qreal coreWidth = qMax<qreal>(2.8, base * 1.35); // white interior

    const QPainterPath redRibbon = strokeRibbon(center, glowWidth);
    const QPainterPath whiteCore = strokeRibbon(center, coreWidth);

    painter->setPen(Qt::NoPen);

    // Soft outer glow (same geometry, more transparent).
    painter->setBrush(QColor(255, 0, 0, int(70 * alpha)));
    painter->drawPath(strokeRibbon(center, glowWidth * 1.55));

    // Main red-glowing border / body.
    painter->setBrush(QColor(255, 30, 30, int(160 * alpha)));
    painter->drawPath(redRibbon);

    // White interior along the same polyline.
    painter->setBrush(QColor(255, 255, 255, int(250 * alpha)));
    painter->drawPath(whiteCore);
}

inline QImage renderLaserTrailSample(int width, int height, qreal pointerSize)
{
    QImage img(width, height, QImage::Format_ARGB32_Premultiplied);
    img.fill(Qt::transparent);
    QPainter painter(&img);
    painter.setRenderHint(QPainter::Antialiasing, true);

    LaserTrailStroke stroke;
    stroke.a = 1.0;
    // Sparse samples — one polyline painted as a continuous ribbon.
    stroke.pts << QPointF(0.05, 0.5) << QPointF(0.35, 0.5) << QPointF(0.65, 0.5)
               << QPointF(0.95, 0.5);
    paintLaserPolyline(&painter, stroke, width, height, pointerSize);
    painter.end();
    return img;
}

inline void sampleColumn(const QImage &img, int x, int yCenter, int halfWindow, int *maxA,
                         int *maxR, int *maxG, int *maxB)
{
    *maxA = *maxR = *maxG = *maxB = 0;
    for (int dy = -halfWindow; dy <= halfWindow; ++dy) {
        const int yy = qBound(0, yCenter + dy, img.height() - 1);
        const QColor c = QColor::fromRgba(img.pixel(x, yy));
        if (c.alpha() > *maxA) {
            *maxA = c.alpha();
            *maxR = c.red();
            *maxG = c.green();
            *maxB = c.blue();
        }
    }
}

inline bool laserTrailImageIsContinuous(const QImage &img, int *whiteGapsOut = nullptr,
                                        int *redGapsOut = nullptr)
{
    if (img.isNull() || img.width() < 32 || img.height() < 8)
        return false;

    const int y = img.height() / 2;
    int inkFirst = -1;
    int inkLast = -1;
    int whiteFirst = -1;
    int whiteLast = -1;
    for (int x = 0; x < img.width(); ++x) {
        int a = 0, r = 0, g = 0, b = 0;
        sampleColumn(img, x, y, 8, &a, &r, &g, &b);
        const bool hasInk = a > 20;
        const bool hasWhite = a > 30 && g > 100 && b > 100 && r > 100;
        if (hasInk) {
            if (inkFirst < 0)
                inkFirst = x;
            inkLast = x;
        }
        if (hasWhite) {
            if (whiteFirst < 0)
                whiteFirst = x;
            whiteLast = x;
        }
    }
    if (inkFirst < 0 || inkLast - inkFirst < img.width() / 3)
        return false;
    if (whiteFirst < 0 || whiteLast - whiteFirst < img.width() / 4)
        return false;

    int whiteGaps = 0;
    int redGaps = 0;
    int whiteRunGap = 0;
    int redRunGap = 0;
    const int gapTol = 6;

    // White continuity only inside the white-core span.
    for (int x = whiteFirst; x <= whiteLast; ++x) {
        int maxA = 0, maxR = 0, maxG = 0, maxB = 0;
        sampleColumn(img, x, y, 8, &maxA, &maxR, &maxG, &maxB);
        const bool hasWhite = maxA > 30 && maxG > 100 && maxB > 100 && maxR > 100;
        if (hasWhite)
            whiteRunGap = 0;
        else {
            ++whiteRunGap;
            if (whiteRunGap > gapTol)
                ++whiteGaps;
        }
    }

    // Red (or white) ink continuity across the full ribbon span.
    for (int x = inkFirst; x <= inkLast; ++x) {
        int maxA = 0, maxR = 0, maxG = 0, maxB = 0;
        sampleColumn(img, x, y, 8, &maxA, &maxR, &maxG, &maxB);
        const bool hasRed = maxA > 20 && maxR > 60 && maxR >= maxG && maxR >= maxB;
        const bool hasWhite = maxA > 30 && maxG > 100 && maxB > 100 && maxR > 100;
        if (hasRed || hasWhite)
            redRunGap = 0;
        else {
            ++redRunGap;
            if (redRunGap > gapTol)
                ++redGaps;
        }
    }

    if (whiteGapsOut)
        *whiteGapsOut = whiteGaps;
    if (redGapsOut)
        *redGapsOut = redGaps;

    return whiteGaps == 0 && redGaps == 0;
}

// Center column: white interior must dominate over red at the midline.
inline bool laserTrailImageHasWhiteInterior(const QImage &img)
{
    if (img.isNull())
        return false;
    const int x = img.width() / 2;
    const int y = img.height() / 2;
    const QColor c = QColor::fromRgba(img.pixel(x, y));
    if (c.alpha() < 40)
        return false;
    // Near-white core (all channels high and similar).
    return c.red() > 180 && c.green() > 180 && c.blue() > 180
            && std::abs(c.red() - c.green()) < 40;
}

// Off-center on the band: still some red glow (border).
inline bool laserTrailImageHasRedBorder(const QImage &img)
{
    if (img.isNull())
        return false;
    const int x = img.width() / 2;
    const int y = img.height() / 2;
    // Sample above the white core into the red ribbon.
    for (int dy = 3; dy <= 14; ++dy) {
        const int yy = qBound(0, y - dy, img.height() - 1);
        const QColor c = QColor::fromRgba(img.pixel(x, yy));
        if (c.alpha() > 30 && c.red() > 100 && c.red() > c.green() + 30 && c.red() > c.blue() + 30)
            return true;
    }
    return false;
}
