Starten Sie API und Weboberfläche auf Ihrem Rechner. Nutzen Sie Beispieldaten und halten Sie diese Version aus dem öffentlichen Internet. Die Website startet keine API.

## Lokale API und Weboberfläche

Verwenden Sie stabiles Rust mit Cargo sowie curl. Für die Weboberfläche brauchen Sie auch Node.js 22 und npm. Öffnen Sie das [Orchestrata-Repository](https://github.com/ilstarno/Orchestrata). Besorgen Sie den Quellcode und prüfen Sie, ob der Cargo-Workspace `orchestrata-api` enthält. Prüfen Sie optionale Pfade im laufenden OpenAPI-Dokument.

Führen Sie im Stammverzeichnis des Repositorys zunächst die Workspace-Tests mit den im Lockfile festgelegten Abhängigkeiten aus. Starten Sie anschließend die API auf der Loopback-Schnittstelle:

```sh
cargo test --manifest-path backend/Cargo.toml --locked --workspace
ORCHESTRATA_HTTP_BIND=127.0.0.1:3000 cargo run --manifest-path backend/Cargo.toml --locked -p orchestrata-api
```

Lassen Sie die API laufen. Prüfen Sie in einem zweiten Terminal aus dem Stammverzeichnis den Statusendpunkt, rufen Sie die ersten drei Zeilen der Rezeptübersicht ab und starten Sie das mitgelieferte Zuordnungsbeispiel:

```sh
curl -fsS http://127.0.0.1:3000/api/v1/health
curl -fsS 'http://127.0.0.1:3000/api/v1/recipes/grid?page_size=3'
curl -fsS -X POST http://127.0.0.1:3000/api/v1/solve \
  -H 'Content-Type: application/json' \
  --data-binary @backend/examples/assignment.json
```

Die Statusabfrage zeigt nur, dass der Prozess antwortet. Die Rezeptübersicht liefert Metadaten zu Rezepten und reicht keine Aufträge ein. Die erwartete Zuordnung lautet `[1,0]` in der Reihenfolge der Eingaben. Diese Prüfungen sagen nichts über Anmeldung, dauerhafte Auftragsspeicherung oder die Kapazität des Dienstes aus.

Lassen Sie die API für die Weboberfläche laufen. Öffnen Sie, sofern Sie Zugriff haben, ein Terminal im separaten [Frontend-Repository](https://github.com/ilstarno/orchestrata-frontend). Prüfen Sie `frontend/package.json` und vergleichen Sie das API-Schema mit dem laufenden Backend.

```sh
(cd frontend && npm install && npm run dev)
```

Öffnen Sie die vom Befehl ausgegebene lokale Adresse. Der Entwicklungsserver leitet `/api` an `127.0.0.1:3000` weiter. Falls Sie in `ORCHESTRATA_HTTP_BIND` einen anderen Loopback-Port setzen, passen Sie die curl-Adressen und das Ziel des Entwicklungsproxys an.

Der [Leitfaden zur lokalen API](../api/) erklärt Anfragen und Antworten. Bei Start- oder Proxyproblemen hilft die [Backend-Einrichtung und Fehlerbehebung (Englisch)](../../../docs/backend-setup/).

## Eigenständige Engine und SDKs evaluieren

Das [Repository der eigenständigen Engine](https://github.com/ilstarno/orchestrata-engine) enthält die Rust-Engine, die native Bibliothek und Anbindungen für C#, Java, Python, Node.js, PHP und Go. Bauen Sie das SDK aus dem passenden Quellstand. Führen Sie im Checkout der Engine mit stabilem Rust Folgendes aus:

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

Führen Sie den Befehl im Repository der eigenständigen Engine aus, nicht im oben genannten API-Repository. Der Befehl findet die passende Build-Datei automatisch.

Die Anforderungen an das Laden der Bibliothek und die Laufzeitumgebung Ihrer Sprache stehen in der README des SDKs. Verwenden Sie die native Bibliothek aus demselben Checkout und eine ABI-kompatible Anbindung. Dass sich die native Bibliothek auf einer Plattform bauen lässt, belegt nicht, dass für andere Plattformen Pakete unterstützt werden.

## Checkliste vor einem Test im Cluster

Bewerten Sie die Nutzung im Cluster separat. Die lokalen Prüfungen weisen nicht nach, dass ein Container-Image, die Konfiguration einer verwalteten Cloud oder der öffentliche Zugriff einsatzbereit sind. Verwenden Sie nur Beispieldaten und deaktivieren Sie den öffentlichen Zugriff.

1. Notieren Sie den Digest des getesteten Images. Verwenden Sie einen eigenen Namespace außerhalb der Produktion.
2. Legen Sie Anforderungen und Obergrenzen für CPU und Arbeitsspeicher fest. Prüfen Sie Status, Rezeptübersicht und Zuordnung von einem zugelassenen Client im Testnetz.
3. Prüfen Sie, ob der Zugriff von außerhalb der vorgesehenen Grenze blockiert ist. Achten Sie bei den Protokollen darauf, dass keine Zugangsdaten oder Kundendaten erfasst werden.
4. Installieren Sie die zu prüfende Version, kehren Sie zum vorherigen Image zurück und wiederholen Sie die Prüfungen. Falls Sie dauerhaften Speicher hinzufügen, testen Sie Sicherung und Wiederherstellung separat.

## Container, Kubernetes und Terraform

Erstellen Sie das API-Image mit dem Dockerfile und installieren Sie es mit Helm in einem vorhandenen privaten Kubernetes-Cluster. Die Terraform-Anleitung gilt für vorhandene Amazon-EKS-, Azure-AKS- und Google-GKE-Cluster und legt keine Cloud-Infrastruktur an.

Das Dockerfile, das Helm-Chart und die Terraform-Hinweise finden Sie, sofern vorhanden, unter `deploy/` und `deploy/terraform/`.

Das Chart hat die lokalen Helm-Prüfungen bestanden. Container und Rollout in verwalteten Clustern wurden nicht getestet.
