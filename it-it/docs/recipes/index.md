Un modello identifica il problema e i campi di input. Esegui l’esempio JSON e confronta l’assegnazione con il risultato atteso. Ogni modello verifica una decisione, non un sistema completo di pianificazione o dispatch.

| Esempio | Che cosa puoi verificare | Che cosa non dimostra |
| --- | --- | --- |
| Assegnazione dei tecnici sul campo | Distribuire appuntamenti sovrapposti tra tecnici diversi e idonei. | Gestione degli interventi in tempo reale, itinerari stradali o turni per l’intera giornata. |
| Assenza in due negozi | Confrontare il piano iniziale con quello dopo un’assenza, mantenendo un turno fisso. | Previsioni, integrazioni con calendari o conformità alle norme sul lavoro. |
| Ritiri e consegne | Assegnare le attività ai veicoli che possono trasportare il carico e coprire la fascia oraria. | Costruzione dei percorsi o capacità cumulativa su più fermate. |
| Assegnazione dei gate | Assegnare voli sovrapposti a gate compatibili e disponibili. | Arrivi in tempo reale, sistemi di gestione dei gate o pianificazione completa dell’aeroporto. |
| Turni per un call center | Coprire più code contemporanee in base alla lingua e alle competenze. | Previsione dei volumi di chiamata, livelli di servizio o gestione del personale in tempo reale. |

## Esegui un esempio del catalogo

Scarica il [catalogo dei modelli](https://github.com/ilstarno/orchestrata-recipes) e assicurati che l’eseguibile compilato `orchestrata-cli` sia nel percorso di ricerca. Esegui il comando dalla cartella principale del catalogo:

```sh
orchestrata-cli recipes/examples/fleet-pickup-delivery/problem.json
```

Il risultato salvato è `[1, 2, 0]`, senza penalità per regole obbligatorie o preferenze. Leggi `inputs.json` per i dati in ingresso e `expected.json` per confrontare il risultato. L’XML descrive lo scenario; la CLI riceve il modello JSON normalizzato. Il repository del catalogo contiene esempi sorgente, non un pacchetto con un numero di versione.

Usa la [guida all’assegnazione](../evaluate/) per provare un esempio più piccolo senza il catalogo. Per i tuoi esempi scegli una decisione, elenca le opzioni ammesse e modella soltanto le regole che puoi controllare. Non usare dati reali di clienti o dipendenti nelle prove locali. I dati sintetici per flotte producono elenchi di fermate ripetibili; il generatore non calcola percorsi né tempi di viaggio.
