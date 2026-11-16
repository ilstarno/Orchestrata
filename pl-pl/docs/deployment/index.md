Najpierw uruchom usługę i interfejs użytkownika na swoim komputerze. Witryna projektu Orchestrata jest statyczna: jej otwarcie nie uruchamia API. Wersja zapoznawcza służy do lokalnych testów z fikcyjnymi danymi, nie do publicznego dostępu ani użytku produkcyjnego.

## Lokalny interfejs API i interfejs webowy

Użyj stabilnego zestawu narzędzi Rust na potrzeby API oraz Node.js 22 i npm na potrzeby interfejsu webowego. W katalogu głównym repozytorium Orchestrata uruchom:

```sh
(cd backend && cargo run -p orchestrata-api)
```

W drugim terminalu uruchom:

```sh
(cd frontend && npm install && npm run dev)
```

API używa adresu `127.0.0.1:3000`. Szczegóły dotyczące żądań i wyników znajdziesz w [przewodniku po lokalnym API](../api/). Otwórz adres interfejsu wyświetlony przez polecenie uruchamiające frontend. Serwer deweloperski przekazuje żądania kierowane do `/api/` do lokalnej usługi. Sprawdź, czy usługa odpowiada, i uruchom dołączony przykład przydziału, zanim wypróbujesz większy przykład:

```sh
curl -fsS http://127.0.0.1:3000/api/v1/health
curl -fsS -X POST http://127.0.0.1:3000/api/v1/solve \
  -H 'Content-Type: application/json' \
  --data-binary @backend/examples/assignment.json
```

Ogranicz tę wersję do interfejsu pętli zwrotnej. Pozytywny wynik sprawdzenia dostępności potwierdza jedynie, że proces odpowiada. Nie potwierdza działania uwierzytelniania, wydajności usługi ani dostępności trwałego magazynu zadań.

## Ocena samodzielnego silnika i zestawów SDK

[Repozytorium samodzielnego silnika](https://github.com/ilstarno/orchestrata-engine/tree/main) zawiera silnik w Rust, bibliotekę natywną i powiązania dla C#, Javy, Pythona, Node.js, PHP i Go. Skompiluj SDK z pasujących źródeł. W lokalnej kopii repozytorium silnika, przy użyciu stabilnego Rusta, uruchom:

```sh
cargo build --manifest-path backend/Cargo.toml --release --locked -p orchestrata-ffi
```

Sprawdź w pliku README zestawu SDK wymagania dotyczące ładowania biblioteki i środowiska uruchomieniowego danego języka. Używaj biblioteki natywnej z tej samej kopii repozytorium i powiązania zgodnego z ABI. Możliwość zbudowania biblioteki natywnej na jednej platformie nie dowodzi dostępności pakietów dla innych platform.

## Przed użyciem klastra

Podczas osobnego przeglądu sprawdzono generowanie eksperymentalnych manifestów wdrożeniowych i ich zgodność ze schematem. Ten lokalny przewodnik nie wymaga obrazu produkcyjnego, certyfikacji zarządzanej usługi chmurowej, publicznego punktu końcowego ani konfiguracji logowania; nie potwierdza też ich dostępności. Testy na klastrze wymagają osobno zweryfikowanej rewizji, zabezpieczonego ruchu przychodzącego, przetestowanej kontroli dostępu, jawnych limitów zasobów i potwierdzenia, że instalacja i wycofanie działają. Nie udostępniaj wersji zapoznawczej API w internecie i nie używaj prawdziwych danych klientów do sprawdzania jej działania.
