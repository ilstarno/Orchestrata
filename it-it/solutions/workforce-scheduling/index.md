## Coprire quattro turni nonostante un’assenza

Alle 14:00 servono due persone in ciascuno dei due negozi. Ben non è più disponibile. Confronta il piano iniziale con un secondo calcolo che esclude Ben dai due turni che poteva coprire. I turni, le regole, i pesi e le altre assegnazioni possibili restano invariati.

### Dati da preparare

Per ogni turno indica il negozio, il ruolo e le persone qualificate e disponibili. Aggiungi le assegnazioni fisse e le preferenze a cui il piano può derogare. Mantieni gli stessi indici del personale nei due calcoli, così i risultati sono confrontabili.

| Campo della ricetta | Tipo | Valore di esempio |
| --- | --- | --- |
| `shift-names` | Elenco di stringhe | Nord addetto con le chiavi, Nord sala vendita, Sud cassa, Sud magazzino |
| `worker-names` | Elenco di stringhe | Ana, Ben, Clara, Diego, Elena; indici da 0 a 4 |
| `eligible-pairs` | Elenco di interi | Coppie di indici turno e persona; per l’assenza di Ben rimuovi 1,1 e 2,1 |
| `pinned-pairs` | Elenco facoltativo di interi | 0,0 assegna ad Ana il turno di addetta con le chiavi al Nord |
| `preferred-assignments` | Elenco facoltativo di interi | Terne di turno, persona e peso: 1,1,5; 2,2,3; 3,3,2 |

Nei dati in ingresso al risolutore, ogni turno è una variabile. Un’assegnazione fissa ammette una sola persona: `north_keyholder` ha `domain: [0]`. La regola obbligatoria `all_different` non cambia tra i due calcoli. Le tre regole `prefer_value` mantengono i pesi originali anche quando una preferenza non può più essere soddisfatta.

### Confrontare i piani

| Turno | Persone idonee prima dell’assenza | Piano iniziale | Dopo l’assenza |
| --- | --- | --- | --- |
| Nord addetto con le chiavi | Ana | Ana | Ana |
| Nord sala vendita | Ben, Clara | Ben | Clara |
| Sud cassa | Ben, Clara, Elena | Clara | Elena |
| Sud magazzino | Diego, Elena | Diego | Diego |

Entrambi i piani coprono tutti e quattro i turni senza assegnare la stessa persona a due turni contemporanei. Il calcolo iniziale restituisce `[0, 1, 2, 3]`, punteggi di 0/0 per regole obbligatorie e preferenze, e verifica 12 combinazioni. Dopo l’assenza il risultato è `[0, 2, 4, 3]`, con 0/−8 e 4 combinazioni verificate. Il −8 dipende da due preferenze non rispettate: 5 punti per Ben in sala vendita al Nord e 3 per Clara alla cassa al Sud. Non c’è alcuna violazione delle regole obbligatorie.

### Eseguire entrambe le versioni

Salva i [dati iniziali](baseline-problem.json) e i [dati aggiornati](adjusted-problem.json):

```sh
orchestrata-cli baseline-problem.json
orchestrata-cli adjusted-problem.json
```

Confronta il [risultato iniziale](baseline-expected.json) e il [risultato aggiornato](adjusted-expected.json). Puoi anche eseguire entrambi i calcoli tramite l’[API locale](../../docs/api/).

### Limiti dell’esempio

I quattro turni sono già definiti e si sovrappongono. L’esempio non prevede la domanda, non sincronizza calendari, non calcola stipendi, straordinari o riposi e non verifica gli obblighi di legge sul lavoro. Non prepara un piano su più settimane.
