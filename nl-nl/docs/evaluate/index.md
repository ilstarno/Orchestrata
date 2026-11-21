## Kies uw startpunt

Lees het [toewijzingsvoorbeeld (Engels)](/Orchestrata/examples/assignment/) zonder iets te installeren. Gebruik voor een bestaande server de [eerste HTTP-aanvraag (Engels)](/Orchestrata/docs/solve-api/). De client heeft geen Rust nodig.

Rust is de programmeertaal van de engine en server. Cargo bouwt Rust-programma’s en beheert afhankelijkheden. Alleen bouwen uit broncode vereist deze tools. De [serverhandleiding (Engels)](/Orchestrata/docs/backend-setup/) legt installatie en versiecontroles uit.

Alle broncodeopdrachten hieronder worden uitgevoerd vanuit de hoofdmap van de Orchestrata-checkout. De shellvoorbeelden gebruiken Linux, macOS of WSL.

Twee taken delen plaatsen 0 en 1. Elke taak kan op beide plaatsen terecht, maar de taken moeten verschillende plaatsen krijgen. Taak A geeft de voorkeur aan plaats 1.

Draai het voorbeeld lokaal. Deze site draait geen API en ontvangt geen uploads. Voor HTTP zie de [handleiding voor de lokale API](../api/).

## Voer het voorbeeld uit

Voer vanuit de hoofdmap van de Orchestrata-repository dit commando uit met een stabiele Rust-toolchain:

```sh
cargo run --manifest-path backend/Cargo.toml --locked -p orchestrata-cli -- backend/examples/assignment.json
```

Het resultaat bevat `values: [1, 0]`; de score voor zowel harde regels als voorkeuren is nul. De waarden volgen de volgorde van de invoer: taak A krijgt plaats 1 en taak B plaats 0. Het resultaat voldoet aan de regel en volgt de voorkeur.

## Pas één ding aan

Kopieer het `backend/examples/assignment.json` naar `backend/examples/preference-change.json`. Verander bij `prefer_value` voor taak A de `value` van `1` naar `0`. Voer hetzelfde commando uit met `backend/examples/preference-change.json`. De verwachte toewijzing is nu `[0, 1]`. De harde regel blijft gelijk.

Geef beide taken in een nieuwe kopie alleen de optie `[0]`. Laat de regel over verschillende plaatsen staan. In een versie waarvan de exacte uitkomst is getoetst, toont de CLI `null`: er is geen geldige toewijzing voor deze invoer.

## Bepaal wat in het model hoort

- Maak een regel **hard** als het plan onbruikbaar is zodra de regel wordt overtreden.
- Gebruik een **voorkeur** als het plan ook bruikbaar blijft wanneer die voorkeur niet wordt gevolgd.
- Verwijder een optie helemaal als die nooit gekozen kan worden.

Begin met een voorbeeld dat u met de hand kunt controleren. Vergelijk de uitkomst met de toewijzingen die u verwacht, ook voor een geval zonder oplossing. Wordt de zoeklimiet overschreden, dan staat de uitkomst niet vast. Dat is niet hetzelfde als aantonen dat er geen geldig plan bestaat.
