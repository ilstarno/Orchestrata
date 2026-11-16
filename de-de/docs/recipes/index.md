Eine Planungsvorlage benennt die Aufgabe und ihre Eingabefelder. Führen Sie das JSON-Beispiel aus und vergleichen Sie die Zuordnung mit dem erwarteten Ergebnis. Jede Vorlage prüft eine Entscheidung, kein ganzes Planungs- oder Dispositionssystem.

| Beispiel | Was Sie prüfen können | Was das Beispiel nicht abdeckt |
| --- | --- | --- |
| Techniker im Außendienst zuordnen | Überschneidende Termine auf verschiedene geeignete Techniker verteilen. | Laufende Einsatzsteuerung, Straßenrouten oder einen Tagesplan. |
| Ausfall in zwei Filialen | Den Ausgangsplan mit dem Plan nach einem Personalausfall vergleichen. Eine festgelegte Schicht bleibt bestehen. | Prognosen, Kalenderanbindung oder die Einhaltung arbeitsrechtlicher Vorgaben. |
| Abholungen und Lieferungen zuordnen | Fahrzeuge nach Ladung und Zeitfenster für jeden Auftrag auswählen. | Routen erstellen oder die Gesamtladung über mehrere Stopps berechnen. |
| Flughafengates zuordnen | Zeitlich überlappende Flüge auf geeignete freie Gates verteilen. | Live-Ankünfte, Gate-Systeme oder vollständige Flughafenplanung. |
| Callcenter besetzen | Gleichzeitig laufende Warteschlangen nach Sprachkenntnissen und Fähigkeiten besetzen. | Anrufprognosen, Serviceziele oder laufende Personalsteuerung. |

## Ein Beispiel aus dem Katalog ausführen

Checken Sie den [Vorlagenkatalog](https://github.com/ilstarno/orchestrata-recipes) aus und stellen Sie sicher, dass das gebaute Programm `orchestrata-cli` im Suchpfad liegt. Führen Sie im Stammverzeichnis des Katalogs aus:

```sh
orchestrata-cli recipes/examples/fleet-pickup-delivery/problem.json
```

Das hinterlegte Ergebnis lautet `[1, 2, 0]`, ohne Strafpunkte für harte Regeln oder Präferenzen. Die fachlichen Eingaben stehen in `inputs.json`, das Vergleichsergebnis in `expected.json`. Die XML-Datei beschreibt das Szenario; die CLI erhält das normalisierte JSON-Modell. Das Katalog-Repository enthält Quellbeispiele, kein versioniertes Katalogpaket.

Für ein kleineres Beispiel ohne Katalog nutzen Sie die [Zuordnungsanleitung](../evaluate/). Wählen Sie für Ihr eigenes Beispiel eine Entscheidung, notieren Sie ihre erlaubten Optionen und modellieren Sie nur Regeln, die Sie prüfen können. Verwenden Sie keine echten Kunden- oder Beschäftigtendaten in der lokalen Vorschau. Synthetische Fuhrparkdaten liefern reproduzierbare Stopps; der Generator berechnet weder Routen noch Fahrzeiten.
