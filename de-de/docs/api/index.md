## Den Einstieg wählen

Öffnen Sie das [Zuweisungsbeispiel (Englisch)](/Orchestrata/examples/assignment/), um ein Ergebnis ohne Installation zu lesen. Für einen laufenden Server folgen Sie der [ersten HTTP-Anfrage (Englisch)](/Orchestrata/docs/solve-api/). Der Client benötigt kein Rust.

Rust ist die Programmiersprache von Engine und Server. Cargo baut Rust-Programme und verwaltet ihre Abhängigkeiten. Diese Werkzeuge werden nur zum Bauen aus Quellcode benötigt. Die [Serveranleitung (Englisch)](/Orchestrata/docs/backend-setup/) erklärt Installation und Versionsprüfung.

Alle Quellcodebefehle unten laufen im Stammverzeichnis des Orchestrata-Checkouts. Die Shell-Beispiele nutzen Linux, macOS oder WSL.

Senden Sie Beispiel-JSON an die Einstiegs-API auf Ihrem Rechner. Diese statische Seite stellt weder Solver noch Anmeldung oder Anfrageweiterleitung bereit.

## Dienst starten und aufrufen

Führen Sie im Stammverzeichnis des Orchestrata-Repositorys Folgendes aus:

```sh
ORCHESTRATA_HTTP_BIND=127.0.0.1:3000 cargo run --manifest-path backend/Cargo.toml --locked -p orchestrata-api
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

## Kataloge für Planungsvorlagen und Fehler

Prüfen Sie vor dem Aufruf in `GET /api/v1/openapi.json`, welche Pfade verfügbar sind. Diese Website betreibt keine API.

| Methode und Pfad | Zweck |
| --- | --- |
| `GET /api/v1/error-codes` | Stabile Fehlerkennungen und ihre Bedeutung abrufen. |
| `GET /api/v1/recipes` | Die mitgelieferten synthetischen Szenarien auflisten. |
| `GET /api/v1/recipes/{recipe_id}` | Die Beschreibung eines Rezepts und seine deklarierten Felder abrufen. |
| `GET /api/v1/recipes/{recipe_id}/example` | Das synthetische Planungsproblem zurückgeben, ohne es zu lösen. |
| `POST /api/v1/recipes/{recipe_id}/run-example` | Ein mitgeliefertes Beispiel ohne Anfrageinhalt lösen. |
| `GET /api/v1/recipes/grid` | Metadaten der Vorlagen filtern und seitenweise abrufen. |

Der separate lokale MCP-Server stellt, sofern enthalten, `list_recipes`, `describe_recipe` und `get_recipe_example` über Standardeingabe und Standardausgabe bereit. Er listet und liest ausschließlich synthetische Beispiele. Er nimmt keine Netzwerkverbindungen an und führt keine Berechnungen zur Lösung aus. Die lokale Evaluierungsversion bietet weder gehostetes MCP noch Zugangsdaten für den Fernzugriff oder eine Anmeldung.
