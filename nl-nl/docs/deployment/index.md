Start de dienst en de webinterface eerst op je eigen computer. De Orchestrata-projectsite is statisch: als je de site opent, start er geen API. Gebruik voorbeeldgegevens en stel deze versie niet open voor het internet.

## Lokale API en webinterface

Gebruik een stabiele Rust-versie met Cargo en curl. Installeer voor de webinterface ook Node.js 22 en npm. [Download het openbare startproject bij deze handleiding](https://github.com/ilstarno/Orchestrata/archive/b8d1271e3faddfffaf220698e381197bcae1752f.zip), pak het uit en open een terminal in de uitgepakte map.

Voer vanuit de hoofdmap eerst de workspacetests uit met de vastgelegde afhankelijkheden en start daarna de API op de loopbackinterface:

```sh
cargo test --manifest-path backend/Cargo.toml --locked --workspace
ORCHESTRATA_HTTP_BIND=127.0.0.1:3000 cargo run --manifest-path backend/Cargo.toml --locked -p orchestrata-api
```

Laat de API draaien. Open een tweede terminal in de hoofdmap, controleer of de API reageert, vraag de eerste drie rijen van de receptenlijst op en voer het meegeleverde toewijzingsvoorbeeld uit:

```sh
curl -fsS http://127.0.0.1:3000/api/v1/health
curl -fsS 'http://127.0.0.1:3000/api/v1/recipes/grid?page_size=3'
curl -fsS -X POST http://127.0.0.1:3000/api/v1/solve \
  -H 'Content-Type: application/json' \
  --data-binary @backend/examples/assignment.json
```

De statuscontrole bevestigt dat het proces reageert. Het receptenoverzicht geeft metadata terug en start geen taken. De verwachte toewijzing is `[1,0]` in invoervolgorde. Deze controles bewijzen niet dat aanmelden, blijvende taakopslag of capaciteit op orde zijn.

Laat de API draaien en gebruik een andere terminal om de webinterface uit dezelfde checkout te starten:

```sh
(cd frontend && npm install && npm run dev)
```

Open het lokale adres dat de opdracht afdrukt. De ontwikkelserver stuurt `/api` door naar `127.0.0.1:3000`. Kies je via `ORCHESTRATA_HTTP_BIND` een andere loopbackpoort, pas dan zowel de curl-adressen als de doelserver van de ontwikkelproxy aan.

De [handleiding voor de lokale API](../api/) beschrijft de aanvragen en antwoorden. Raadpleeg bij start- of proxyproblemen de [backendhandleiding en probleemoplossing (Engels)](../../../docs/backend-setup/).

## De zelfstandige engine en de SDK’s evalueren

De [repository van de zelfstandige engine](https://github.com/ilstarno/orchestrata-engine/tree/d7b920e32e5402d96b59b424983199f82c7e8699) bevat de Rust-engine, de native bibliotheek en bindings voor C#, Java, Python, Node.js, PHP en Go. Bouw de SDK uit de bijbehorende broncode. Voer vanuit de checkout van de engine met stabiel Rust het volgende uit:

```sh
cargo build --manifest-path backend/Cargo.toml --release --locked -p orchestrata-ffi
```

Deze bouwopdracht geldt voor de checkout van de engine, niet voor die van de openbare API hierboven.

Raadpleeg de README van de SDK voor de vereisten voor het laden van de bibliotheek en de runtime van uw taal. Gebruik de native bibliotheek uit dezelfde checkout en een ABI-compatibele binding. Dat u de native bibliotheek op één platform kunt bouwen, bewijst niet dat pakketdistributie op andere platforms wordt ondersteund.

## Checklist vóór een clustertest

Beoordeel clustergebruik apart. De controles hierboven tonen niet aan dat een containerimage, configuratie voor een beheerde cloud of publieke toegang klaar is voor gebruik. Gebruik alleen voorbeeldgegevens en laat publieke toegang uitgeschakeld.

1. Leg de gecontroleerde bronrevisie en de digest van het daarmee gebouwde image vast. Gebruik een eigen namespace buiten productie.
2. Stel aanvragen en limieten in voor CPU en geheugen. Controleer de status, receptenlijst en toewijzing vanaf een toegestane client in het testnetwerk.
3. Controleer dat toegang van buiten de toegestane omgeving wordt geblokkeerd. Controleer de logs zonder inlog- of klantgegevens vast te leggen.
4. Installeer de te testen versie, keer terug naar het vorige image en herhaal de controles. Voeg je blijvende opslag toe, test back-ups en herstel dan afzonderlijk.

## Containers, Kubernetes en Terraform

Bouw de API-image met de Dockerfile en installeer deze met Helm in een bestaand privécluster. De Terraform-handleiding behandelt bestaande clusters in Amazon EKS, Azure AKS en Google GKE. Er wordt geen cloudinfrastructuur aangemaakt.

- [Handleiding voor containers en Helm (Engels)](https://github.com/ilstarno/Orchestrata/tree/2c921cedad7e20d3c1a892b35024d3465e336d75/deploy)
- [Terraform-handleiding (Engels)](https://github.com/ilstarno/Orchestrata/tree/2c921cedad7e20d3c1a892b35024d3465e336d75/deploy/terraform)

De Helm-controles zijn lokaal geslaagd. De container en de installatie op beheerde clusters zijn niet getest.
