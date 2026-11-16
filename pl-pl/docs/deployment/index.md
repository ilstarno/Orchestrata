Najpierw uruchom usługę i interfejs webowy na swoim komputerze. Witryna projektu Orchestrata jest statyczna: jej otwarcie nie uruchamia API. Używaj danych przykładowych i nie udostępniaj tej wersji przez internet.

## Lokalne API i interfejs webowy

Użyj stabilnej wersji Rusta z Cargo oraz narzędzi Git i curl. Do interfejsu webowego zainstaluj też Node.js 22 i npm. Poniższe polecenia korzystają z określonej, sprawdzonej rewizji publicznego projektu startowego:

```sh
git clone https://github.com/ilstarno/Orchestrata.git
cd Orchestrata
git checkout --detach b8d1271e3faddfffaf220698e381197bcae1752f
```

W katalogu głównym repozytorium uruchom testy obszaru roboczego z wersjami zależności zapisanymi w pliku blokady. Następnie uruchom API na interfejsie pętli zwrotnej:

```sh
cargo test --manifest-path backend/Cargo.toml --locked --workspace
ORCHESTRATA_HTTP_BIND=127.0.0.1:3000 cargo run --manifest-path backend/Cargo.toml --locked -p orchestrata-api
```

Pozostaw API uruchomione. W drugim terminalu, w katalogu głównym, sprawdź status, pobierz trzy pierwsze pozycje z listy przepisów i uruchom dołączony przykład przydziału:

```sh
curl -fsS http://127.0.0.1:3000/api/v1/health
curl -fsS 'http://127.0.0.1:3000/api/v1/recipes/grid?page_size=3'
curl -fsS -X POST http://127.0.0.1:3000/api/v1/solve \
  -H 'Content-Type: application/json' \
  --data-binary @backend/examples/assignment.json
```

Pozytywny wynik sprawdzenia statusu potwierdza, że proces odpowiada. Lista zwraca metadane przepisów i nie zleca żadnych zadań. Oczekiwany przydział to `[1,0]` w kolejności danych wejściowych. Te kontrole nie potwierdzają działania logowania, trwałego zapisu zadań ani wydajności usługi.

Pozostaw API uruchomione i skorzystaj z kolejnego terminala, aby uruchomić interfejs webowy z tej samej kopii repozytorium:

```sh
(cd frontend && npm install && npm run dev)
```

Otwórz lokalny adres wyświetlony przez polecenie. Serwer deweloperski przekazuje żądania z `/api` do `127.0.0.1:3000`. Jeśli wybierzesz inny lokalny port w `ORCHESTRATA_HTTP_BIND`, zmień zarówno adresy curl, jak i adres docelowy deweloperskiego proxy.

[Przewodnik po lokalnym API](../api/) opisuje żądania i odpowiedzi. W razie problemów z uruchomieniem lub proxy skorzystaj z [instrukcji konfiguracji i rozwiązywania problemów z backendem (po angielsku)](../../../docs/backend-setup/).

## Ocena samodzielnego silnika i zestawów SDK

[Repozytorium samodzielnego silnika](https://github.com/ilstarno/orchestrata-engine/tree/d7b920e32e5402d96b59b424983199f82c7e8699) zawiera silnik w Rust, bibliotekę natywną i powiązania dla C#, Javy, Pythona, Node.js, PHP i Go. Skompiluj SDK z pasujących źródeł. W lokalnej kopii repozytorium silnika, przy użyciu stabilnego Rusta, uruchom:

```sh
cargo build --manifest-path backend/Cargo.toml --release --locked -p orchestrata-ffi
```

Uruchom to polecenie w kopii repozytorium silnika, nie w użytej powyżej kopii publicznego API.

Sprawdź w pliku README zestawu SDK wymagania dotyczące ładowania biblioteki i środowiska uruchomieniowego danego języka. Używaj biblioteki natywnej z tej samej kopii repozytorium i powiązania zgodnego z ABI. Możliwość zbudowania biblioteki natywnej na jednej platformie nie dowodzi dostępności pakietów dla innych platform.

## Lista kontrolna przed testem na klastrze

Oceniaj pracę na klastrze osobno. Lokalne kontrole nie dowodzą gotowości obrazu, konfiguracji zarządzanej chmury ani dostępu publicznego. Używaj tylko danych przykładowych i pozostaw dostęp publiczny wyłączony.

1. Zapisz sprawdzoną rewizję źródeł i digest zbudowanego z niej obrazu. Użyj osobnej, nieprodukcyjnej przestrzeni nazw.
2. Ustaw wymagania i limity CPU i pamięci. Z uprawnionego klienta w sieci testowej sprawdź status, listę przepisów i przykład przydziału.
3. Sprawdź, czy dostęp spoza wyznaczonej sieci jest blokowany. Sprawdź dzienniki bez zapisywania danych uwierzytelniających lub danych klientów.
4. Zainstaluj badaną wersję, wróć do poprzedniego obrazu i powtórz kontrole. Jeśli dodajesz trwałe przechowywanie, sprawdź osobno tworzenie kopii zapasowej i odtwarzanie danych.

## Kontenery, Kubernetes i Terraform

Zbuduj obraz API z pliku Dockerfile i zainstaluj go za pomocą Helm w istniejącym prywatnym klastrze Kubernetes. Instrukcja Terraform dotyczy istniejących klastrów Amazon EKS, Azure AKS i Google GKE. Nie tworzy infrastruktury w chmurze.

- [Instrukcja obsługi kontenerów i Helm (po angielsku)](https://github.com/ilstarno/Orchestrata/tree/2c921cedad7e20d3c1a892b35024d3465e336d75/deploy)
- [Instrukcja Terraform (po angielsku)](https://github.com/ilstarno/Orchestrata/tree/2c921cedad7e20d3c1a892b35024d3465e336d75/deploy/terraform)

Pakiet przeszedł lokalne kontrole Helm. Nie przetestowano kontenera ani wdrożenia w klastrach zarządzanych.
