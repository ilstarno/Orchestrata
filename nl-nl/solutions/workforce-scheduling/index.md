## Vier diensten bezetten na een afmelding

Beide winkels hebben om 14.00 uur elk twee medewerkers nodig. Ben is niet meer beschikbaar. Vergelijk het oorspronkelijke plan met een tweede berekening waarin Ben niet meer in aanmerking komt voor de twee diensten die hij kon draaien. De diensten, regels, gewichten en overige mogelijkheden veranderen niet.

### Gegevens die u nodig hebt

Vermeld per dienst de locatie, de rol en alle bevoegde en beschikbare medewerkers. Leg ook vaste toewijzingen vast en geef aan van welke voorkeuren mag worden afgeweken. Gebruik voor de medewerkers in beide berekeningen dezelfde indexnummers, zodat de resultaten vergelijkbaar blijven.

| Receptveld | Type | Voorbeeldwaarde |
| --- | --- | --- |
| `shift-names` | Lijst met tekstwaarden | Noord sleutelhouder, Noord winkelvloer, Zuid kassa, Zuid voorraad |
| `worker-names` | Lijst met tekstwaarden | Ana, Ben, Clara, Diego, Elena; indexnummers 0 t/m 4 |
| `eligible-pairs` | Lijst met gehele getallen | Paren van dienst- en medewerkerindex; verwijder 1,1 en 2,1 bij de afmelding van Ben |
| `pinned-pairs` | Optionele lijst met gehele getallen | 0,0 wijst Ana vast toe als sleutelhouder in Noord |
| `preferred-assignments` | Optionele lijst met gehele getallen | Reeksen van dienstindex, medewerkerindex en gewicht: 1,1,5; 2,2,3; 3,3,2 |

In de invoer voor de oplosser is elke dienst een variabele. Bij een vaste toewijzing is precies één medewerker toegestaan: voor `north_keyholder` geldt `domain: [0]`. De verplichte regel `all_different` blijft in beide berekeningen gelijk. De drie `prefer_value`-regels houden hun oorspronkelijke gewicht, ook als een voorkeur onhaalbaar wordt.

### De plannen vergelijken

| Dienst | Geschikte medewerkers vóór de afmelding | Oorspronkelijk | Na de afmelding |
| --- | --- | --- | --- |
| Noord sleutelhouder | Ana | Ana | Ana |
| Noord winkelvloer | Ben, Clara | Ben | Clara |
| Zuid kassa | Ben, Clara, Elena | Clara | Elena |
| Zuid voorraad | Diego, Elena | Diego | Diego |

Beide plannen bezetten alle vier de diensten zonder iemand dubbel in te plannen. De oorspronkelijke berekening geeft `[0, 1, 2, 3]`, scores van 0/0 voor verplichte regels en voorkeuren, en 12 gecontroleerde combinaties. Na de afmelding zijn dat `[0, 2, 4, 3]`, 0/−8 en 4 gecontroleerde combinaties. De −8 komt door twee niet vervulde voorkeuren: 5 punten voor Ben op de winkelvloer in Noord en 3 voor Clara aan de kassa in Zuid. Er wordt geen verplichte regel overtreden.

### Beide varianten uitvoeren

Sla de [oorspronkelijke invoer](baseline-problem.json) en de [aangepaste invoer](adjusted-problem.json) op:

```sh
orchestrata-cli baseline-problem.json
orchestrata-cli adjusted-problem.json
```

Vergelijk de [oorspronkelijke uitvoer](baseline-expected.json) met de [aangepaste uitvoer](adjusted-expected.json). Beide berekeningen kunnen ook via de [lokale API](../../docs/api/) worden gestart.

### Wat dit voorbeeld niet doet

De vier diensten staan al vast en overlappen elkaar. Het voorbeeld doet geen vraagvoorspelling, agendakoppeling of loonberekening. Het modelleert geen overwerk of rusttijden, toetst niet aan arbeidswetgeving en maakt geen meerweeks rooster.
