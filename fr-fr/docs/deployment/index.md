Exécutez d’abord le service et l’interface sur votre poste de travail. Le site du projet Orchestrata est statique : l’ouvrir ne démarre aucune API. Ce guide est destiné à une évaluation locale avec des données fictives, non à un accès public ou à un usage en production.

## API locale et interface web

Utilisez une chaîne d’outils Rust stable pour l’API, ainsi que Node.js 22 et npm pour l’interface web. À la racine du dépôt Orchestrata, exécutez la commande suivante :

```sh
(cd backend && cargo run -p orchestrata-api)
```

Dans un deuxième terminal, exécutez :

```sh
(cd frontend && npm install && npm run dev)
```

L’API utilise `127.0.0.1:3000`. Le [guide de l’API locale](../api/) détaille les requêtes et les résultats. Ouvrez l’adresse de l’interface affichée par la commande du frontend. Le serveur web de développement transmet au service local les requêtes visant `/api/`. Vérifiez que le service répond et exécutez l’exemple d’affectation fourni avant d’en évaluer un plus volumineux :

```sh
curl -fsS http://127.0.0.1:3000/api/v1/health
curl -fsS -X POST http://127.0.0.1:3000/api/v1/solve \
  -H 'Content-Type: application/json' \
  --data-binary @backend/examples/assignment.json
```

Limitez cette version à l’interface de bouclage. Un contrôle d’état réussi confirme uniquement que le processus répond, pas que l’authentification fonctionne, que le service dispose d’une capacité donnée ou qu’un stockage persistant des tâches est disponible.

## Évaluer le moteur autonome et les SDK

La [proposition d’intégration du moteur autonome](https://github.com/ilstarno/orchestrata-engine/pull/1) contient le moteur Rust, la bibliothèque native et les liaisons pour C#, Java, Python, Node.js, PHP et Go. Cette branche n’est ni une version empaquetée des SDK, ni une version identifiée par un tag. Depuis la copie locale du dépôt du moteur, avec Rust stable, exécutez :

```sh
cargo build --manifest-path backend/Cargo.toml --release --locked -p orchestrata-ffi
```

Consultez le README du SDK pour connaître les exigences de chargement et d’environnement d’exécution propres à votre langage. Utilisez la bibliothèque native issue de la même révision du dépôt et une liaison compatible avec l’ABI. La compilation de la bibliothèque native sur une plateforme ne prouve pas que le conditionnement en paquets est pris en charge sur d’autres plateformes.

## Avant d’utiliser un cluster

La génération des manifestes de déploiement expérimentaux et leur conformité au schéma ont été vérifiées lors d’une revue distincte. Ce guide local ne nécessite ni ne laisse entendre l’existence d’une image de production, d’une certification pour un cloud managé, d’un point d’accès public ou d’une configuration d’authentification. Pour une évaluation sur cluster, il faut une révision examinée à cette fin, un accès entrant protégé, des contrôles d’accès testés, des limites de ressources explicites et des preuves d’installation et de retour arrière. N’exposez pas une API en préversion à Internet et ne saisissez pas de données client réelles pour vérifier son fonctionnement.
