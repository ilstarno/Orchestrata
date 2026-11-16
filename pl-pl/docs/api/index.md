Interfejs API w wersji startowej przyjmuje niewielki problem planistyczny w formacie JSON. Ta witryna jest statyczna: nie uruchamia modułu rozwiązującego, nie umożliwia logowania i nie przekazuje wywołań do Twojego komputera. Uruchom usługę lokalnie i używaj fikcyjnych danych podczas testowania.

## Uruchamianie i wywoływanie usługi

Uruchom poniższe polecenie w katalogu głównym repozytorium Orchestrata:

```sh
(cd backend && cargo run -p orchestrata-api)
```

W drugim terminalu, również w katalogu głównym repozytorium, uruchom:

```sh
curl -fsS http://127.0.0.1:3000/api/v1/health
curl -fsS -X POST http://127.0.0.1:3000/api/v1/solve \
  -H 'Content-Type: application/json' \
  --data-binary @backend/examples/assignment.json
```

| Metoda i ścieżka | Przeznaczenie |
| --- | --- |
| `GET /api/v1/health` | Sprawdź, czy lokalny proces HTTP odpowiada. |
| `POST /api/v1/solve` | Prześlij problem ze zmiennymi całkowitoliczbowymi do dokładnego rozwiązania w ramach limitu przeszukiwania. |
| `GET /api/v1/openapi.json` | Odczytaj schemat udostępniany przez uruchomioną wersję usługi. |

Każda zmienna ma nazwę i tablicę dozwolonych liczb całkowitych. Ograniczenia wskazują pozycje w tablicy zmiennych, numerowane od zera. Listę pól i znaczników znajdziesz w schemacie uruchomionej usługi. Jeśli klient domyślnie używa liczb zmiennoprzecinkowych, zadbaj o przesyłanie liczb całkowitych bez utraty precyzji.

## Interpretowanie wyników i błędów

Przykład zwraca obiekt JSON z polem `solution`. Wartości są ułożone w tej samej kolejności co zmienne. Oceny ograniczeń obowiązkowych i preferencji są podawane osobno. Lokalna wersja startowa sprawdza w trybie dokładnym maksymalnie 100 000 kombinacji. Błąd żądania, osiągnięcie limitu lub awaria serwera nie dowodzą, że model nie ma rozwiązania. Wersja ze zweryfikowanymi zasadami zwracania wyników dokładnych zwraca `{"solution":null}` tylko wtedy, gdy dokładne przeszukiwanie wykaże, że żaden plan nie spełnia wszystkich obowiązkowych ograniczeń.

## Receptury i katalog błędów w trakcie przeglądu

Poniższe ścieżki zaimplementowano w ramach prac podlegających przeglądowi lub lokalnej integracji. Zmiany w propozycji HTTP znajdziesz w [zmianach HTTP w przeglądzie](https://github.com/ilstarno/Orchestrata/pull/22). Zestaw ścieżek scalonych lokalnie może się różnić. Sprawdź uruchamianą rewizję. Nie zakładaj, że ścieżki są dostępne w domyślnie pobranej wersji repozytorium lub w usłudze hostingowej.

| Metoda i ścieżka | Przeznaczenie |
| --- | --- |
| `GET /api/v1/error-codes` | Odczytaj stabilne identyfikatory błędów i ich znaczenie. |
| `GET /api/v1/recipes` | Wyświetl listę dołączonych scenariuszy syntetycznych. |
| `GET /api/v1/recipes/{recipe_id}` | Odczytaj opis receptury i deklarowane pola. |
| `GET /api/v1/recipes/{recipe_id}/example` | Pobierz syntetyczny problem bez uruchamiania rozwiązania. |
| `POST /api/v1/recipes/{recipe_id}/run-example` | Rozwiąż dołączony przykład bez przesyłania treści żądania. Ta ścieżka jest częścią lokalnych prac podlegających przeglądowi i nie stanowi opublikowanego interfejsu. |

Osobna lokalna wersja MCP przeznaczona do przeglądu udostępnia `list_recipes`, `describe_recipe` i `get_recipe_example` przez standardowe wejście i wyjście. Pozwala wyłącznie wyświetlać listę syntetycznych przykładów i odczytywać je. Nie przyjmuje połączeń sieciowych i nie uruchamia rozwiązywania problemów. Lokalna wersja testowa nie oferuje hostowanego MCP, danych uwierzytelniających do zdalnego dostępu ani logowania.
