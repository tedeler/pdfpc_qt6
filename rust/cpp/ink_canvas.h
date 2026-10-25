#pragma once

#include <QColor>
#include <QJsonArray>
#include <QJsonDocument>
#include <QJsonObject>
#include <QPainter>
#include <QPen>
#include <QtQuick/QQuickPaintedItem>
#include <QtQml/qqmlregistration.h>

// Overlay that paints pen strokes and an optional laser pointer in page
// coordinates (0..1), matching the letterboxed slide image beneath it.
class InkCanvas : public QQuickPaintedItem
{
    Q_OBJECT
    QML_ELEMENT
    Q_PROPERTY(QString strokesJson READ strokesJson WRITE setStrokesJson NOTIFY strokesJsonChanged)
    Q_PROPERTY(bool pointerVisible READ pointerVisible WRITE setPointerVisible NOTIFY pointerVisibleChanged)
    Q_PROPERTY(qreal pointerX READ pointerX WRITE setPointerX NOTIFY pointerXChanged)
    Q_PROPERTY(qreal pointerY READ pointerY WRITE setPointerY NOTIFY pointerYChanged)

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

        if (m_pointerVisible && m_pointerX >= 0.0 && m_pointerY >= 0.0
            && m_pointerX <= 1.0 && m_pointerY <= 1.0) {
            painter->setPen(Qt::NoPen);
            painter->setBrush(QColor(220, 30, 30));
            const qreal r = 8.0;
            painter->drawEllipse(QPointF(m_pointerX * w, m_pointerY * h), r, r);
        }
    }

signals:
    void strokesJsonChanged();
    void pointerVisibleChanged();
    void pointerXChanged();
    void pointerYChanged();

private:
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

    QString m_strokesJson;
    QVector<QVector<QPointF>> m_strokes;
    bool m_pointerVisible = false;
    qreal m_pointerX = -1.0;
    qreal m_pointerY = -1.0;
};
