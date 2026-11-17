Start de API en webinterface op je computer. Gebruik voorbeeldgegevens en houd deze build buiten het openbare internet. De site start geen API.

## Lokale API en webinterface

Gebruik een stabiele Rust-versie met Cargo en curl. Installeer voor de webinterface ook Node.js 22 en npm. Open de [Orchestrata-repository](https://github.com/ilstarno/Orchestrata). Haal de broncode op en controleer of de Cargo-werkruimte `orchestrata-api` bevat. Controleer optionele routes in het OpenAPI-document van de draaiende API.

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

Laat de API draaien voor de webinterface en open, als u toegang hebt, een terminal in de [aparte frontend-repository](https://github.com/ilstarno/orchestrata-frontend). Controleer `frontend/package.json` en vergelijk het API-schema met de draaiende backend.

```sh
(cd frontend && npm install && npm run dev)
```

Open het lokale adres dat de opdracht afdrukt. De ontwikkelserver stuurt `/api` door naar `127.0.0.1:3000`. Kies je via `ORCHESTRATA_HTTP_BIND` een andere loopbackpoort, pas dan zowel de curl-adressen als de doelserver van de ontwikkelproxy aan.

De [handleiding voor de lokale API](../api/) beschrijft de aanvragen en antwoorden. Raadpleeg bij start- of proxyproblemen de [backendhandleiding en probleemoplossing (Engels)](../../../docs/backend-setup/).

## De zelfstandige engine en de SDK’s evalueren

De [repository van de zelfstandige engine](https://github.com/ilstarno/orchestrata-engine) bevat de Rust-engine, de native bibliotheek en bindings voor C#, Java, Python, Node.js, PHP en Go. Bouw de SDK uit de bijbehorende broncode. Voer vanuit de checkout van de engine met stabiel Rust het volgende uit:

```sh
(
  if [ -f Cargo.toml ] && [ -f crates/ffi/include/orchestrata.h ]; then
      engine_manifest=Cargo.toml
  elif [ -f backend/Cargo.toml ] && [ -f backend/crates/ffi/include/orchestrata.h ]; then
      engine_manifest=backend/Cargo.toml
  else
      printf '%s\n' 'Run these commands from the standalone engine repository.' >&2
      exit 1
  fi
  cargo build --manifest-path "$engine_manifest" --release --locked -p orchestrata-ffi
)
```

Voer dit uit in de repository van de zelfstandige engine, niet in de API-repository hierboven. Het commando zoekt zelf het juiste Cargo-bestand.

Raadpleeg de README van de SDK voor de vereisten voor het laden van de bibliotheek en de runtime van uw taal. Gebruik de native bibliotheek uit dezelfde checkout en een ABI-compatibele binding. Dat u de native bibliotheek op één platform kunt bouwen, bewijst niet dat pakketdistributie op andere platforms wordt ondersteund.

## Checklist vóór een clustertest

Beoordeel clustergebruik apart. De controles hierboven tonen niet aan dat een containerimage, configuratie voor een beheerde cloud of publieke toegang klaar is voor gebruik. Gebruik alleen voorbeeldgegevens en laat publieke toegang uitgeschakeld.

1. Noteer de digest van de geteste image. Gebruik een eigen namespace buiten productie.
2. Stel aanvragen en limieten in voor CPU en geheugen. Controleer de status, receptenlijst en toewijzing vanaf een toegestane client in het testnetwerk.
3. Controleer dat toegang van buiten de toegestane omgeving wordt geblokkeerd. Controleer de logs zonder inlog- of klantgegevens vast te leggen.
4. Installeer de te testen versie, keer terug naar het vorige image en herhaal de controles. Voeg je blijvende opslag toe, test back-ups en herstel dan afzonderlijk.

## Containers, Kubernetes en Terraform

Bouw de API-image met de Dockerfile en installeer deze met Helm in een bestaand privécluster. De Terraform-handleiding behandelt bestaande clusters in Amazon EKS, Azure AKS en Google GKE. Er wordt geen cloudinfrastructuur aangemaakt.

Raadpleeg voor Dockerfile, Helm en Terraform `deploy/` en `deploy/terraform/` als deze mappen aanwezig zijn.

De Helm-controles zijn lokaal geslaagd. De container en de installatie op beheerde clusters zijn niet getest.
