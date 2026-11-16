Envoyez un exemple JSON à l’API de démarrage sur votre poste. Ce site statique ne propose ni solveur, ni connexion, ni relais de requêtes.

## Démarrer et appeler le service

Exécutez la commande suivante à la racine du dépôt Orchestrata :

```sh
(cd backend && cargo run -p orchestrata-api)
```

Dans un autre terminal, toujours à la racine du dépôt, exécutez les commandes suivantes :

```sh
curl -fsS http://127.0.0.1:3000/api/v1/health
curl -fsS -X POST http://127.0.0.1:3000/api/v1/solve \
  -H 'Content-Type: application/json' \
  --data-binary @backend/examples/assignment.json
```

| Méthode et chemin | Fonction |
| --- | --- |
| `GET /api/v1/health` | Vérifier que le processus HTTP local répond. |
| `POST /api/v1/solve` | Soumettre un problème à variables entières pour une résolution exacte dans une limite de recherche fixée. |
| `GET /api/v1/openapi.json` | Consulter le schéma exposé par la version en cours d’exécution. |

Chaque variable possède un nom et un tableau d’entiers autorisés. Les contraintes font référence aux indices du tableau de variables, à partir de zéro. Consultez le schéma du service en cours d’exécution pour connaître les champs et les étiquettes. Si votre client utilise des nombres à virgule flottante par défaut, veillez à transmettre les entiers sans perte de précision.

## Interpréter les résultats et les erreurs

L’exemple renvoie un objet JSON contenant le champ `solution`. Les valeurs suivent l’ordre de vos variables. Les scores des contraintes impératives et des préférences sont distincts. La version locale de démarrage limite la recherche exacte à 100 000 combinaisons. Une erreur de requête, une limite de recherche atteinte ou une panne du serveur ne prouve pas que le modèle est irréalisable. Une version dont le contrat de résultat exact a été vérifié ne renvoie `{"solution":null}` que si la recherche exacte démontre qu’aucun plan ne satisfait l’ensemble des contraintes impératives.

## Catalogues des modèles et des erreurs

Vérifiez les routes ci-dessous dans `GET /api/v1/openapi.json` avant de les appeler. Ce site n’héberge pas l’API.

| Méthode et chemin | Fonction |
| --- | --- |
| `GET /api/v1/error-codes` | Consulter les identifiants d’erreur stables et leur signification. |
| `GET /api/v1/recipes` | Lister les scénarios synthétiques fournis. |
| `GET /api/v1/recipes/{recipe_id}` | Lire la description et les champs déclarés d’une recette. |
| `GET /api/v1/recipes/{recipe_id}/example` | Récupérer le problème synthétique sans le résoudre. |
| `POST /api/v1/recipes/{recipe_id}/run-example` | Résoudre un exemple fourni sans corps de requête. |
| `GET /api/v1/recipes/grid` | Filtrer et paginer les métadonnées des modèles fournis. |

Le serveur MCP local distinct, s’il est inclus, expose `list_recipes`, `describe_recipe` et `get_recipe_example` par l’entrée et la sortie standard. Elle permet uniquement de lister et de consulter des exemples synthétiques. Elle n’accepte pas de connexions réseau et ne lance aucune résolution. Cette version d’évaluation locale ne fournit ni MCP hébergé, ni identifiants d’accès à distance, ni fonction de connexion.
