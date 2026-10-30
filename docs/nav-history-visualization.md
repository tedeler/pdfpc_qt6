# Navigation und History in pdfpc_qt6

Dieses Dokument erklärt den History-Mechanismus der Anwendung in verständlicher Sprache. Es beschreibt nicht nur das technische Muster, sondern auch, warum es für eine PDF-Präsentationssoftware genau so umgesetzt ist.

---

## 1. Warum dieses System überhaupt nötig ist

In einer Präsentationssoftware gibt es zwei sehr verschiedene Arten von Bewegung zwischen Folien:

1. Normales Vor- und Zurückgehen
   - z. B. nächste Folie, vorige Folie, 10 Slides vor/zurück
   - Das ist einfach "ich sehe weiter"

2. Bedeutende Sprünge
   - z. B. in ein PDF-Links-Ziel
   - z. B. per GoTo zu einer bestimmten Folie
   - z. B. markierte Folien auf dem Stack
   - Das sind "ich habe einen expliziten Zielpunkt gewählt"

Wenn man beide Mischformen in denselben Verlauf stecken würde, wäre die History-Logik schnell verwirrend. Ein normaler Folienwechsel darf nicht wie ein bewusster Sprung behandelt werden. Sonst würde die App beim normalen Blättern ständig den Rück-/Vorwärts-Pfad zerstören.

Das ist genau der Grund, warum die Anwendung in `LeaveKind` unterscheidet:

- normale Schritte
- history-walks
- echte Sprünge
- Menü-/Stack-Sprünge

---

## 2. Die zentrale Idee: nicht jede Folienbewegung erzeugt einen Stack-Eintrag

Die Anwendung unterscheidet drei Arten von Bewegung:

| Art | Beispiel | Effekt auf History-Stack |
|---|---|---|
| Normaler Schritt | nächste Folie, vorherige Folie, +/-10 | kein neuer Stack-Eintrag |
| History-Walk | Zurück/Weiter im Navigationspfad | kein neuer Stack-Eintrag |
| Echte Sprünge | GoTo, PDF-Link, Markierung | neuer Stack-Eintrag |
| Menü-Sprung | Auswahl aus dem J-Menü | forward-tail wird verworfen, aber kein normaler Stack-Eintrag |

Das ist ein sehr wichtiges Designprinzip: Nur wirklich bedeutende Sprünge sollten den Navigationspfad beeinflussen.

Wenn man nur vorwärts durch die Folien geht, ist das kein "Zielpunkt" im Sinne der Historie. Es ist einfach der normale Ablauf einer Präsentation.

---

## 3. Was die Anwendung in der Praxis bedeutet

Stellen Sie sich eine Präsentation vor:

- Sie sind auf Folie 12
- Sie öffnen ein PDF-TOC-Link auf Folie 27
- Sie gehen mit Pfeil zurück
- Sie wollen danach wieder zu Folie 27 bzw. zu den vorherigen sinnvollen Stellen gelangen

In einer guten Präsentations-UI sollte das so funktionieren:

- Der Sprung auf Folie 27 ist ein echter "jump"
- Der normale Rückweg durch die Folien ist nur informativ
- Der Vorwärts-Pfad darf aber nicht einfach verloren gehen

Das Verhalten ist deshalb ähnlich wie bei einem Browser:

- Ein normaler Seitenwechsel ist kein gespeicherter Sprungpunkt
- Ein Klick auf ein internes PDF-Ziel ist ein echter Sprungpunkt
- Ein Rück-/Vorwärtslauf innerhalb der Präsentation ist nur ein Verlauf des bisherigen Pfads

---

## 4. Die drei Hauptzustände der Logik

Die Anwendung verwaltet intern drei Arten von Informationen:

### 4.1 back
Das ist die Reihenfolge der Seiten, die du hinter dir gelassen hast.

```text
back = [0, 2, 5, 9]
```

Das bedeutet: Du hast dich von 0 über 2, 5 und 9 bewegt; diese Seiten sind im Verlauf hinter dir.

### 4.2 forward
Das ist der Teil des Pfads, den du noch "vor dir" hättest, wenn du nach hinten gegangen bist.

```text
forward = [11, 13]
```

Das bedeutet: Wenn du nach vorne gehst, kannst du wieder zu 11 und 13.

### 4.3 saved
Das sind markierte oder wichtige Stellen, die bewusst im Stack bleiben sollen.

```text
saved = {3, 10, 27}
```

Das sind keine zufälligen Folien, sondern Seiten, die als sinnvolle Sprungziele festgehalten werden.

### 4.4 last_visited
F��r jede Seite wird gespeichert, wann sie zuletzt besucht wurde.

Das wird im J-Menü verwendet, um die zuletzt besuchten Seiten oben anzuzeigen.

---

## 5. Die vier Navigationstypen im Code

Im Code werden die Navigationsarten als `LeaveKind` modelliert:

```rust
enum LeaveKind {
    Stack,   // Go-to / PDF-Link / Stack-Markierung
    Menu,    // J-Menü-Auswahl
    Step,    // +/-1 / +/-10
    History, // Back/Forward-History
}
```

Dann wird entschieden, ob dieser Vorgang den Stack beeinflusst:

```rust
fn stack_action(kind: LeaveKind) -> StackAction {
    match kind {
        LeaveKind::Stack => StackAction::RecordAndTruncate,
        LeaveKind::Menu => StackAction::TruncateOnly,
        LeaveKind::Step | LeaveKind::History => StackAction::None,
    }
}
```

Das ist der eigentliche Kern: 

- `Stack` => hier wird ein echter Eintrag angelegt
- `Menu` => der forward-tail wird verworfen, aber kein normaler Eintrag erzeugt
- `Step` => nur eine normale Folienbewegung, kein Stack-Eintrag
- `History` => ebenfalls kein Stack-Eintrag

---

## 6. Was genau bedeutet das für die Anwendung?

### 6.1 Normales Blättern = kein Stack-Eintrag

Wenn der Nutzer mit den Pfeiltasten durch die Präsentation geht, passiert das in `jump_pages()`:

```rust
self.go_to_page(target, LeaveKind::Step);
```

`LeaveKind::Step` ist in `stack_action()` ausdrücklich als `None` definiert. Das heißt:

- Folie 12 -> 13 -> 14
- keine Markierung im Stack
- keine neue History-Position als “bedeutender Punkt"

Das ist korrekt, weil das ganz normaler Präsentationsfluss ist.

Das gleiche gilt für `+/-10`-Sprünge:

```rust
KEY_RIGHT => Some(if shift { PAGE_SKIP } else { 1 }),
KEY_LEFT => Some(if shift { -PAGE_SKIP } else { -1 }),
```

`PAGE_SKIP` ist 10. Auch das ist nur ein normaler Schritt, kein wahrer jump.

### 6.2 History-Befehle = kein Stack-Eintrag

Back/Forward in der History sind ebenfalls keine Stack-Positionen. Sie verändern nur den Verlauf:

```rust
pub fn history_back(...) { ... }
pub fn history_forward(...) { ... }
```

Der Aufruf führt zu:

```rust
self.go_to_page(target, LeaveKind::History);
```

Und `History` ist ebenfalls `StackAction::None`.

Das ist logisch: History ist ein Verlauf, aber kein bedeutender Sprungpunkt wie ein PDF-Link oder ein manuell markierter Stack-Eintrag.

### 6.3 GoTo, PDF-Link, Marker = echter Stack-Eintrag

Ein echter Sprung wird als `LeaveKind::Stack` behandelt:

```rust
fn stack_jump(self: Pin<&mut Self>, target: i32) {
    self.go_to_page(target, LeaveKind::Stack);
}
```

Das passiert bei:

- GoTo (`g`)
- PDF-Link-Klick
- expliziter Stack-Markierung (`m`)

Diese Art von Navigation ist für die Anwendung besonders wichtig, weil sie einen sinnvollen Rückkehrpunkt definiert.

Beispiel:
- von Folie 3 auf eine Beamer-TOC-Folie springen
- dann zurück zu der Folie, von der aus du gestartet bist

Das muss in der History sauber funktionieren.

### 6.4 J-Menü = verwirft den forward-tail, aber keine normale Markierung

Beim J-Menü wird `LeaveKind::Menu` verwendet:

```rust
fn menu_jump(self: Pin<&mut Self>, target: i32) {
    self.go_to_page(target, LeaveKind::Menu);
}
```

Das entspricht `TruncateOnly`:
- der zukünftige Verlauf wird verworfen
- aber keine neue Stack-Markierung wird hinzugefügt

Das ist genau sinnvoll: Das Menü ist eine Auswahl aus vorhandenen Zielen, kein neuer bedeutender Punkt.

---

## 7. Ein konkretes Beispiel aus der Anwendung

Angenommen, der Nutzer arbeitet so:

1. Er beginnt auf Folie 1
2. Er geht auf Folie 2, 3, 4, 5 (normale Schritte)
3. Er klickt auf einen internen PDF-Link auf Folie 20
4. Er geht wieder zurück auf Folie 5
5. Er will den zurückliegenden Pfad wieder so sehen, wie er vor dem Link war

Ohne diese Unterscheidung würde die normale Folge von Folien 2,3,4,5 als "bedeutender Jump" behandelt werden. Das wäre falsch. In der Anwendung ist das nicht erwünscht.

Die Logik macht genau das Richtige:

- normales Blättern erzeugt keine Stack-Position
- der PDF-Link erzeugt einen echten Stack-Eintrag
- der Rück-/Vorwärts-History-Walk bewegt sich durch Verlauf, ohne als neuer Stack-Punkt zu gelten

Damit bleibt die sogenannte "jump stack" sauber und nützlich.

---

## 8. Kleine Visualisierung des Ablaufs

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

## 9. Warum dieses Design gut ist

Das ist ein sehr gutes Design für eine Präsentationssoftware, weil es eine klare Trennung macht zwischen:

- alltäglichem Folienfluss
- bewusster Sprungnavigation
- Verlauf von bereits besuchten Punkten

Das verhindert zwei typische Fehler:

1. Der normale Folienfluss füllt den Stack mit irrelevanten Einträgen
2. Ein PDF-Link wird als "nur ein Schritt" behandelt und verliert dadurch seine Bedeutung

Die Anwendung löst genau dieses Problem elegant.

---

## 10. Kurzfazit

Die Logik ist bewusst so gebaut, dass nur wirklich sinnvolle Sprünge den Stack beeinflussen. Der normale Folienbetrieb und der History-Verlauf sind von den "bedeutenden" Jump-Zielen getrennt.

Das Ergebnis ist:

- `+/-1` und `+/-10` sind fließende Präsentationsbewegungen, kein Stack-Eintrag
- History zurück/vorwärts ist Verlauf, kein Stack-Eintrag
- GoTo, PDF-Link und Markierung sind echte Sprünge und erzeugen daher einen Stack-Eintrag

Das ist exakt die richtige Semantik für eine PDF-Präsentations-Anwendung.

---

## 11. Relevante Stellen im Code

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

Diese Stellen zeigen die konkrete Anwendung der oben beschriebenen Regeln.
