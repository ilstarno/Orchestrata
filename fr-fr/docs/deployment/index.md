## Choisir votre point de départ

Lisez l’[exemple d’affectation (anglais)](/Orchestrata/examples/assignment/) sans installation. Pour un serveur existant, suivez la [première requête HTTP (anglais)](/Orchestrata/docs/solve-api/). Le client n’a pas besoin de Rust.

Rust est le langage du moteur et du serveur. Cargo compile les programmes Rust et gère leurs dépendances. Ces outils servent uniquement à construire depuis les sources. Le [guide du serveur (anglais)](/Orchestrata/docs/backend-setup/) explique l’installation et les vérifications de version.

Toutes les commandes sur les sources ci-dessous partent de la racine du checkout Orchestrata. Les exemples de shell utilisent Linux, macOS ou WSL.

## Démarrer un serveur local

Le [guide du serveur (anglais)](/Orchestrata/docs/backend-setup/) décrit l’installation et le démarrage. Le site et le serveur sont séparés. La première requête ne nécessite ni interface navigateur ni base de données.

```sh
ORCHESTRATA_HTTP_BIND=127.0.0.1:3000 \
  cargo run --manifest-path backend/Cargo.toml --locked -p orchestrata-api
```

## Utiliser une bibliothèque native

Choisissez une liaison dans le [guide SDK (anglais)](/Orchestrata/docs/sdk/). Il utilise les sources publiques du moteur, la bibliothèque native correspondante et une installation locale du paquet.

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
