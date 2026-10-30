# Navigation und History in pdfpc_qt6

## Teil 1: Anwenderdoku

Dieses Verhalten beschreibt, wie die Präsentationsnavigation in pdfpc_qt6 funktioniert. Es ist so gedacht, dass normale Präsentationsarbeit flüssig bleibt und gleichzeitig wichtige Sprünge sauber zurückverfolgbar sind.

### 1.1 Grundidee

Bei einer Präsentation gibt es zwei unterschiedliche Arten von Bewegung:

- normales Vor- und Zurückgehen
- echte Sprünge zu einem wichtigen Ziel

Normales Blättern ist einfach der tägliche Präsentationsfluss: von Folie zu Folie, mit Pfeiltasten, mit der Tastatur oder mit einem Sprung von +/-10 Folien. Dadurch will man in der Regel nur schnell weiterkommen.

Echte Sprünge sind dagegen Punkte, an denen man bewusst eine andere Stelle in der Präsentation anspringt, zum Beispiel:

- per GoTo zu einer bestimmten Folie,
- über ein internes PDF-Linkziel,
- über eine Markierung auf dem Stack.

Diese Sprünge sind für die Navigation wichtig und müssen sauber im Verlauf bleiben.

### 1.2 Warum normales Blättern nicht wie ein Sprung behandelt werden soll

Wenn jede Folie, die man mit der Pfeiltaste anspringt, als echter Sprung behandelt würde, wäre die Navigation schnell unbrauchbar:

- der Verlauf würde mit jeder normalen Folie überladen,
- der Vorwärts- und Rückwärts-Pfad wäre kaum noch aussagekräftig,
- man würde wichtige Sprünge mit alltäglichem Blättern vermischen.

Deshalb behandelt die Anwendung normales Blättern als reinen Ablauf und nicht als markanten Sprungpunkt.

### 1.3 Was ist ein Stack-Eintrag?

Ein Stack-Eintrag ist ein bewusst markierter Rückkehrpunkt. Er entsteht nur bei wichtigen Aktionen, zum Beispiel:

- GoTo zu einer Seite,
- Klick auf einen PDF-Link,
- aktive Markierung einer Folie auf dem Stack.

Diese Einträge sind für die Präsentation besonders wichtig, weil sie signalisieren: "An dieser Stelle wurde bewusst eine neue Richtung gewählt. Wenn ich später zurück gehe, soll ich diese Stelle nicht einfach als normalen Schritt behandeln."

### 1.4 Was ist mit Back/Forward und normalen Schritten?

Back/Forward und normale Schritt-Navigationen sind Teil des Ablaufverlaufs. Sie ändern die aktuelle Position, aber sie erzeugen keinen neuen bedeutenden Stack-Punkt.

Das ist wichtig, weil Back/Forward die Reise durch die bereits gesehenen Seiten zeigen soll, nicht aber jedes einzelne Blättern als neue historische Markierung festhalten.

### 1.5 Was passiert im Alltag?

Ein typischer Ablauf sieht so aus:

1. Der Vortrag beginnt auf Folie 1.
2. Der Nutzer geht normal weiter: 2, 3, 4, 5.
3. Er klickt auf ein internes PDF-Link auf Folie 20.
4. Er geht mit Back zurück auf die vorherige Stelle.
5. Er möchte später wieder an die Stelle gelangen, an der er den Link aufgerufen hat.

Die App soll dabei so arbeiten:

- der normale Verlauf von Folie 2 bis 5 ist kein wichtiger Sprungpunkt,
- der Link auf Folie 20 ist ein echter Sprungpunkt,
- Back/Forward zeigt den Verlauf an, ohne den Ablauf mit irrelevanten Schritten zu überladen.

Das macht die Navigation verständlich und stabil.

### 1.6 Warum ist das für die Präsentationsarbeit gut?

Es schafft eine saubere Trennung zwischen:

- Alltag der Präsentation,
- bewussten Navigationen,
- Verlauf der zuletzt besuchten Punkte,
- wichtigen Sprungzielen.

Das bedeutet:

- das normale Vorwärtsblättern bleibt unaufdringlich,
- wichtige Ziele bleiben nachvollziehbar,
- der Verlauf bleibt konsistent,
- der Benutzer kann gezielt zwischen interessanten Punkten springen, ohne dass alle normalen Folienwechsel den Stack verunreinigen.

---

## Teil 2: Technische Doku

Die technische Umsetzung sitzt in den Dateien `rust/src/nav_history.rs` und `rust/src/session.rs`.

### 2.1 Die grundlegende Unterscheidung

Im Code gibt es ein Enum, das die Art der Navigation beschreibt:

```rust
enum LeaveKind {
    Stack,   // Go-to / PDF-Link / explizite Markierung
    Menu,    // Auswahl aus dem J-Menü
    Step,    // +/-1 / +/-10
    History, // Back/Forward-History
}
```

Danach entscheidet die App, ob die aktuelle Bewegung den Stack beeinflusst:

```rust
fn stack_action(kind: LeaveKind) -> StackAction {
    match kind {
        LeaveKind::Stack => StackAction::RecordAndTruncate,
        LeaveKind::Menu => StackAction::TruncateOnly,
        LeaveKind::Step | LeaveKind::History => StackAction::None,
    }
}
```

Das ist der Kern der Logik:

- `Stack` erzeugt einen echten Eintrag
- `Menu` verwirft den Forward-Tail, aber erzeugt keinen normalen Eintrag
- `Step` erzeugt keinen Stack-Eintrag
- `History` erzeugt keinen Stack-Eintrag

### 2.2 Normales Blättern: keine Stack-Markierung

Die normalen Seitenwechsel werden als `LeaveKind::Step` behandelt:

```rust
pub fn next_page_action(self: Pin<&mut Self>) {
    self.jump_pages(1);
}

pub fn prev_page_action(self: Pin<&mut Self>) {
    self.jump_pages(-1);
}

fn jump_pages(self: Pin<&mut Self>, delta: i32) {
    let Some(target) = page_after_jump(cur, count, delta) else {
        return;
    };
    self.go_to_page(target, LeaveKind::Step);
}
```

Damit gilt für `+/-1` und `+/-10`:

- normaler Seitenwechsel
- kein Stack-Eintrag
- kein wichtiges Sprungziel

Die gleiche Logik gilt für die Tastensteuerung, etwa Pfeil rechts, links, Space, PageUp und PageDown.

### 2.3 History-Back und History-Forward: Verlauf, aber kein Stack-Eintrag

Hier ist das Verhalten ebenfalls bewusst getrennt:

```rust
pub fn history_back(...) {
    let target = self.as_mut().rust_mut().nav_history.go_back(cur, Instant::now());
    if let Some(target) = target {
        self.apply_history_page(target);
    }
}

pub fn history_forward(...) {
    let target = self.as_mut().rust_mut().nav_history.go_forward(cur, Instant::now());
    if let Some(target) = target {
        self.apply_history_page(target);
    }
}

fn apply_history_page(self: Pin<&mut Self>, target: i32) {
    self.go_to_page(target, LeaveKind::History);
}
```

Machen wir die Semantik explizit:

- Back/Forward verändert den Verlauf
- Back/Forward erzeugt keinen neuen Stack-Punkt
- der Verlauf bleibt konsistent, aber er ist kein defaultmäßiger Jump-Stack

### 2.4 Echte Sprünge: GoTo, PDF-Link, Stack-Markierung

Ein echter Sprung wird als `LeaveKind::Stack` behandelt:

```rust
fn stack_jump(self: Pin<&mut Self>, target: i32) {
    self.go_to_page(target, LeaveKind::Stack);
}
```

Das passiert bei:

- `g` (GoTo)
- Klick auf eine interne PDF-Linkseite
- explizitem Markieren mit `m`

In diesem Fall gilt:

```rust
LeaveKind::Stack => StackAction::RecordAndTruncate
```

Das bedeutet:

- die aktuelle Seite wird als wirklicher Sprungpunkt notiert,
- der Forward-Tail wird verworfen,
- der Verlauf bleibt sinnvoll und auf bedeutende Punkte fokussiert.

### 2.5 J-Menü: verwirft nur den Forward-Tail

Beim J-Menü wird `LeaveKind::Menu` verwendet:

```rust
fn menu_jump(self: Pin<&mut Self>, target: i32) {
    self.go_to_page(target, LeaveKind::Menu);
}
```

Damit gilt:

```rust
LeaveKind::Menu => StackAction::TruncateOnly
```

Das heißt:

- der bisherige Future-/Forward-Pfad wird verworfen,
- aber es wird kein normaler Stack-Eintrag erzeugt,
- die Auswahl kommt aus dem bestehenden Verlauf und ist keine neue markante Ziel-Position.

### 2.6 Warum diese Trennung wichtig ist

Der besondere Punkt ist die Trennung zwischen:

- "normal dahinblättern",
- "bewusst springen",
- "im Verlauf zurückgehen",
- "aus einer Auswahl einen bestimmten Zielpunkt wählen".

Diese Trennung ist in der Anwendung nicht zufällig, sondern eine bewusste Modellierung der Präsentationsarbeit:

- normale Darstellung und Bewegung sollen die Stack-Logik nicht verunreinigen,
- echte Zielpunkte sollen sich sauber als Sprungpunkte verhalten,
- der Rück-/Vorwärts-Verlauf bleibt trotzdem nachvollziehbar.

---

## 3. Übersicht des Verhaltens

```text
Normale Schritt-Navigation:
  10 -> 11 -> 12 -> 13
  => kein Stack-Eintrag

PDF-Link oder GoTo:
  12 -> 27
  => neuer Stack-Eintrag
  => forward-tail wird verworfen

History-Back:
  27 -> 12
  => kein neuer Stack-Eintrag
  => nur Verlauf verschiebt sich

History-Forward:
  12 -> 27
  => kein neuer Stack-Eintrag
  => Verlauf wiederherstellen
```

---

## 4. Zusammenfassung

Die Architektur trennt sauber zwischen:

- normalem Folienfluss,
- meaningful jumps,
- Verlaufserinnerung,
- bewusst gewählten Zielen.

Das ist genau das Verhalten, das für eine Präsentationssoftware sinnvoll ist: Der Nutzer soll nicht mit jeder normalen Folienbewegung den Stack verwirren, aber wichtige Sprünge sollen nach wie vor sauber zurückverfolgbar bleiben.

Die Kernstellen im Code:

- `rust/src/nav_history.rs`
  - `LeaveKind`
  - `stack_action()`
  - `go_back()`
  - `go_forward()`
  - `begin_new_jump()`

- `rust/src/session.rs`
  - `jump_pages()`
  - `follow_link()`
  - `history_back()`
  - `history_forward()`
  - `go_to_page()`

Diese Stellen zeigen die genaue Umsetzung des beschriebenen Verhaltens.
