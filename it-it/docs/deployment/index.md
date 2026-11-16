Avviare prima il servizio e l’interfaccia sul proprio computer. Il sito del progetto Orchestrata è statico: aprirlo non avvia alcuna API. L’anteprima è destinata alla valutazione locale con dati fittizi, non all’accesso pubblico o all’uso in produzione.

## API locale e interfaccia web

Usare una toolchain stabile di Rust per l’API, e Node.js 22 e npm per l’interfaccia web. Dalla directory principale del repository Orchestrata, eseguire:

```sh
(cd backend && cargo run -p orchestrata-api)
```

In un secondo terminale, eseguire:

```sh
(cd frontend && npm install && npm run dev)
```

L’API usa `127.0.0.1:3000`. Per i dettagli su richieste e risultati, consultare la [guida all’API locale](../api/). Aprire l’indirizzo dell’interfaccia indicato dal comando del frontend. Il server web di sviluppo inoltra le richieste a `/api/` al servizio locale. Verificare che il servizio risponda ed eseguire l’esempio di assegnazione incluso prima di provarne uno più ampio:

```sh
curl -fsS http://127.0.0.1:3000/api/v1/health
curl -fsS -X POST http://127.0.0.1:3000/api/v1/solve \
  -H 'Content-Type: application/json' \
  --data-binary @backend/examples/assignment.json
```

Limitare questa build all’interfaccia di loopback. Una verifica di disponibilità riuscita conferma soltanto che il processo risponde; non verifica l’autenticazione, la capacità del servizio o la disponibilità di un archivio persistente dei job.

## Valutare il motore autonomo e gli SDK

La [importazione del motore autonomo in revisione](https://github.com/ilstarno/orchestrata-engine/pull/1) include il motore Rust, la libreria nativa e i binding per C#, Java, Python, Node.js, PHP e Go. Il ramo non è un rilascio SDK né come pacchetto né come tag. Dalla copia locale del motore, con Rust stabile, eseguire:

```sh
cargo build --manifest-path backend/Cargo.toml --release --locked -p orchestrata-ffi
```

Consultare il README dell’SDK per i requisiti di caricamento e di runtime del linguaggio in uso. Usare la libreria nativa della stessa revisione estratta e un binding compatibile con l’ABI. La compilazione della libreria nativa su una piattaforma non dimostra che siano supportati pacchetti per altre piattaforme.

## Prima di usare un cluster

In un’attività di revisione separata sono state controllate la generazione dei manifesti sperimentali di deployment e la loro conformità allo schema. Questa guida locale non richiede né dà per scontati un’immagine per la produzione, una certificazione per servizi cloud gestiti, un endpoint pubblico o una configurazione di accesso. Per una valutazione su cluster occorrono una revisione verificata appositamente, un ingresso protetto, controlli di accesso testati, limiti espliciti alle risorse e verifiche documentate di installazione e rollback. Non esporre a Internet un’API in anteprima e non usare dati reali dei clienti per provarla.
