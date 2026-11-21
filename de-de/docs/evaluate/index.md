## Den Einstieg wählen

Öffnen Sie das [Zuweisungsbeispiel (Englisch)](/Orchestrata/examples/assignment/), um ein Ergebnis ohne Installation zu lesen. Für einen laufenden Server folgen Sie der [ersten HTTP-Anfrage (Englisch)](/Orchestrata/docs/solve-api/). Der Client benötigt kein Rust.

Rust ist die Programmiersprache von Engine und Server. Cargo baut Rust-Programme und verwaltet ihre Abhängigkeiten. Diese Werkzeuge werden nur zum Bauen aus Quellcode benötigt. Die [Serveranleitung (Englisch)](/Orchestrata/docs/backend-setup/) erklärt Installation und Versionsprüfung.

Alle Quellcodebefehle unten laufen im Stammverzeichnis des Orchestrata-Checkouts. Die Shell-Beispiele nutzen Linux, macOS oder WSL.

Zwei Aufgaben werden auf die Plätze 0 und 1 verteilt. Beide Plätze sind für jede Aufgabe möglich, aber jede braucht einen eigenen Platz. Aufgabe A bevorzugt Platz 1.

Führen Sie das Beispiel lokal aus. Diese Website betreibt keine API und nimmt keine Uploads an. Für HTTP siehe die [Anleitung zur lokalen API](../api/).

## Beispiel ausführen

Führen Sie mit einer stabilen Rust-Toolchain im Stammverzeichnis des Orchestrata-Repositorys Folgendes aus:

```sh
cargo run --manifest-path backend/Cargo.toml --locked -p orchestrata-cli -- backend/examples/assignment.json
```

Das Ergebnis enthält `values: [1, 0]`. Die Bewertungen für harte und weiche Regeln sind beide null. Die Werte folgen der Eingabereihenfolge: Aufgabe A erhält Platz 1 und Aufgabe B Platz 0. Die Regel und die Präferenz sind erfüllt.

## Eine Änderung ausprobieren

Kopieren Sie das `backend/examples/assignment.json` nach `backend/examples/preference-change.json`. Ändern Sie für Aufgabe A im Eintrag `prefer_value` den `value` von `1` in `0`. Führen Sie denselben Befehl mit `backend/examples/preference-change.json` aus. Die erwartete Zuordnung ist jetzt `[0, 1]`. Die harte Regel bleibt unverändert.

Geben Sie in einer neuen Kopie beiden Aufgaben nur die Option `[0]`. Lassen Sie die Regel für verschiedene Plätze unverändert. In einer Version mit geprüftem exaktem Ergebnis gibt die CLI `null` aus: Für diese Eingabe gibt es keine gültige Zuordnung.

## Was gehört ins Modell?

- Eine Regel ist **hart**, wenn ein Verstoß den Plan unbrauchbar macht.
- Eine **Präferenz** kann unerfüllt bleiben, ohne den Plan unbrauchbar zu machen.
- Entfernen Sie eine Option ganz, wenn sie für eine Entscheidung nie infrage kommt.

Beginnen Sie mit einem Beispiel, das Sie von Hand prüfen können. Vergleichen Sie den ausgegebenen Plan mit Ihren erwarteten Zuordnungen, auch für einen unlösbaren Fall. Wird das Suchlimit überschritten, steht das Ergebnis nicht fest. Das ist etwas anderes als der Nachweis, dass kein gültiger Plan existiert.
