#pragma once

#include <QColor>
#include <QJsonArray>
#include <QJsonDocument>
#include <QJsonObject>
#include <QPainter>
#include <QPen>
#include <QRadialGradient>
#include <QtQuick/QQuickPaintedItem>
#include <QtQml/qqmlregistration.h>

// Overlay that paints pen strokes, temporary laser afterglow trails, and an
// optional laser pointer in page coordinates (0..1), matching the letterboxed
// slide image beneath it.
class InkCanvas : public QQuickPaintedItem
{
    Q_OBJECT
    QML_ELEMENT
    Q_PROPERTY(QString strokesJson READ strokesJson WRITE setStrokesJson NOTIFY strokesJsonChanged)
    Q_PROPERTY(QString laserTrailsJson READ laserTrailsJson WRITE setLaserTrailsJson NOTIFY laserTrailsJsonChanged)
    Q_PROPERTY(bool pointerVisible READ pointerVisible WRITE setPointerVisible NOTIFY pointerVisibleChanged)
    Q_PROPERTY(qreal pointerX READ pointerX WRITE setPointerX NOTIFY pointerXChanged)
    Q_PROPERTY(qreal pointerY READ pointerY WRITE setPointerY NOTIFY pointerYChanged)
    // Pointer radius as a fraction of the page height, so presenter and
    // audience show the same relative size.
    Q_PROPERTY(qreal pointerSize READ pointerSize WRITE setPointerSize NOTIFY pointerSizeChanged)

public:
    explicit InkCanvas(QQuickItem *parent = nullptr)
        : QQuickPaintedItem(parent)
    {
        setAntialiasing(true);
        setOpaquePainting(false);
    }

    QString strokesJson() const { return m_strokesJson; }
    void setStrokesJson(const QString &json)
    {
        if (m_strokesJson == json)
            return;
        m_strokesJson = json;
        parseStrokes();
        emit strokesJsonChanged();
        update();
    }

    QString laserTrailsJson() const { return m_laserTrailsJson; }
    void setLaserTrailsJson(const QString &json)
    {
        if (m_laserTrailsJson == json)
            return;
        m_laserTrailsJson = json;
        parseLaserTrails();
        emit laserTrailsJsonChanged();
        update();
    }

    bool pointerVisible() const { return m_pointerVisible; }
    void setPointerVisible(bool visible)
    {
        if (m_pointerVisible == visible)
            return;
        m_pointerVisible = visible;
        emit pointerVisibleChanged();
        update();
    }

    qreal pointerX() const { return m_pointerX; }
    void setPointerX(qreal x)
    {
        if (qFuzzyCompare(m_pointerX, x))
            return;
        m_pointerX = x;
        emit pointerXChanged();
        update();
    }

    qreal pointerY() const { return m_pointerY; }
    void setPointerY(qreal y)
    {
        if (qFuzzyCompare(m_pointerY, y))
            return;
        m_pointerY = y;
        emit pointerYChanged();
        update();
    }

    qreal pointerSize() const { return m_pointerSize; }
    void setPointerSize(qreal size)
    {
        if (qFuzzyCompare(m_pointerSize, size))
            return;
        m_pointerSize = size;
        emit pointerSizeChanged();
        update();
    }

    void paint(QPainter *painter) override
    {
        const qreal w = width();
        const qreal h = height();
        if (w <= 0 || h <= 0)
            return;

        QPen pen(QColor(220, 30, 30), 3.0, Qt::SolidLine, Qt::RoundCap, Qt::RoundJoin);
        painter->setPen(pen);
        painter->setBrush(Qt::NoBrush);

        for (const auto &stroke : m_strokes) {
            if (stroke.size() < 2)
                continue;
            for (int i = 1; i < stroke.size(); ++i) {
                const QPointF a(stroke[i - 1].x() * w, stroke[i - 1].y() * h);
                const QPointF b(stroke[i].x() * w, stroke[i].y() * h);
                painter->drawLine(a, b);
            }
        }

        // Laser afterglow: white outline, red core; opacity from each point's age.
        const qreal coreWidth = qMax<qreal>(2.5, m_pointerSize * h * 0.55);
        const qreal outlineWidth = coreWidth + qMax<qreal>(2.0, coreWidth * 0.55);
        for (const auto &stroke : m_laserTrails) {
            if (stroke.size() < 2)
                continue;
            for (int i = 1; i < stroke.size(); ++i) {
                const TimedPoint &pa = stroke[i - 1];
                const TimedPoint &pb = stroke[i];
                const qreal alpha = qBound(0.0, (pa.a + pb.a) * 0.5, 1.0);
                if (alpha <= 0.01)
                    continue;
                const QPointF a(pa.x * w, pa.y * h);
                const QPointF b(pb.x * w, pb.y * h);

                QPen outline(QColor(255, 255, 255, int(230 * alpha)), outlineWidth,
                             Qt::SolidLine, Qt::RoundCap, Qt::RoundJoin);
                painter->setPen(outline);
                painter->drawLine(a, b);

                QPen core(QColor(220, 30, 30, int(255 * alpha)), coreWidth,
                          Qt::SolidLine, Qt::RoundCap, Qt::RoundJoin);
                painter->setPen(core);
                painter->drawLine(a, b);
            }
        }

        if (m_pointerVisible && m_pointerX >= 0.0 && m_pointerY >= 0.0
            && m_pointerX <= 1.0 && m_pointerY <= 1.0) {
            const QPointF c(m_pointerX * w, m_pointerY * h);
            const qreal r = qMax<qreal>(3.0, m_pointerSize * h);
            const qreal glow = r * 2.2;
            QRadialGradient gradient(c, glow);
            gradient.setColorAt(0.0, QColor(255, 40, 40, 235));
            gradient.setColorAt(r / glow * 0.75, QColor(255, 30, 30, 200));
            gradient.setColorAt(r / glow, QColor(255, 0, 0, 110));
            gradient.setColorAt(1.0, QColor(255, 0, 0, 0));
            painter->setPen(Qt::NoPen);
            painter->setBrush(gradient);
            painter->drawEllipse(c, glow, glow);
        }
    }

signals:
    void strokesJsonChanged();
    void laserTrailsJsonChanged();
    void pointerVisibleChanged();
    void pointerXChanged();
    void pointerYChanged();
    void pointerSizeChanged();

private:
    struct TimedPoint {
        qreal x = 0.0;
        qreal y = 0.0;
        qreal a = 1.0;
    };

    void parseStrokes()
    {
        m_strokes.clear();
        const QJsonDocument doc = QJsonDocument::fromJson(m_strokesJson.toUtf8());
        if (!doc.isArray())
            return;
        const QJsonArray pages = doc.array();
        for (const QJsonValue &strokeVal : pages) {
            if (!strokeVal.isArray())
                continue;
            QVector<QPointF> stroke;
            for (const QJsonValue &ptVal : strokeVal.toArray()) {
                const QJsonObject obj = ptVal.toObject();
                stroke.append(QPointF(obj.value(QStringLiteral("x")).toDouble(),
                                      obj.value(QStringLiteral("y")).toDouble()));
            }
            if (!stroke.isEmpty())
                m_strokes.append(stroke);
        }
    }

    void parseLaserTrails()
    {
        m_laserTrails.clear();
        const QJsonDocument doc = QJsonDocument::fromJson(m_laserTrailsJson.toUtf8());
        if (!doc.isArray())
            return;
        for (const QJsonValue &strokeVal : doc.array()) {
            if (!strokeVal.isArray())
                continue;
            QVector<TimedPoint> stroke;
            for (const QJsonValue &ptVal : strokeVal.toArray()) {
                const QJsonObject obj = ptVal.toObject();
                TimedPoint pt;
                pt.x = obj.value(QStringLiteral("x")).toDouble();
                pt.y = obj.value(QStringLiteral("y")).toDouble();
                pt.a = obj.value(QStringLiteral("a")).toDouble(1.0);
                stroke.append(pt);
            }
            if (stroke.size() >= 2)
                m_laserTrails.append(stroke);
        }
    }

    QString m_strokesJson;
    QString m_laserTrailsJson;
    QVector<QVector<QPointF>> m_strokes;
    QVector<QVector<TimedPoint>> m_laserTrails;
    bool m_pointerVisible = false;
    qreal m_pointerX = -1.0;
    qreal m_pointerY = -1.0;
    qreal m_pointerSize = 0.022;
};
