Eine Planungsvorlage beschreibt eine kleine Planungsaufgabe und die benötigten Felder. Die ausführbaren Beispiele verwenden fiktive Daten. Lesen Sie die Beschreibung, führen Sie das JSON-Modell aus und vergleichen Sie die Zuordnungen mit dem erwarteten Ergebnis. Die Beispiele prüfen einzelne Entscheidungen; sie ersetzen kein Planungs- oder Dispositionssystem.

| Beispiel | Was Sie prüfen können | Was das Beispiel nicht abdeckt |
| --- | --- | --- |
| Techniker im Außendienst zuordnen | Überschneidende Termine auf verschiedene geeignete Techniker verteilen. | Laufende Einsatzsteuerung, Straßenrouten oder einen Tagesplan. |
| Ausfall in zwei Filialen | Den Ausgangsplan mit dem Plan nach einem Personalausfall vergleichen. Eine festgelegte Schicht bleibt bestehen. | Prognosen, Kalenderanbindung oder die Einhaltung arbeitsrechtlicher Vorgaben. |
| Abholungen und Lieferungen zuordnen | Fahrzeuge nach Ladung und Zeitfenster für jeden Auftrag auswählen. | Routen erstellen oder die Gesamtladung über mehrere Stopps berechnen. |
| Flughafengates zuordnen | Zeitlich überlappende Flüge auf geeignete freie Gates verteilen. | Live-Ankünfte, Gate-Systeme oder vollständige Flughafenplanung. |
| Callcenter besetzen | Gleichzeitig laufende Warteschlangen nach Sprachkenntnissen und Fähigkeiten besetzen. | Anrufprognosen, Serviceziele oder laufende Personalsteuerung. |

## Ein Beispiel aus dem Katalog ausführen

Checken Sie den [Vorlagenimport zur Prüfung](https://github.com/ilstarno/orchestrata-recipes/pull/1) aus und stellen Sie sicher, dass das gebaute Programm `orchestrata-cli` im Suchpfad liegt. Führen Sie im Stammverzeichnis des Vorlagen-Repositorys aus:

```sh
orchestrata-cli recipes/examples/fleet-pickup-delivery/problem.json
```

Das hinterlegte Ergebnis lautet `[1, 2, 0]`, ohne Strafpunkte für harte Regeln oder Präferenzen. Die fachlichen Eingaben stehen in `inputs.json`, das Vergleichsergebnis in `expected.json`. Die XML-Datei beschreibt das Szenario; die CLI erhält das normalisierte JSON-Modell. Der Import-Branch enthält Quellmaterial, keine versionierte Katalogfreigabe.

Für ein kleineres Beispiel ohne Katalog nutzen Sie die [Zuordnungsanleitung](../evaluate/). Wählen Sie für Ihr eigenes Beispiel eine Entscheidung, notieren Sie ihre erlaubten Optionen und modellieren Sie nur Regeln, die Sie prüfen können. Verwenden Sie keine echten Kunden- oder Beschäftigtendaten in der lokalen Vorschau. Synthetische Fuhrparkdaten liefern reproduzierbare Stopps; der Generator berechnet weder Routen noch Fahrzeiten.
