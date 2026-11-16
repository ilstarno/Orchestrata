Starten Sie den Dienst und die Oberfläche zunächst auf Ihrem Rechner. Die Projektwebsite von Orchestrata ist statisch. Wenn Sie sie öffnen, wird keine API gestartet. Diese Anleitung dient der lokalen Evaluierung mit fiktiven Daten, nicht dem öffentlichen Zugriff oder dem Produktivbetrieb.

## Lokale API und Weboberfläche

Verwenden Sie für die API eine stabile Rust-Toolchain und für die Weboberfläche Node.js 22 und npm. Führen Sie im Stammverzeichnis des Orchestrata-Repositorys Folgendes aus:

```sh
(cd backend && cargo run -p orchestrata-api)
```

Führen Sie in einem zweiten Terminal Folgendes aus:

```sh
(cd frontend && npm install && npm run dev)
```

Die API verwendet `127.0.0.1:3000`. Einzelheiten zu Anfragen und Ergebnissen finden Sie im [Leitfaden zur lokalen API](../api/). Öffnen Sie die Adresse der Oberfläche, die der Frontend-Befehl ausgibt. Der Entwicklungsserver leitet Anfragen an `/api/` an den lokalen Dienst weiter. Prüfen Sie die Erreichbarkeit und führen Sie das mitgelieferte Zuordnungsbeispiel aus, bevor Sie ein größeres Beispiel testen:

```sh
curl -fsS http://127.0.0.1:3000/api/v1/health
curl -fsS -X POST http://127.0.0.1:3000/api/v1/solve \
  -H 'Content-Type: application/json' \
  --data-binary @backend/examples/assignment.json
```

Beschränken Sie diesen Build auf die Loopback-Schnittstelle. Eine erfolgreiche Statusprüfung bestätigt nur, dass ein Prozess antwortet. Sie bestätigt weder eine funktionierende Authentifizierung noch die Kapazität des Dienstes oder die Verfügbarkeit eines persistenten Auftragsspeichers.

## Eigenständige Engine und SDKs evaluieren

Das [Repository der eigenständigen Engine](https://github.com/ilstarno/orchestrata-engine/tree/main) enthält die Rust-Engine, die native Bibliothek und Anbindungen für C#, Java, Python, Node.js, PHP und Go. Bauen Sie das SDK aus dem passenden Quellstand. Führen Sie im Checkout der Engine mit stabilem Rust Folgendes aus:

```sh
cargo build --manifest-path backend/Cargo.toml --release --locked -p orchestrata-ffi
```

Die Anforderungen an das Laden der Bibliothek und die Laufzeitumgebung Ihrer Sprache stehen in der README des SDKs. Verwenden Sie die native Bibliothek aus demselben Checkout und eine ABI-kompatible Anbindung. Dass sich die native Bibliothek auf einer Plattform bauen lässt, belegt nicht, dass für andere Plattformen Pakete unterstützt werden.

## Vor der Verwendung eines Clusters

Die experimentellen Deployment-Manifeste wurden im Rahmen einer separaten Prüfung daraufhin überprüft, ob sie korrekt gerendert werden und dem Schema entsprechen. Dieser lokale Leitfaden setzt weder ein Produktionsimage noch eine Zertifizierung für verwaltete Cloud-Dienste, einen öffentlichen Endpunkt oder eine Anmeldekonfiguration voraus und trifft dazu auch keine Zusage. Für eine Cluster-Evaluierung brauchen Sie eine eigens geprüfte Revision, geschützten eingehenden Zugriff, getestete Zugriffskontrollen, explizite Ressourcenlimits und Nachweise für Installation und Rollback. Stellen Sie eine Vorschau-API nicht ins Internet und verwenden Sie keine realen Kundendaten zum Testen.
