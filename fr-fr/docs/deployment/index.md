Exécutez l’API et l’interface web sur votre poste. Utilisez les données d’exemple et gardez cette version hors de l’Internet public. Le site ne démarre pas l’API.

## API locale et interface web

Utilisez une version stable de Rust avec Cargo et curl. Pour l’interface web, installez aussi Node.js 22 et npm. Ouvrez le [dépôt Orchestrata](https://github.com/ilstarno/Orchestrata). Récupérez les sources et vérifiez que l’espace de travail Cargo contient `orchestrata-api`. Vérifiez les routes facultatives dans le document OpenAPI du service.

À la racine du dépôt, exécutez les tests de l’espace de travail avec les dépendances verrouillées, puis démarrez l’API sur l’interface de bouclage :

```sh
cargo test --manifest-path backend/Cargo.toml --locked --workspace
ORCHESTRATA_HTTP_BIND=127.0.0.1:3000 cargo run --manifest-path backend/Cargo.toml --locked -p orchestrata-api
```

Laissez l’API tourner. Dans un deuxième terminal, depuis la racine du dépôt, vérifiez que le service répond, demandez les trois premières lignes du catalogue de recettes et exécutez l’exemple d’affectation fourni :

```sh
curl -fsS http://127.0.0.1:3000/api/v1/health
curl -fsS 'http://127.0.0.1:3000/api/v1/recipes/grid?page_size=3'
curl -fsS -X POST http://127.0.0.1:3000/api/v1/solve \
  -H 'Content-Type: application/json' \
  --data-binary @backend/examples/assignment.json
```

Le contrôle d’état confirme que le processus répond. La liste renvoie les métadonnées des recettes et ne lance aucune tâche. Le résultat attendu est `[1,0]` dans l’ordre des entrées. Ces contrôles ne vérifient ni la connexion des utilisateurs, ni le stockage persistant des tâches, ni la capacité du service.

Pour utiliser l’interface web, laissez l’API tourner et ouvrez un terminal dans le [dépôt distinct du frontend](https://github.com/ilstarno/orchestrata-frontend) si vous y avez accès. Vérifiez `frontend/package.json` et comparez le schéma API avec celui du backend.

```sh
(cd frontend && npm install && npm run dev)
```

Ouvrez l’adresse locale affichée par la commande. Le serveur de développement redirige `/api` vers `127.0.0.1:3000`. Si vous choisissez un autre port de bouclage dans `ORCHESTRATA_HTTP_BIND`, modifiez à la fois les adresses curl et la destination du proxy de développement.

Le [guide de l’API locale](../api/) décrit les requêtes et les réponses. En cas de problème de démarrage ou de proxy, consultez le [guide de configuration et de dépannage du backend (en anglais)](../../../docs/backend-setup/).

## Évaluer le moteur autonome et les SDK

Le [dépôt du moteur autonome](https://github.com/ilstarno/orchestrata-engine) contient le moteur Rust, la bibliothèque native et les liaisons pour C#, Java, Python, Node.js, PHP et Go. Compilez le SDK à partir des sources correspondantes. Depuis la copie locale du dépôt du moteur, avec Rust stable, exécutez :

```sh
cargo build --manifest-path backend/Cargo.toml --release --locked -p orchestrata-ffi
```

Exécutez cette commande dans la copie du moteur, pas dans celle de l’API publique utilisée ci-dessus.

Consultez le README du SDK pour connaître les exigences de chargement et d’environnement d’exécution propres à votre langage. Utilisez la bibliothèque native issue de la même révision du dépôt et une liaison compatible avec l’ABI. La compilation de la bibliothèque native sur une plateforme ne prouve pas que le conditionnement en paquets est pris en charge sur d’autres plateformes.

## Liste de vérification avant un essai en cluster

Évaluez le cluster séparément. Les contrôles locaux ne démontrent pas qu’une image, une configuration de cloud managé ou un accès public sont prêts. Utilisez uniquement des données d’exemple et laissez l’accès public désactivé.

1. Notez le digest de l’image testée. Utilisez un espace de noms dédié, hors production.
2. Fixez les demandes et limites de CPU et de mémoire. Depuis un client autorisé sur le réseau de test, vérifiez l’état, la liste des recettes et l’affectation.
3. Vérifiez que les accès en dehors du périmètre autorisé sont bloqués. Examinez les journaux sans y consigner d’identifiants ni de données clients.
4. Installez la version à vérifier, revenez à l’image précédente et répétez les contrôles. Si vous ajoutez un stockage persistant, testez séparément la sauvegarde et la restauration.

## Conteneurs, Kubernetes et Terraform

Construisez l’image de l’API à partir du Dockerfile, puis installez-la avec Helm dans un cluster Kubernetes privé existant. Le guide Terraform concerne les clusters Amazon EKS, Azure AKS et Google GKE existants. Il ne crée pas d’infrastructure cloud.

Pour le Dockerfile, Helm et Terraform, consultez `deploy/` et `deploy/terraform/` si ces dossiers sont présents.

Le chart a passé les contrôles Helm locaux. Le conteneur et le déploiement sur des clusters gérés n’ont pas été testés.
