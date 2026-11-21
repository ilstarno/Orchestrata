## Kies uw startpunt

Lees het [toewijzingsvoorbeeld (Engels)](/Orchestrata/examples/assignment/) zonder iets te installeren. Gebruik voor een bestaande server de [eerste HTTP-aanvraag (Engels)](/Orchestrata/docs/solve-api/). De client heeft geen Rust nodig.

Rust is de programmeertaal van de engine en server. Cargo bouwt Rust-programma’s en beheert afhankelijkheden. Alleen bouwen uit broncode vereist deze tools. De [serverhandleiding (Engels)](/Orchestrata/docs/backend-setup/) legt installatie en versiecontroles uit.

Alle broncodeopdrachten hieronder worden uitgevoerd vanuit de hoofdmap van de Orchestrata-checkout. De shellvoorbeelden gebruiken Linux, macOS of WSL.

## Een lokale server starten

Volg de [serverhandleiding (Engels)](/Orchestrata/docs/backend-setup/) voor installatie en opstarten. Website en server zijn gescheiden. Voor de eerste aanvraag zijn geen browserinterface of database nodig.

```sh
ORCHESTRATA_HTTP_BIND=127.0.0.1:3000 \
  cargo run --manifest-path backend/Cargo.toml --locked -p orchestrata-api
```

## Een native bibliotheek gebruiken

Kies een taalbinding in de [SDK-handleiding (Engels)](/Orchestrata/docs/sdk/). Deze gebruikt de openbare enginebroncode, een bijpassende native bibliotheek en een lokale pakketinstallatie.

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
