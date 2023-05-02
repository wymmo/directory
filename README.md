# Annuaire Wymmo.com / Les Pros de l'immo

Vous pouvez contribuer à l'annuaire [wymmo.com](https://www.wymmo.com) en nous soumettant une Pull Request.

## Données

Les fichiers contenant les données affichés sur l'annuaire Wymmo.com sont contenus dans le répertoire `directory`.
Les autres fichiers sont simplement destinés à la vérification des données avant intégration sur le site Wymmo.com.

### formats de fichiers

2 format de fichiers sont présents dans ce dépot

#### Les fiches `catégories` dans le dossier `directory/tags`
```yaml
key: portail                    # la clé doit être obligatoirement identique au nom du fichier 
title: Portails immobiliers     # titre de la catégorie
description:                    # paragraphes de texte libre permettant de documenter la catégorie
  - |
    Un portail immobilier est un site regroupant de nombreuses ressources en relation avec le monde de l'immobilier.
    Par exemple on peut trouver des annonces immobilières, des simulateurs, des indications sur les tendances de prix du marché immobilier.

```

#### les fiches `produit / entreprise` à la racine du dossier `directory`

```yaml
# wymmo.yaml
key: wymmo                                  # la clé doit être obligatoirement identique au nom du fichier 
name: Wymmo.com                             # nom court de votre produit / entreprise ...
title: Wymmo, où voudriez-vous habiter ?    # nom complet / avec slogan  
tags: ["portail"]                           # tag/catégories d'appartenance de votre site/produit
created_in: 2021                            # année de création
description:                                # paragraphes de texte libre permettant de documenter votre produit
  - |
    Où voudriez-vous habiter ?
url: https://www.wymmo.com                  # url de votre site
backlink: https://www.wymmo.com/thanks      # url de votre site contenant un lien vers Wymmo.com
```


Le backlink n'est pas obligatoire, les fiches sans backlink, ou avec backlink dont la vérification échouerait, seront acceptées et affichées sans problème, mais le lien vers votre site sera en `nofollow`.

## Licence

Les codes sources et fichiers de données `.yaml` de ce présent dépot sont en licence  [`CC BY-SA`](https://creativecommons.org/licenses/by-sa/4.0/).

Vous êtes libres de
- copier et restribuer ce matériel dans tout medium et sous tout format.
- transformer, adapter, construire à partir de ce matériel pour tout but, y compris commercial.

Vous avez les obligations de
- attribuer le crédit nécessaire aux auteurs originaux du présent matériel, en mentionnant les modifications apportées
- le cas échéant partager aux mêmes conditions le matériel modifié

[Texte de la licence CC BY-SA en français](https://creativecommons.org/licenses/by/4.0/legalcode.fr)


## Tester les données

Pour vérifier la congruance des données, vous pouvez installer le langage [Rust](https://www.rust-lang.org/tools/install)
Puis vous pouvez lancer la commande `cargo t` à la racine de ce dépôt.
