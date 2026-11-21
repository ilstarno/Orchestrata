## Den Einstieg wählen

Öffnen Sie das [Zuweisungsbeispiel (Englisch)](/Orchestrata/examples/assignment/), um ein Ergebnis ohne Installation zu lesen. Für einen laufenden Server folgen Sie der [ersten HTTP-Anfrage (Englisch)](/Orchestrata/docs/solve-api/). Der Client benötigt kein Rust.

Rust ist die Programmiersprache von Engine und Server. Cargo baut Rust-Programme und verwaltet ihre Abhängigkeiten. Diese Werkzeuge werden nur zum Bauen aus Quellcode benötigt. Die [Serveranleitung (Englisch)](/Orchestrata/docs/backend-setup/) erklärt Installation und Versionsprüfung.

Alle Quellcodebefehle unten laufen im Stammverzeichnis des Orchestrata-Checkouts. Die Shell-Beispiele nutzen Linux, macOS oder WSL.

## Einen lokalen Server starten

Die [Serveranleitung (Englisch)](/Orchestrata/docs/backend-setup/) führt durch Installation und Start. Website und Server sind getrennt. Für die erste Anfrage sind weder Browseroberfläche noch Datenbank nötig.

```sh
ORCHESTRATA_HTTP_BIND=127.0.0.1:3000 \
  cargo run --manifest-path backend/Cargo.toml --locked -p orchestrata-api
```

## Eine native Bibliothek nutzen

Wählen Sie eine Sprachbindung im [SDK-Leitfaden (Englisch)](/Orchestrata/docs/sdk/). Er verwendet den öffentlichen Engine-Quellcode, eine passende native Bibliothek und eine lokale Paketinstallation.

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
