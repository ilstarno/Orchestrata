Die Einsteiger-API nimmt ein kleines Planungsproblem im JSON-Format entgegen. Diese Website ist statisch: Sie hostet keinen Solver, bietet keine Anmeldung und leitet keine Aufrufe an Ihren Rechner weiter. Führen Sie den Dienst lokal aus und verwenden Sie für die Evaluierung fiktive Daten.

## Dienst starten und aufrufen

Führen Sie im Stammverzeichnis des Orchestrata-Repositorys Folgendes aus:

```sh
(cd backend && cargo run -p orchestrata-api)
```

Führen Sie in einem weiteren Terminal ebenfalls im Stammverzeichnis des Repositorys Folgendes aus:

```sh
curl -fsS http://127.0.0.1:3000/api/v1/health
curl -fsS -X POST http://127.0.0.1:3000/api/v1/solve \
  -H 'Content-Type: application/json' \
  --data-binary @backend/examples/assignment.json
```

| Methode und Pfad | Zweck |
| --- | --- |
| `GET /api/v1/health` | Prüfen, ob der lokale HTTP-Prozess antwortet. |
| `POST /api/v1/solve` | Ein Problem mit ganzzahligen Variablenwerten zur exakten Lösung innerhalb eines festen Suchlimits einreichen. |
| `GET /api/v1/openapi.json` | Das vom laufenden Build bereitgestellte Schema abrufen. |

Jede Variable hat einen Namen und ein Array zulässiger Ganzzahlen. Die Nebenbedingungen verweisen auf Positionen im Variablenarray, beginnend bei null. Welche Felder und Tags unterstützt werden, steht im Schema des laufenden Dienstes. Wenn Ihr Client standardmäßig Gleitkommazahlen verwendet, achten Sie darauf, Ganzzahlen ohne Genauigkeitsverlust zu übertragen.

## Ergebnisse und Fehler auswerten

Das Beispiel gibt ein JSON-Objekt mit dem Feld `solution` zurück. Die Werte stehen in der Reihenfolge Ihrer Variablen. Die Bewertungen für harte und weiche Regeln werden getrennt ausgewiesen. Die lokale Einstiegsversion prüft bei der exakten Suche höchstens 100.000 Kombinationen. Ein Anfragefehler, ein erreichtes Suchlimit oder ein Serverfehler beweist nicht, dass das Modell unlösbar ist. Ein Build mit verifizierter Semantik für exakte Ergebnisse liefert `{"solution":null}` nur dann zurück, wenn die exakte Suche nachweist, dass kein Plan alle harten Regeln erfüllt.

## Rezepte und Fehlerkatalog in der Review-Phase

Die folgenden Pfade wurden in Review-Entwürfen oder bei der lokalen Integration umgesetzt. Welche Änderungen der [HTTP-Entwurf zur Prüfung](https://github.com/ilstarno/Orchestrata/pull/22) enthält, steht im Pull Request. Der lokal zusammengeführte Stand kann andere Pfade umfassen. Prüfen Sie die verwendete Revision. Gehen Sie nicht davon aus, dass diese Pfade nach einem Standard-Checkout oder bei einem gehosteten Dienst verfügbar sind.

| Methode und Pfad | Zweck |
| --- | --- |
| `GET /api/v1/error-codes` | Stabile Fehlerkennungen und ihre Bedeutung abrufen. |
| `GET /api/v1/recipes` | Die mitgelieferten synthetischen Szenarien auflisten. |
| `GET /api/v1/recipes/{recipe_id}` | Die Beschreibung eines Rezepts und seine deklarierten Felder abrufen. |
| `GET /api/v1/recipes/{recipe_id}/example` | Das synthetische Planungsproblem zurückgeben, ohne es zu lösen. |
| `POST /api/v1/recipes/{recipe_id}/run-example` | Ein mitgeliefertes Beispiel ohne Anfrageinhalt lösen. Dieser Pfad gehört zur lokalen Review-Arbeit und ist keine veröffentlichte Schnittstelle. |

Ein separater lokaler MCP-Review-Build stellt `list_recipes`, `describe_recipe` und `get_recipe_example` über Standardeingabe und Standardausgabe bereit. Er listet und liest ausschließlich synthetische Beispiele. Er nimmt keine Netzwerkverbindungen an und führt keine Berechnungen zur Lösung aus. Die lokale Evaluierungsversion bietet weder gehostetes MCP noch Zugangsdaten für den Fernzugriff oder eine Anmeldung.
