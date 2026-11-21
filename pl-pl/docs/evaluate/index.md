## Wybierz punkt startowy

Przeczytaj [przykład przydziału (po angielsku)](/Orchestrata/examples/assignment/) bez instalacji. Dla istniejącego serwera użyj [pierwszego żądania HTTP (po angielsku)](/Orchestrata/docs/solve-api/). Klient nie wymaga Rusta.

Rust to język programowania silnika i serwera. Cargo buduje programy Rust i zarządza zależnościami. Narzędzia te są potrzebne tylko do budowania ze źródeł. [Instrukcja serwera (po angielsku)](/Orchestrata/docs/backend-setup/) opisuje instalację i sprawdzanie wersji.

Wszystkie poniższe polecenia dotyczące źródeł uruchom w głównym katalogu checkoutu Orchestrata. Przykłady powłoki używają Linuxa, macOS lub WSL.

Dwa zadania zajmą miejsca 0 i 1. Każde może trafić na dowolne miejsce, ale muszą dostać różne miejsca. Zadanie A preferuje miejsce 1.

Uruchom przykład lokalnie. Witryna nie uruchamia API ani nie przyjmuje plików. W przypadku HTTP zobacz [instrukcję lokalnego API](../api/).

## Uruchom przykład

W katalogu głównym repozytorium Orchestrata, z aktualną stabilną wersją narzędzi Rust:

```sh
cargo run --manifest-path backend/Cargo.toml --locked -p orchestrata-cli -- backend/examples/assignment.json
```

Wynik zawiera `values: [1, 0]`. Ocena przestrzegania obowiązkowych zasad i ocena preferencji wynoszą zero. Wartości są w takiej samej kolejności jak zadania w danych wejściowych: A trafia na miejsce 1, a B na miejsce 0. Zasada i preferencja zostały spełnione.

## Zmień jedną rzecz

Skopiuj `backend/examples/assignment.json` do `backend/examples/preference-change.json`. Dla zadania A w warunku `prefer_value` zmień pole `value` z `1` na `0`. Uruchom to samo polecenie z plikiem `backend/examples/preference-change.json`. Oczekiwany przydział to teraz `[0, 1]`. Obowiązkowa zasada pozostaje bez zmian.

W nowej kopii daj obu zadaniom tylko opcję `[0]`. Zachowaj zasadę różnych miejsc. W wersji ze zweryfikowanym wyszukiwaniem dokładnym CLI zwróci `null`: dla tych danych nie istnieje poprawny przydział.

## Ustal, co należy uwzględnić w modelu

- Zasada jest **obowiązkowa**, gdy jej złamanie oznacza, że plan nie nadaje się do użycia.
- Użyj **preferencji**, gdy plan nadaje się do użycia nawet wtedy, gdy nie da się jej uwzględnić.
- Usuń opcję, której nigdy nie można wybrać.

Zacznij od przykładu, który możesz sprawdzić ręcznie. Porównaj otrzymane przydziały z oczekiwanymi, także dla przypadku bez rozwiązania. Po przekroczeniu limitu wyszukiwania wynik pozostaje nieznany. To nie to samo co ustalenie, że poprawny plan nie istnieje.
