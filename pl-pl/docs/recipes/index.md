Szablon planowania opisuje mały scenariusz i używane w nim pola. Przykłady do uruchomienia zawierają fikcyjne dane. Przeczytaj opis, uruchom model JSON i porównaj przydziały z oczekiwanym wynikiem. Przykłady sprawdzają konkretne decyzje i nie zastępują systemu planowania ani dyspozytorni.

| Przykład | Co można sprawdzić | Czego przykład nie pokazuje |
| --- | --- | --- |
| Przydział techników terenowych | Przypisanie nakładających się wizyt różnym technikom z odpowiednimi kwalifikacjami. | Dyspozycji na żywo, tras drogowych ani całodziennego przydziału pracy. |
| Nieobecność w dwóch sklepach | Porównanie planu wyjściowego z planem po zgłoszonej nieobecności, przy zachowaniu stałej zmiany. | Prognozowania, integracji z kalendarzem ani zgodności z przepisami prawa pracy. |
| Przydział odbiorów i dostaw | Dobór pojazdów o wystarczającej ładowności i dostępnych w wymaganym czasie. | Wyznaczania tras ani łącznego obciążenia pojazdu przy wielu przystankach. |
| Przydział bramek na lotnisku | Przypisanie jednoczesnych lotów do odpowiednich wolnych bramek. | Aktualnych przylotów, systemów bramek ani pełnego planowania lotniska. |
| Obsada centrum obsługi telefonicznej | Obsada kilku kolejek naraz według wymaganych języków i umiejętności. | Prognoz liczby połączeń, poziomu obsługi ani zarządzania obsadą na żywo. |

## Uruchom przykład z katalogu

Pobierz gałąź [importu szablonów do przeglądu](https://github.com/ilstarno/orchestrata-recipes/pull/1) i upewnij się, że skompilowane polecenie `orchestrata-cli` jest dostępne na ścieżce wyszukiwania. W katalogu głównym repozytorium szablonów wykonaj:

```sh
orchestrata-cli recipes/examples/fleet-pickup-delivery/problem.json
```

Zapisany wynik to `[1, 2, 0]`, bez punktów karnych za zasady i preferencje. Dane wejściowe są w pliku `inputs.json`, a wynik do porównania w `expected.json`. XML opisuje scenariusz; CLI przyjmuje znormalizowany model JSON. Osobna gałąź importu zawiera pliki źródłowe, a nie opublikowane wydanie katalogu.

Skorzystaj z [instrukcji przydziału](../evaluate/), aby uruchomić mniejszy przykład bez katalogu. Wybierz jedną decyzję do zamodelowania, spisz dozwolone opcje i uwzględnij tylko zasady, które możesz sprawdzić. Nie używaj prawdziwych danych klientów ani pracowników w wersji do lokalnych prób. Dane syntetyczne floty pozwalają odtworzyć listę przystanków; generator nie wyznacza tras ani czasu przejazdu.
