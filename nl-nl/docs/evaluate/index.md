Met Orchestrata controleert u of een plan voldoet aan regels en voorkeuren. Dit voorbeeld heeft twee taken en twee plaatsen, genummerd 0 en 1. Elke taak kan op beide plaatsen terecht, maar de twee taken mogen niet dezelfde plaats krijgen. Voor taak A heeft plaats 1 de voorkeur.

Het voorbeeld gebruikt fictieve gegevens en draait lokaal. Deze website draait geen API en ontvangt geen uploads. Gebruik de [handleiding voor de lokale API](../api/) als u het voorbeeld via HTTP wilt uitvoeren.

## Voer het voorbeeld uit

Voer vanuit de hoofdmap van de Orchestrata-repository dit commando uit met een stabiele Rust-toolchain:

```sh
(cd backend && cargo run -p orchestrata-cli -- examples/assignment.json)
```

Het resultaat bevat `values: [1, 0]`; de score voor zowel harde regels als voorkeuren is nul. De waarden volgen de volgorde van de invoer: taak A krijgt plaats 1 en taak B plaats 0. Het resultaat voldoet aan de regel en volgt de voorkeur.

## Pas één ding aan

Kopieer het [toewijzingsvoorbeeld](https://github.com/ilstarno/Orchestrata/blob/9dab0545beb8cb9ded93cc632e41f0b6bc718e77/backend/examples/assignment.json) naar `backend/examples/preference-change.json`. Verander bij `prefer_value` voor taak A de `value` van `1` naar `0`. Voer hetzelfde commando uit met `examples/preference-change.json`. De verwachte toewijzing is nu `[0, 1]`. De harde regel blijft gelijk.

Geef beide taken in een nieuwe kopie alleen de optie `[0]`. Laat de regel over verschillende plaatsen staan. In een versie waarvan de exacte uitkomst is getoetst, toont de CLI `null`: er is geen geldige toewijzing voor deze invoer.

## Bepaal wat in het model hoort

- Maak een regel **hard** als het plan onbruikbaar is zodra de regel wordt overtreden.
- Gebruik een **voorkeur** als het plan ook bruikbaar blijft wanneer die voorkeur niet wordt gevolgd.
- Verwijder een optie helemaal als die nooit gekozen kan worden.

Begin met een voorbeeld dat u met de hand kunt controleren. Vergelijk de uitkomst met de toewijzingen die u verwacht, ook voor een geval zonder oplossing. Wordt de zoeklimiet overschreden, dan staat de uitkomst niet vast. Dat is niet hetzelfde als aantonen dat er geen geldig plan bestaat.
