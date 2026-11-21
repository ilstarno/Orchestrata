## Scegli da dove iniziare

Leggi l’[esempio di assegnazione (inglese)](/Orchestrata/examples/assignment/) senza installare strumenti. Per un server esistente segui la [prima richiesta HTTP (inglese)](/Orchestrata/docs/solve-api/). Il client non richiede Rust.

Rust è il linguaggio del motore e del server. Cargo compila i programmi Rust e gestisce le dipendenze. Questi strumenti servono solo per compilare dal sorgente. La [guida del server (inglese)](/Orchestrata/docs/backend-setup/) spiega installazione e controllo delle versioni.

Tutti i comandi sul sorgente qui sotto partono dalla cartella principale del checkout Orchestrata. Gli esempi di shell usano Linux, macOS o WSL.

Due attività si dividono i posti 0 e 1. Entrambe possono usare qualsiasi posto, ma i posti assegnati devono essere distinti. L’attività A preferisce il posto 1.

Esegui l’esempio in locale. Il sito non ospita l’API e non accetta caricamenti. Per HTTP, consulta la [guida all’API locale](../api/).

## Esegui l’esempio

Dalla cartella principale del repository Orchestrata, con una versione stabile degli strumenti Rust:

```sh
cargo run --manifest-path backend/Cargo.toml --locked -p orchestrata-cli -- backend/examples/assignment.json
```

Il risultato contiene `values: [1, 0]`. I punteggi per i vincoli obbligatori e per le preferenze sono entrambi a zero. I valori seguono l’ordine dei dati in ingresso: l’attività A occupa il posto 1, la B il posto 0. La regola e la preferenza sono rispettate.

## Cambia una cosa

Copia l’`backend/examples/assignment.json` in `backend/examples/preference-change.json`. Nel vincolo `prefer_value` dell’attività A cambia il campo `value` da `1` a `0`. Esegui lo stesso comando con `backend/examples/preference-change.json`. L’assegnazione attesa ora è `[0, 1]`; la regola obbligatoria rimane invariata.

In una nuova copia, lascia a entrambe le attività solo l’opzione `[0]` e mantieni la regola dei posti distinti. In una versione con risultati esatti verificati, la CLI restituisce `null`: per questi dati non esiste un’assegnazione valida.

## Decidi cosa includere nel modello

- Rendi una regola **obbligatoria** se violarla rende il piano inutilizzabile.
- Usa una **preferenza** se il piano rimane utilizzabile anche quando non viene rispettata.
- Elimina un’opzione se non può mai essere scelta.

Inizia con un esempio abbastanza piccolo da poterlo controllare a mano. Confronta il piano ottenuto con le assegnazioni attese, includendo un caso senza soluzione. Se il limite di ricerca viene superato, non è stato possibile stabilire il risultato. Questo non equivale a dimostrare che non esiste un piano valido.
