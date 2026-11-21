## Choisir votre point de départ

Lisez l’[exemple d’affectation (anglais)](/Orchestrata/examples/assignment/) sans installation. Pour un serveur existant, suivez la [première requête HTTP (anglais)](/Orchestrata/docs/solve-api/). Le client n’a pas besoin de Rust.

Rust est le langage du moteur et du serveur. Cargo compile les programmes Rust et gère leurs dépendances. Ces outils servent uniquement à construire depuis les sources. Le [guide du serveur (anglais)](/Orchestrata/docs/backend-setup/) explique l’installation et les vérifications de version.

Toutes les commandes sur les sources ci-dessous partent de la racine du checkout Orchestrata. Les exemples de shell utilisent Linux, macOS ou WSL.

Deux tâches se partagent les places 0 et 1. Chaque tâche peut occuper l’une ou l’autre, mais elles doivent occuper des places distinctes. La tâche A préfère la place 1.

Exécutez l’exemple en local. Ce site n’héberge pas l’API et n’accepte pas de fichiers. Pour HTTP, consultez le [guide de l’API locale](../api/).

## Exécuter l’exemple

Depuis la racine du dépôt Orchestrata, avec une version stable des outils Rust :

```sh
cargo run --manifest-path backend/Cargo.toml --locked -p orchestrata-cli -- backend/examples/assignment.json
```

Le résultat contient `values: [1, 0]`. Les scores des règles obligatoires et des préférences sont tous deux à zéro. Les valeurs suivent l’ordre de la saisie : la tâche A occupe la place 1 et la tâche B la place 0. La règle et la préférence sont respectées.

## Modifier un seul élément

Copiez l’`backend/examples/assignment.json` dans `backend/examples/preference-change.json`. Dans la contrainte `prefer_value` de la tâche A, remplacez la `value` de `1` par `0`. Relancez la même commande avec `backend/examples/preference-change.json`. L’affectation attendue devient `[0, 1]` ; la règle obligatoire ne change pas.

Dans une nouvelle copie, ne laissez que l’option `[0]` aux deux tâches et conservez la règle des places distinctes. Dans une version dont les résultats exacts ont été vérifiés, la CLI renvoie `null` : aucune affectation ne convient à ces données.

## Choisir ce que le modèle doit représenter

- Rendez une règle **obligatoire** si son non-respect rend le plan inutilisable.
- Utilisez une **préférence** si le plan reste utilisable même lorsqu’elle n’est pas satisfaite.
- Supprimez une option si elle ne peut jamais être choisie.

Commencez avec un exemple que vous pouvez vérifier à la main. Comparez les affectations obtenues à celles que vous attendez, y compris pour un cas sans solution. Si la limite de recherche est dépassée, le résultat reste inconnu. Cela ne démontre pas qu’aucun plan valide n’existe.
