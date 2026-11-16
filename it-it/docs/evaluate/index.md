Con Orchestrata puoi verificare se un piano rispetta regole e preferenze. In questo esempio ci sono due attività e due posti, numerati 0 e 1. Ogni attività può occupare entrambi i posti, ma le due attività non possono occupare lo stesso posto. Per l’attività A è preferibile il posto 1.

L’esempio usa dati fittizi e si esegue in locale. Questo sito non gestisce l’API e non accetta caricamenti. Per eseguire l’esempio via HTTP, consulta la [guida all’API locale](../api/).

## Esegui l’esempio

Dalla cartella principale del repository Orchestrata, con una versione stabile degli strumenti Rust:

```sh
(cd backend && cargo run -p orchestrata-cli -- examples/assignment.json)
```

Il risultato contiene `values: [1, 0]`. I punteggi per i vincoli obbligatori e per le preferenze sono entrambi a zero. I valori seguono l’ordine dei dati in ingresso: l’attività A occupa il posto 1, la B il posto 0. La regola e la preferenza sono rispettate.

## Cambia una cosa

Copia l’[esempio di assegnazione](https://github.com/ilstarno/Orchestrata/blob/9dab0545beb8cb9ded93cc632e41f0b6bc718e77/backend/examples/assignment.json) in `backend/examples/preference-change.json`. Nel vincolo `prefer_value` dell’attività A cambia il campo `value` da `1` a `0`. Esegui lo stesso comando con `examples/preference-change.json`. L’assegnazione attesa ora è `[0, 1]`; la regola obbligatoria rimane invariata.

In una nuova copia, lascia a entrambe le attività solo l’opzione `[0]` e mantieni la regola dei posti distinti. In una versione con risultati esatti verificati, la CLI restituisce `null`: per questi dati non esiste un’assegnazione valida.

## Decidi cosa includere nel modello

- Rendi una regola **obbligatoria** se violarla rende il piano inutilizzabile.
- Usa una **preferenza** se il piano rimane utilizzabile anche quando non viene rispettata.
- Elimina un’opzione se non può mai essere scelta.

Inizia con un esempio abbastanza piccolo da poterlo controllare a mano. Confronta il piano ottenuto con le assegnazioni attese, includendo un caso senza soluzione. Se il limite di ricerca viene superato, non è stato possibile stabilire il risultato. Questo non equivale a dimostrare che non esiste un piano valido.
