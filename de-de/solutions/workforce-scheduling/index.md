## Vier Schichten trotz eines Ausfalls besetzen

Beide Filialen brauchen um 14 Uhr jeweils zwei Mitarbeitende. Ben fällt aus. Vergleichen Sie den ursprünglichen Plan mit einer zweiten Berechnung, in der Ben für die beiden möglichen Einsätze nicht mehr zur Verfügung steht. Schichten, Regeln, Gewichtungen und alle anderen Besetzungsmöglichkeiten bleiben gleich.

### Welche Angaben Sie brauchen

Geben Sie für jede Schicht den Standort, die Aufgabe und die qualifizierten, verfügbaren Mitarbeitenden an. Erfassen Sie außerdem feste Zuweisungen und Wünsche, von denen der Plan abweichen darf. Die Nummern der Mitarbeitenden bleiben in beiden Durchläufen gleich, damit sich die Ergebnisse vergleichen lassen.

| Rezeptfeld | Typ | Beispielwert |
| --- | --- | --- |
| `shift-names` | Liste von Textwerten | Nord Schlüsselverantwortung, Nord Verkaufsfläche, Süd Kasse, Süd Warenlager |
| `worker-names` | Liste von Textwerten | Ana, Ben, Clara, Diego, Elena; Nummern 0 bis 4 |
| `eligible-pairs` | Liste von Ganzzahlen | Paare aus Schicht- und Personalnummern; bei Bens Ausfall die Paare 1,1 und 2,1 entfernen |
| `pinned-pairs` | Optionale Liste von Ganzzahlen | 0,0 weist Ana fest die Schlüsselverantwortung in Filiale Nord zu |
| `preferred-assignments` | Optionale Liste von Ganzzahlen | Dreiergruppen aus Schichtnummer, Personalnummer und Gewicht: 1,1,5; 2,2,3; 3,3,2 |

In den Eingabedaten ist jede Schicht eine Variable. Für eine feste Zuweisung gibt es genau eine zulässige Person: Für `north_keyholder` gilt `domain: [0]`. Die zwingende Regel `all_different` bleibt in beiden Durchläufen gleich. Die drei `prefer_value`-Regeln behalten ihre Gewichte, auch wenn sich ein Wunsch nicht mehr erfüllen lässt.

### Pläne vergleichen

| Schicht | Vor dem Ausfall verfügbar und qualifiziert | Ursprünglicher Plan | Nach dem Ausfall |
| --- | --- | --- | --- |
| Nord Schlüsselverantwortung | Ana | Ana | Ana |
| Nord Verkaufsfläche | Ben, Clara | Ben | Clara |
| Süd Kasse | Ben, Clara, Elena | Clara | Elena |
| Süd Warenlager | Diego, Elena | Diego | Diego |

Beide Pläne decken alle vier Schichten ohne eine Person doppelt einzuplanen ab. Der ursprüngliche Durchlauf ergibt `[0, 1, 2, 3]`, eine Bewertung von 0/0 für zwingende bzw. bevorzugte Regeln und zwölf geprüfte Kombinationen. Nach dem Ausfall sind es `[0, 2, 4, 3]`, 0/−8 und vier geprüfte Kombinationen. Die −8 setzen sich aus zwei nicht erfüllten Wünschen zusammen: 5 Punkte für Ben auf der Verkaufsfläche Nord und 3 für Clara an der Kasse Süd. Eine zwingende Regel wird dadurch nicht verletzt.

### Beide Varianten ausführen

Speichern Sie die [ursprüngliche Eingabe](baseline-problem.json) und die [angepasste Eingabe](adjusted-problem.json):

```sh
orchestrata-cli baseline-problem.json
orchestrata-cli adjusted-problem.json
```

Vergleichen Sie die [ursprüngliche Ausgabe](baseline-expected.json) mit der [angepassten Ausgabe](adjusted-expected.json). Beide Eingaben lassen sich auch über die [lokale API](../../docs/api/) ausführen.

### Was dieses Beispiel nicht abdeckt

Die vier Schichten stehen bereits fest und überschneiden sich. Das Beispiel enthält keine Bedarfsprognose, Kalendersynchronisierung, Lohnabrechnung, Berechnung von Überstunden oder Ruhezeiten, Prüfung arbeitsrechtlicher Vorgaben oder mehrwöchige Dienstplanung.
