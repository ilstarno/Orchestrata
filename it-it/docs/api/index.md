Invia un esempio JSON all’API di base sul tuo computer. Questo sito statico non offre né risolutore, né accesso, né inoltro delle richieste.

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

## Cataloghi dei modelli e degli errori

I percorsi seguenti sono disponibili nella [revisione pubblica verificata](https://github.com/ilstarno/Orchestrata/tree/b8d1271e3faddfffaf220698e381197bcae1752f). Per le altre revisioni consulta lo schema OpenAPI del servizio in esecuzione. Il servizio non è ospitato.

| Metodo e percorso | Scopo |
| --- | --- |
| `GET /api/v1/error-codes` | Consultare gli identificativi stabili degli errori e il loro significato. |
| `GET /api/v1/recipes` | Elencare gli scenari sintetici inclusi. |
| `GET /api/v1/recipes/{recipe_id}` | Leggere la descrizione e i campi dichiarati di una ricetta. |
| `GET /api/v1/recipes/{recipe_id}/example` | Ottenere il problema sintetico senza eseguirlo. |
| `POST /api/v1/recipes/{recipe_id}/run-example` | Risolvere un esempio incluso senza corpo della richiesta. |
| `GET /api/v1/recipes/grid` | Filtrare e consultare per pagina i metadati dei modelli inclusi. |

Il server MCP locale separato di questa revisione espone `list_recipes`, `describe_recipe` e `get_recipe_example` tramite input e output standard. Consente soltanto di elencare e leggere esempi sintetici. Non accetta connessioni di rete e non esegue risoluzioni. La build di valutazione locale non offre un MCP ospitato, credenziali per l’accesso remoto o funzioni di accesso.
