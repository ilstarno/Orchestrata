Un modèle de planification décrit une situation simple et les champs qu’elle utilise. Les exemples à exécuter contiennent des données fictives. Lisez la description, exécutez le modèle JSON, puis comparez les affectations au résultat attendu. Les exemples vérifient des décisions précises ; ils ne remplacent pas votre système de planification ou de répartition des interventions.

| Exemple | Ce que vous pouvez examiner | Ce que l’exemple ne démontre pas |
| --- | --- | --- |
| Affectation de techniciens sur le terrain | Répartir des rendez-vous simultanés entre plusieurs techniciens qualifiés. | Répartition en temps réel, itinéraires routiers ou planning complet d’une journée. |
| Absence dans deux magasins | Comparer le planning initial au planning après une absence, tout en conservant un poste fixé. | Prévisions, intégration d’agendas ou conformité au droit du travail. |
| Affectation des enlèvements et livraisons | Choisir des véhicules capables de transporter la charge et disponibles sur le créneau. | Construction des tournées ou calcul de la charge cumulée sur plusieurs arrêts. |
| Affectation des portes d’embarquement | Affecter des vols simultanés à des portes adaptées et libres. | Arrivées en temps réel, systèmes de gestion des portes ou planification complète d’un aéroport. |
| Équipe de centre d’appels | Couvrir des files d’attente simultanées selon les langues et les compétences requises. | Prévisions du volume d’appels, niveaux de service ou gestion des équipes en temps réel. |

## Exécuter un exemple du catalogue

Récupérez la branche du [projet d’import des modèles](https://github.com/ilstarno/orchestrata-recipes/pull/1) et vérifiez que l’exécutable compilé `orchestrata-cli` est dans votre chemin de recherche. À la racine du dépôt des modèles, exécutez :

```sh
orchestrata-cli recipes/examples/fleet-pickup-delivery/problem.json
```

Le résultat enregistré est `[1, 2, 0]`, sans pénalité pour les règles obligatoires ni pour les préférences. Lisez `inputs.json` pour voir les données d’entrée et `expected.json` pour vérifier le résultat. Le XML décrit la situation ; la CLI reçoit le modèle JSON normalisé. La branche d’import séparée contient les sources, pas une version publiée du catalogue.

Suivez le [guide d’affectation](../evaluate/) pour exécuter un exemple plus simple sans le catalogue. Pour créer le vôtre, choisissez une décision, notez les options autorisées et ne modélisez que les règles vérifiables. N’utilisez pas de données réelles de clients ou de salariés dans un aperçu local. Les jeux de données synthétiques produisent des listes d’arrêts reproductibles ; le générateur ne calcule ni tournées ni temps de trajet.
