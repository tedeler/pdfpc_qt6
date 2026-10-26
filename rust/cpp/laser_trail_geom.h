#pragma once

#include <cmath>

#include <QColor>
#include <QImage>
#include <QPainter>
#include <QPainterPath>
#include <QPen>
#include <QPointF>
#include <QVector>
#include <QtGlobal>

// Geometry + painting for a continuous laser afterglow stroke:
// one soft red band with a thin white core (no per-segment RoundCap beads).

struct LaserTrailPoint {
    qreal x = 0.0; // page coords 0..1
    qreal y = 0.0;
    qreal a = 1.0; // opacity 0..1
};

inline QVector<QPointF> densifyPolyline(const QVector<QPointF> &pts, qreal maxStepPx)
{
    QVector<QPointF> out;
    if (pts.isEmpty())
        return out;
    maxStepPx = qMax<qreal>(0.25, maxStepPx);
    out.reserve(pts.size() * 2);
    out.append(pts.first());
    for (int i = 1; i < pts.size(); ++i) {
        const QPointF a = pts[i - 1];
        const QPointF b = pts[i];
        const QPointF d = b - a;
        const qreal dist = std::hypot(d.x(), d.y());
        if (dist <= maxStepPx) {
            out.append(b);
            continue;
        }
        const int steps = qMax(1, int(std::ceil(dist / maxStepPx)));
        for (int s = 1; s <= steps; ++s) {
            const qreal t = qreal(s) / qreal(steps);
            out.append(a + d * t);
        }
    }
    return out;
}

inline QPainterPath laserStrokePath(const QVector<QPointF> &pixelPts)
{
    QPainterPath path;
    if (pixelPts.size() < 2)
        return path;
    path.moveTo(pixelPts.first());
    for (int i = 1; i < pixelPts.size(); ++i)
        path.lineTo(pixelPts[i]);
    return path;
}

inline qreal averageAlpha(const QVector<LaserTrailPoint> &stroke)
{
    if (stroke.isEmpty())
        return 0.0;
    qreal sum = 0.0;
    for (const auto &p : stroke)
        sum += p.a;
    return sum / stroke.size();
}

// Paint one trail stroke as continuous layered pens (red glow + white core).
inline void paintContinuousLaserStroke(QPainter *painter, const QVector<LaserTrailPoint> &stroke,
                                       qreal pageW, qreal pageH, qreal pointerSize)
{
    if (!painter || stroke.size() < 2 || pageW <= 0 || pageH <= 0)
        return;

    QVector<QPointF> raw;
    raw.reserve(stroke.size());
    for (const auto &p : stroke)
        raw.append(QPointF(p.x * pageW, p.y * pageH));

    const qreal base = qMax<qreal>(1.2, pointerSize * pageH);
    const qreal glowOuter = base * 5.5;
    const qreal glowMid = base * 3.2;
    const qreal glowInner = base * 1.8;
    const qreal coreWidth = qMax<qreal>(1.6, base * 0.7);
    // Step well below the white core so RoundJoin joins cannot open gaps.
    const QVector<QPointF> dense = densifyPolyline(raw, qMax<qreal>(0.75, coreWidth * 0.35));
    const QPainterPath path = laserStrokePath(dense);
    if (path.isEmpty())
        return;

    const qreal alpha = qBound(0.0, averageAlpha(stroke), 1.0);
    if (alpha <= 0.01)
        return;

    painter->setBrush(Qt::NoBrush);
    // One path with RoundJoin: continuous band. RoundCap only affects the two
    // endpoints (not each segment), so there are no bead artifacts.
    const auto strokePath = [&](const QColor &color, qreal width) {
        QPen pen(color, width, Qt::SolidLine, Qt::RoundCap, Qt::RoundJoin);
        painter->setPen(pen);
        painter->drawPath(path);
    };

    strokePath(QColor(255, 0, 0, int(55 * alpha)), glowOuter);
    strokePath(QColor(255, 20, 20, int(110 * alpha)), glowMid);
    strokePath(QColor(255, 60, 60, int(180 * alpha)), glowInner);
    strokePath(QColor(255, 255, 255, int(245 * alpha)), coreWidth);
}

// Render a horizontal stroke into an image for continuity checks in tests.
inline QImage renderLaserTrailSample(int width, int height, qreal pointerSize)
{
    QImage img(width, height, QImage::Format_ARGB32_Premultiplied);
    img.fill(Qt::transparent);
    QPainter painter(&img);
    painter.setRenderHint(QPainter::Antialiasing, true);

    QVector<LaserTrailPoint> stroke;
    // Sparse samples on purpose — paint must densify into a continuous band.
    // Keep y fixed so a center scanline is a fair continuity probe.
    stroke.append({0.05, 0.5, 1.0});
    stroke.append({0.35, 0.5, 1.0});
    stroke.append({0.65, 0.5, 1.0});
    stroke.append({0.95, 0.5, 1.0});
    paintContinuousLaserStroke(&painter, stroke, width, height, pointerSize);
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

// Returns true when the middle scanline has a continuous white-ish core and
// surrounding red band without long transparent gaps along the stroke span.
inline bool laserTrailImageIsContinuous(const QImage &img, int *whiteGapsOut = nullptr,
                                        int *redGapsOut = nullptr)
{
    if (img.isNull() || img.width() < 32 || img.height() < 8)
        return false;

    const int y = img.height() / 2;
    int first = -1;
    int last = -1;
    for (int x = 0; x < img.width(); ++x) {
        int a = 0, r = 0, g = 0, b = 0;
        sampleColumn(img, x, y, 6, &a, &r, &g, &b);
        if (a > 20) {
            if (first < 0)
                first = x;
            last = x;
        }
    }
    if (first < 0 || last - first < img.width() / 3)
        return false;

    int whiteGaps = 0;
    int redGaps = 0;
    int whiteRunGap = 0;
    int redRunGap = 0;
    const int gapTol = 4; // allow tiny AA holes

    for (int x = first; x <= last; ++x) {
        int maxA = 0, maxR = 0, maxG = 0, maxB = 0;
        sampleColumn(img, x, y, 6, &maxA, &maxR, &maxG, &maxB);

        const bool hasRed = maxA > 20 && maxR > 60 && maxR >= maxG && maxR >= maxB;
        // Bright core (white or hot pink-white) near the centerline.
        const bool hasWhite = maxA > 30 && maxG > 100 && maxB > 100 && maxR > 100;

        if (hasWhite)
            whiteRunGap = 0;
        else {
            ++whiteRunGap;
            if (whiteRunGap > gapTol)
                ++whiteGaps;
        }

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

    // Continuous band: no sustained gaps in white core or red glow along the span.
    return whiteGaps == 0 && redGaps == 0;
}
