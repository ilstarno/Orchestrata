## Scegli da dove iniziare

Leggi l’[esempio di assegnazione (inglese)](/Orchestrata/examples/assignment/) senza installare strumenti. Per un server esistente segui la [prima richiesta HTTP (inglese)](/Orchestrata/docs/solve-api/). Il client non richiede Rust.

Rust è il linguaggio del motore e del server. Cargo compila i programmi Rust e gestisce le dipendenze. Questi strumenti servono solo per compilare dal sorgente. La [guida del server (inglese)](/Orchestrata/docs/backend-setup/) spiega installazione e controllo delle versioni.

Tutti i comandi sul sorgente qui sotto partono dalla cartella principale del checkout Orchestrata. Gli esempi di shell usano Linux, macOS o WSL.

## Avviare un server locale

Segui la [guida del server (inglese)](/Orchestrata/docs/backend-setup/) per installazione e avvio. Il sito e il server sono separati. La prima richiesta non richiede un’interfaccia browser o un database.

```sh
ORCHESTRATA_HTTP_BIND=127.0.0.1:3000 \
  cargo run --manifest-path backend/Cargo.toml --locked -p orchestrata-api
```

## Usare una libreria nativa

Scegli il binding nella [guida SDK (inglese)](/Orchestrata/docs/sdk/). Usa il sorgente pubblico del motore, la libreria nativa corrispondente e un’installazione locale del pacchetto.

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
