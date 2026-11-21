## Kies uw startpunt

Lees het [toewijzingsvoorbeeld (Engels)](/Orchestrata/examples/assignment/) zonder iets te installeren. Gebruik voor een bestaande server de [eerste HTTP-aanvraag (Engels)](/Orchestrata/docs/solve-api/). De client heeft geen Rust nodig.

Rust is de programmeertaal van de engine en server. Cargo bouwt Rust-programma’s en beheert afhankelijkheden. Alleen bouwen uit broncode vereist deze tools. De [serverhandleiding (Engels)](/Orchestrata/docs/backend-setup/) legt installatie en versiecontroles uit.

Alle broncodeopdrachten hieronder worden uitgevoerd vanuit de hoofdmap van de Orchestrata-checkout. De shellvoorbeelden gebruiken Linux, macOS of WSL.

Stuur voorbeeld-JSON naar de starters-API op je computer. Deze statische site biedt geen solver of aanmelding en stuurt geen verzoeken door.

## De service starten en aanroepen

Voer dit uit vanuit de hoofdmap van de Orchestrata-repository:

```sh
ORCHESTRATA_HTTP_BIND=127.0.0.1:3000 cargo run --manifest-path backend/Cargo.toml --locked -p orchestrata-api
```

Voer in een andere terminal, ook vanuit de hoofdmap van de repository, het volgende uit:

```sh
curl -fsS http://127.0.0.1:3000/api/v1/health
curl -fsS -X POST http://127.0.0.1:3000/api/v1/solve \
  -H 'Content-Type: application/json' \
  --data-binary @backend/examples/assignment.json
```

| Methode en pad | Doel |
| --- | --- |
| `GET /api/v1/health` | Controleren of het lokale HTTP-proces reageert. |
| `POST /api/v1/solve` | Een probleem met gehele waarden indienen voor een exacte oplossing binnen een begrensde zoekruimte. |
| `GET /api/v1/openapi.json` | Het schema ophalen dat de draaiende build beschikbaar stelt. |

Elke variabele heeft een naam en een array met toegestane gehele getallen. Beperkingen verwijzen naar posities in de array met variabelen, te beginnen bij nul. Raadpleeg het schema van de draaiende service voor de velden en tags. Gebruikt uw client standaard zwevendekommagetallen, zorg dan dat gehele getallen zonder precisieverlies worden verzonden.

## Resultaten en fouten interpreteren

Het voorbeeld geeft een JSON-object met het veld `solution` terug. De volgorde van de waarden komt overeen met die van de variabelen. De scores voor harde en zachte regels worden apart vermeld. De lokale startersversie onderzoekt bij exact zoeken maximaal 100.000 combinaties. Een verzoekfout, een bereikte zoeklimiet of een serverstoring bewijst niet dat het model onoplosbaar is. Een build met geverifieerd gedrag voor exacte resultaten geeft alleen `{"solution":null}` terug als de exacte zoekactie bewijst dat geen enkele planning aan alle harde regels voldoet.

## Catalogi voor sjablonen en fouten

Controleer de onderstaande routes in `GET /api/v1/openapi.json` voordat u ze aanroept. Deze site host de API niet.

| Methode en pad | Doel |
| --- | --- |
| `GET /api/v1/error-codes` | Stabiele foutcodes en hun betekenis ophalen. |
| `GET /api/v1/recipes` | De meegeleverde synthetische scenario’s tonen. |
| `GET /api/v1/recipes/{recipe_id}` | De beschrijving van een recept en de opgegeven velden ophalen. |
| `GET /api/v1/recipes/{recipe_id}/example` | Het synthetische probleem ophalen, zonder het op te lossen. |
| `POST /api/v1/recipes/{recipe_id}/run-example` | Los een meegeleverd voorbeeld op zonder verzoekinhoud. |
| `GET /api/v1/recipes/grid` | Filter en doorloop de metagegevens van de meegeleverde sjablonen per pagina. |

De aparte lokale MCP-server, indien aanwezig, biedt `list_recipes`, `describe_recipe` en `get_recipe_example` via standaardinvoer en standaarduitvoer. Hiermee kunt u uitsluitend synthetische voorbeelden opvragen en bekijken. De build accepteert geen netwerkverbindingen en voert geen berekeningen uit om een oplossing te vinden. Deze lokale evaluatiebuild biedt geen gehoste MCP, gegevens voor externe toegang of inlogfunctie.
