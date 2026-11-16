Orchestrata pozwala sprawdzić, czy plan spełnia zasady i uwzględnia preferencje. W tym przykładzie są dwa zadania i dwa miejsca, oznaczone numerami 0 i 1. Każde zadanie może trafić na dowolne miejsce, ale oba nie mogą zająć tego samego. Dla zadania A preferowane jest miejsce 1.

Przykład używa fikcyjnych danych i działa lokalnie. Ta witryna nie uruchamia API ani nie przyjmuje plików. Aby uruchomić przykład przez HTTP, skorzystaj z [instrukcji lokalnego API](../api/).

## Uruchom przykład

W katalogu głównym repozytorium Orchestrata, z aktualną stabilną wersją narzędzi Rust:

```sh
(cd backend && cargo run -p orchestrata-cli -- examples/assignment.json)
```

Wynik zawiera `values: [1, 0]`. Ocena przestrzegania obowiązkowych zasad i ocena preferencji wynoszą zero. Wartości są w takiej samej kolejności jak zadania w danych wejściowych: A trafia na miejsce 1, a B na miejsce 0. Zasada i preferencja zostały spełnione.

## Zmień jedną rzecz

Skopiuj [przykład przydziału](https://github.com/ilstarno/Orchestrata/blob/9dab0545beb8cb9ded93cc632e41f0b6bc718e77/backend/examples/assignment.json) do `backend/examples/preference-change.json`. Dla zadania A w warunku `prefer_value` zmień pole `value` z `1` na `0`. Uruchom to samo polecenie z plikiem `examples/preference-change.json`. Oczekiwany przydział to teraz `[0, 1]`. Obowiązkowa zasada pozostaje bez zmian.

W nowej kopii daj obu zadaniom tylko opcję `[0]`. Zachowaj zasadę różnych miejsc. W wersji ze zweryfikowanym wyszukiwaniem dokładnym CLI zwróci `null`: dla tych danych nie istnieje poprawny przydział.

## Ustal, co należy uwzględnić w modelu

- Zasada jest **obowiązkowa**, gdy jej złamanie oznacza, że plan nie nadaje się do użycia.
- Użyj **preferencji**, gdy plan nadaje się do użycia nawet wtedy, gdy nie da się jej uwzględnić.
- Usuń opcję, której nigdy nie można wybrać.

Zacznij od przykładu, który możesz sprawdzić ręcznie. Porównaj otrzymane przydziały z oczekiwanymi, także dla przypadku bez rozwiązania. Po przekroczeniu limitu wyszukiwania wynik pozostaje nieznany. To nie to samo co ustalenie, że poprawny plan nie istnieje.
