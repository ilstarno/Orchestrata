Avvia l’API e l’interfaccia web sul tuo computer. Usa dati di esempio e non esporre questa build a Internet. Il sito non avvia l’API.

## API locale e interfaccia web

Usa una versione stabile di Rust con Cargo e curl. Per l’interfaccia web, installa anche Node.js 22 e npm. Apri il [repository Orchestrata](https://github.com/ilstarno/Orchestrata). Scarica il codice sorgente e verifica che il workspace Cargo contenga `orchestrata-api`. Controlla i percorsi facoltativi nel documento OpenAPI del servizio.

Dalla directory principale del repository, esegui i test del workspace con le dipendenze bloccate e avvia l’API sull’interfaccia di loopback:

```sh
cargo test --manifest-path backend/Cargo.toml --locked --workspace
ORCHESTRATA_HTTP_BIND=127.0.0.1:3000 cargo run --manifest-path backend/Cargo.toml --locked -p orchestrata-api
```

Lascia l’API in esecuzione. Da un secondo terminale, nella directory principale, verifica la risposta, richiedi le prime tre righe dell’elenco delle ricette ed esegui l’esempio di assegnazione incluso:

```sh
curl -fsS http://127.0.0.1:3000/api/v1/health
curl -fsS 'http://127.0.0.1:3000/api/v1/recipes/grid?page_size=3'
curl -fsS -X POST http://127.0.0.1:3000/api/v1/solve \
  -H 'Content-Type: application/json' \
  --data-binary @backend/examples/assignment.json
```

La verifica dello stato conferma che il processo risponde. L’elenco restituisce i metadati delle ricette e non invia lavori. L’assegnazione attesa è `[1,0]` nell’ordine dei dati in ingresso. Questi controlli non verificano accesso, archiviazione persistente dei lavori o capacità del servizio.

Per usare l’interfaccia web, lascia attiva l’API e apri un terminale nel [repository separato del frontend](https://github.com/ilstarno/orchestrata-frontend) se hai accesso. Verifica `frontend/package.json` e confronta lo schema API con quello del backend in esecuzione.

```sh
(cd frontend && npm install && npm run dev)
```

Apri l’indirizzo locale mostrato dal comando. Il server di sviluppo inoltra `/api` a `127.0.0.1:3000`. Se imposti un’altra porta di loopback in `ORCHESTRATA_HTTP_BIND`, aggiorna sia gli indirizzi curl sia la destinazione del proxy di sviluppo.

La [guida all’API locale](../api/) spiega richieste e risposte. Per problemi di avvio o di proxy, consulta la [guida alla configurazione e alla risoluzione dei problemi del backend (in inglese)](../../../docs/backend-setup/).

## Valutare il motore autonomo e gli SDK

Il [repository del motore autonomo](https://github.com/ilstarno/orchestrata-engine) include il motore Rust, la libreria nativa e i binding per C#, Java, Python, Node.js, PHP e Go. Compila l’SDK dai sorgenti corrispondenti. Dalla copia locale del motore, con Rust stabile, esegui:

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

Esegui il comando nel repository del motore autonomo, non in quello dell’API citato sopra. Il comando individua il file di compilazione da usare.

Consultare il README dell’SDK per i requisiti di caricamento e di runtime del linguaggio in uso. Usare la libreria nativa della stessa revisione estratta e un binding compatibile con l’ABI. La compilazione della libreria nativa su una piattaforma non dimostra che siano supportati pacchetti per altre piattaforme.

## Lista di controllo prima di un test su cluster

Valuta il cluster separatamente. I controlli locali non dimostrano che un’immagine, la configurazione di un servizio cloud gestito o l’accesso pubblico siano pronti. Usa solo dati di esempio e lascia disattivato l’accesso pubblico.

1. Registra il digest dell’immagine testata. Usa un namespace dedicato, non di produzione.
2. Imposta richieste e limiti per CPU e memoria. Da un client autorizzato nella rete di test, controlla lo stato, l’elenco delle ricette e l’assegnazione.
3. Verifica che l’accesso dall’esterno dell’ambiente consentito sia bloccato. Controlla i log senza registrare credenziali o dati dei clienti.
4. Installa la versione da valutare, torna all’immagine precedente e ripeti i controlli. Se aggiungi storage persistente, verifica separatamente backup e ripristino.

## Container, Kubernetes e Terraform

Crea l’immagine dell’API dal Dockerfile e installala con Helm in un cluster Kubernetes privato già esistente. La guida Terraform riguarda cluster Amazon EKS, Azure AKS e Google GKE già operativi. Non crea infrastruttura cloud.

Per Dockerfile, Helm e Terraform, consulta `deploy/` e `deploy/terraform/` se presenti.

Il chart ha superato i controlli Helm locali. Il container e l’installazione su cluster gestiti non sono stati testati.
