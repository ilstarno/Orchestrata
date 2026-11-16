## Couvrir quatre postes malgré une absence

À 14 h, chacun des deux magasins a besoin de deux personnes. Ben est absent. Comparez le plan initial à un second calcul où Ben n’est plus disponible pour les deux postes qu’il pouvait occuper. Les postes, les règles, leur pondération et les autres possibilités ne changent pas.

### Informations à fournir

Pour chaque poste, indiquez le magasin, le rôle et les personnes disponibles et qualifiées. Ajoutez les affectations imposées ainsi que les préférences auxquelles le plan peut déroger. Conservez les mêmes indices de personnel dans les deux calculs afin de pouvoir comparer les résultats.

| Champ de la recette | Type | Exemple de valeur |
| --- | --- | --- |
| `shift-names` | Liste de chaînes | Nord responsable des clés, Nord rayon, Sud caisse, Sud réserve |
| `worker-names` | Liste de chaînes | Ana, Ben, Clara, Diego, Elena ; indices de 0 à 4 |
| `eligible-pairs` | Liste d’entiers | Paires d’indices poste et personne ; retirer 1,1 et 2,1 en cas d’absence de Ben |
| `pinned-pairs` | Liste facultative d’entiers | 0,0 affecte Ana au poste de responsable des clés du magasin Nord |
| `preferred-assignments` | Liste facultative d’entiers | Triplets indice de poste, indice de personne, poids : 1,1,5 ; 2,2,3 ; 3,3,2 |

Dans les données d’entrée du solveur, chaque poste est une variable. Une affectation imposée ne laisse qu’une seule personne possible : `north_keyholder` a pour domaine `domain: [0]`. La règle obligatoire `all_different` reste identique dans les deux calculs. Les trois règles `prefer_value` conservent leur poids d’origine, même lorsqu’une préférence ne peut plus être satisfaite.

### Comparer les plans

| Poste | Personnes éligibles avant l’absence | Plan initial | Après l’absence |
| --- | --- | --- | --- |
| Nord responsable des clés | Ana | Ana | Ana |
| Nord rayon | Ben, Clara | Ben | Clara |
| Sud caisse | Ben, Clara, Elena | Clara | Elena |
| Sud réserve | Diego, Elena | Diego | Diego |

Les deux plans couvrent les quatre postes sans affecter une même personne à deux postes simultanés. Le calcul initial produit `[0, 1, 2, 3]`, des scores de 0/0 pour les règles obligatoires et les préférences, et examine 12 combinaisons. Après l’absence, il produit `[0, 2, 4, 3]`, des scores de 0/−8 et examine 4 combinaisons. Le score de −8 correspond à deux préférences non satisfaites : 5 points pour Ben en rayon au Nord et 3 pour Clara en caisse au Sud. Aucune règle obligatoire n’est enfreinte.

### Exécuter les deux variantes

Enregistrez les [données d’entrée initiales](baseline-problem.json) et les [données d’entrée ajustées](adjusted-problem.json) :

```sh
orchestrata-cli baseline-problem.json
orchestrata-cli adjusted-problem.json
```

Comparez le [résultat initial](baseline-expected.json) et le [résultat ajusté](adjusted-expected.json). Vous pouvez aussi exécuter les deux calculs via l’[API locale](../../docs/api/).

### Limites de l’exemple

Les quatre postes sont déjà définis et leurs horaires se chevauchent. L’exemple ne couvre ni la prévision de la demande, ni la synchronisation d’agendas, ni la paie, ni les heures supplémentaires ou les temps de repos. Il ne vérifie pas les règles du droit du travail et ne construit pas de planning sur plusieurs semaines.
