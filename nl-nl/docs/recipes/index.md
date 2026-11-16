Een planningssjabloon benoemt het probleem en de invoervelden. Draai het JSON-voorbeeld en vergelijk de toewijzing met de verwachte uitkomst. Elk sjabloon test één beslissing, geen volledig planning- of dispatchsysteem.

| Voorbeeld | Wat u kunt bekijken | Wat het voorbeeld niet laat zien |
| --- | --- | --- |
| Monteurs toewijzen | Verdeel overlappende afspraken over verschillende monteurs die voor het werk in aanmerking komen. | Live aansturing, routeplanning of een volledige dagplanning. |
| Personeelsuitval in twee winkels | Vergelijk het oorspronkelijke rooster met een rooster na een afmelding, met behoud van een vaste dienst. | Voorspellingen, agenda-integratie of naleving van arbeidsregels. |
| Ophalen en bezorgen | Wijs taken toe aan voertuigen die de lading aankunnen en gedurende het hele tijdvenster beschikbaar zijn. | Voertuigroutes opbouwen of de totale belading over meerdere stops bepalen. |
| Luchthavengates toewijzen | Wijs overlappende vluchten toe aan geschikte, beschikbare gates. | Live aankomsten, gatesystemen of volledige luchthavenplanning. |
| Callcenter bezetten | Bezet gelijktijdige wachtrijen op basis van taal en vaardigheden. | Voorspelling van het belvolume, serviceniveaus of live personeelsbeheer. |

## Voer een catalogusvoorbeeld uit

Open een checkout van de [sjablooncatalogus](https://github.com/ilstarno/orchestrata-recipes) en zorg dat het gebouwde programma `orchestrata-cli` in uw zoekpad staat. Voer dit uit vanuit de hoofdmap van de catalogus:

```sh
orchestrata-cli recipes/examples/fleet-pickup-delivery/problem.json
```

De opgeslagen uitkomst is `[1, 2, 0]`, zonder strafpunten voor harde regels of voorkeuren. Lees `inputs.json` voor de invoergegevens en vergelijk de uitkomst met `expected.json`. De XML beschrijft de situatie; de CLI krijgt het genormaliseerde JSON-model. De catalogusrepository bevat bronvoorbeelden, geen versiegebonden cataloguspakket.

Gebruik de [toewijzingshandleiding](../evaluate/) voor een kleiner voorbeeld zonder catalogus. Kies voor uw eigen model één beslissing, noteer de toegestane opties en modelleer alleen regels die u kunt controleren. Gebruik geen echte klant- of personeelsgegevens in een lokale preview. Synthetische wagenparkgegevens leveren reproduceerbare stops op; de generator berekent geen routes of reistijden.
