L’API introduttiva accetta un piccolo problema di pianificazione in formato JSON. Il sito è statico: non ospita un risolutore, non offre funzioni di accesso e non inoltra richieste al computer in uso. Avviare il servizio in locale e usare dati fittizi per la valutazione.

## Avviare e chiamare il servizio

Eseguire questo comando dalla directory principale del repository Orchestrata:

```sh
(cd backend && cargo run -p orchestrata-api)
```

In un altro terminale, sempre dalla directory principale del repository, eseguire:

```sh
curl -fsS http://127.0.0.1:3000/api/v1/health
curl -fsS -X POST http://127.0.0.1:3000/api/v1/solve \
  -H 'Content-Type: application/json' \
  --data-binary @backend/examples/assignment.json
```

| Metodo e percorso | Scopo |
| --- | --- |
| `GET /api/v1/health` | Verificare che il processo HTTP locale risponda. |
| `POST /api/v1/solve` | Inviare un problema con variabili a dominio intero per una risoluzione esatta entro un limite di ricerca. |
| `GET /api/v1/openapi.json` | Consultare lo schema esposto dalla build in esecuzione. |

Ogni variabile ha un nome e un array di interi ammessi. I vincoli fanno riferimento alle posizioni nell’array delle variabili, a partire da zero. Consultare lo schema del servizio in esecuzione per conoscere campi e tag. Se il client usa numeri in virgola mobile per impostazione predefinita, assicurarsi di inviare gli interi senza perdita di precisione.

## Interpretare risultati ed errori

L’esempio restituisce un oggetto JSON con il campo `solution`. I valori restituiti seguono l’ordine delle variabili. I punteggi per i vincoli obbligatori e per le preferenze sono distinti. La versione introduttiva locale limita la ricerca esatta a 100.000 combinazioni. Un errore nella richiesta, il raggiungimento del limite o un errore del server non dimostrano che il modello non abbia soluzioni. Una build con semantica verificata per i risultati esatti restituisce `{"solution":null}` solo quando la ricerca esatta dimostra che nessun piano soddisfa tutti i vincoli obbligatori.

## Ricette e catalogo degli errori in revisione

I percorsi che seguono sono stati implementati nell’ambito di una revisione o di un’integrazione locale. Consultare la [bozza HTTP in revisione](https://github.com/ilstarno/Orchestrata/pull/22) per le modifiche che contiene. L’insieme dei percorsi integrati in locale può essere diverso. Verificare la revisione in esecuzione: questi percorsi non sono necessariamente presenti in un checkout predefinito o in un servizio ospitato.

| Metodo e percorso | Scopo |
| --- | --- |
| `GET /api/v1/error-codes` | Consultare gli identificativi stabili degli errori e il loro significato. |
| `GET /api/v1/recipes` | Elencare gli scenari sintetici inclusi. |
| `GET /api/v1/recipes/{recipe_id}` | Leggere la descrizione e i campi dichiarati di una ricetta. |
| `GET /api/v1/recipes/{recipe_id}/example` | Ottenere il problema sintetico senza eseguirlo. |
| `POST /api/v1/recipes/{recipe_id}/run-example` | Risolvere un esempio incluso senza inviare un corpo della richiesta. Il percorso fa parte del lavoro di revisione locale e non è un’interfaccia rilasciata. |

Una build locale MCP separata, in revisione, espone `list_recipes`, `describe_recipe` e `get_recipe_example` tramite input e output standard. Consente soltanto di elencare e leggere esempi sintetici. Non accetta connessioni di rete e non esegue risoluzioni. La build di valutazione locale non offre un MCP ospitato, credenziali per l’accesso remoto o funzioni di accesso.
