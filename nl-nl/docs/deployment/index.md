Start de service en de gebruikersinterface eerst op uw eigen werkstation. De projectsite van Orchestrata is statisch. Als u de site opent, wordt er geen API gestart. De preview is bedoeld voor lokale evaluatie met fictieve gegevens, niet voor openbare toegang of productiegebruik.

## Lokale API en webinterface

Gebruik een stabiele Rust-toolchain voor de API en Node.js 22 en npm voor de webinterface. Voer dit uit vanuit de hoofdmap van de Orchestrata-repository:

```sh
(cd backend && cargo run -p orchestrata-api)
```

Voer dit uit in een tweede terminal:

```sh
(cd frontend && npm install && npm run dev)
```

De API gebruikt `127.0.0.1:3000`. Zie de [handleiding voor de lokale API](../api/) voor informatie over verzoeken en resultaten. Open het adres dat de frontendopdracht weergeeft. De ontwikkelserver stuurt verzoeken aan `/api/` door naar de lokale service. Controleer of de service reageert en voer het meegeleverde toewijzingsvoorbeeld uit voordat u een groter voorbeeld probeert:

```sh
curl -fsS http://127.0.0.1:3000/api/v1/health
curl -fsS -X POST http://127.0.0.1:3000/api/v1/solve \
  -H 'Content-Type: application/json' \
  --data-binary @backend/examples/assignment.json
```

Beperk deze build tot de loopbackinterface. Een geslaagde statuscontrole bevestigt alleen dat het proces reageert. Dit zegt niets over authenticatie, de capaciteit van de service of de beschikbaarheid van persistente taakopslag.

## De zelfstandige engine en de SDK’s evalueren

De [repository van de zelfstandige engine](https://github.com/ilstarno/orchestrata-engine/tree/main) bevat de Rust-engine, de native bibliotheek en bindings voor C#, Java, Python, Node.js, PHP en Go. Bouw de SDK uit de bijbehorende broncode. Voer vanuit de checkout van de engine met stabiel Rust het volgende uit:

```sh
cargo build --manifest-path backend/Cargo.toml --release --locked -p orchestrata-ffi
```

Raadpleeg de README van de SDK voor de vereisten voor het laden van de bibliotheek en de runtime van uw taal. Gebruik de native bibliotheek uit dezelfde checkout en een ABI-compatibele binding. Dat u de native bibliotheek op één platform kunt bouwen, bewijst niet dat pakketdistributie op andere platforms wordt ondersteund.

## Voordat u een cluster gebruikt

De experimentele deploymentmanifests zijn bij een afzonderlijke beoordeling gecontroleerd op correcte rendering en overeenstemming met het schema. Deze lokale handleiding vereist noch belooft een productie-image, certificering voor beheerde clouddiensten, een openbaar eindpunt of een inlogconfiguratie. Voor een evaluatie op een cluster zijn een specifiek beoordeelde revisie, afgeschermde ingress, geteste toegangscontroles, expliciete resourcelimieten en bewijs van installatie en rollback nodig. Stel een preview-API niet bloot aan internet en gebruik geen echte klantgegevens om te zien of de API werkt.
