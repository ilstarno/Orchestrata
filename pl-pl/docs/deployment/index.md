Uruchom API i interfejs webowy na swoim komputerze. Używaj danych przykładowych i nie udostępniaj tej wersji publicznie w internecie. Witryna nie uruchamia API.

## Lokalne API i interfejs webowy

Użyj stabilnej wersji Rusta z Cargo oraz narzędzia curl. Do interfejsu webowego zainstaluj też Node.js 22 i npm. Otwórz [repozytorium Orchestrata](https://github.com/ilstarno/Orchestrata). Pobierz źródła i sprawdź, czy obszar roboczy Cargo zawiera `orchestrata-api`. Sprawdź ścieżki opcjonalne w dokumencie OpenAPI działającego serwera.

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

Aby użyć interfejsu webowego, pozostaw uruchomione API i otwórz terminal w [osobnym repozytorium frontendu](https://github.com/ilstarno/orchestrata-frontend), jeśli masz do niego dostęp. Sprawdź `frontend/package.json` oraz zgodność schematu API z działającym backendem.

```sh
(cd frontend && npm install && npm run dev)
```

Otwórz lokalny adres wyświetlony przez polecenie. Serwer deweloperski przekazuje żądania z `/api` do `127.0.0.1:3000`. Jeśli wybierzesz inny lokalny port w `ORCHESTRATA_HTTP_BIND`, zmień zarówno adresy curl, jak i adres docelowy deweloperskiego proxy.

[Przewodnik po lokalnym API](../api/) opisuje żądania i odpowiedzi. W razie problemów z uruchomieniem lub proxy skorzystaj z [instrukcji konfiguracji i rozwiązywania problemów z backendem (po angielsku)](../../../docs/backend-setup/).

## Ocena samodzielnego silnika i zestawów SDK

[Repozytorium samodzielnego silnika](https://github.com/ilstarno/orchestrata-engine) zawiera silnik w Rust, bibliotekę natywną i powiązania dla C#, Javy, Pythona, Node.js, PHP i Go. Skompiluj SDK z pasujących źródeł. W lokalnej kopii repozytorium silnika, przy użyciu stabilnego Rusta, uruchom:

```sh
(
  if [ -f Cargo.toml ] && [ -f crates/ffi/include/orchestrata.h ]; then
      engine_manifest=Cargo.toml
  elif [ -f backend/Cargo.toml ] && [ -f backend/crates/ffi/include/orchestrata.h ]; then
      engine_manifest=backend/Cargo.toml
  else
      printf '%s\n' 'Run these commands from the standalone engine repository.' >&2
      exit 1
  fi
  cargo build --manifest-path "$engine_manifest" --release --locked -p orchestrata-ffi
)
```

Uruchom polecenie w repozytorium samodzielnego silnika, a nie w repozytorium API opisanym powyżej. Polecenie samo znajdzie właściwy plik Cargo.

Sprawdź w pliku README zestawu SDK wymagania dotyczące ładowania biblioteki i środowiska uruchomieniowego danego języka. Używaj biblioteki natywnej z tej samej kopii repozytorium i powiązania zgodnego z ABI. Możliwość zbudowania biblioteki natywnej na jednej platformie nie dowodzi dostępności pakietów dla innych platform.

## Lista kontrolna przed testem na klastrze

Oceniaj pracę na klastrze osobno. Lokalne kontrole nie dowodzą gotowości obrazu, konfiguracji zarządzanej chmury ani dostępu publicznego. Używaj tylko danych przykładowych i pozostaw dostęp publiczny wyłączony.

1. Zapisz skrót testowanego obrazu. Użyj osobnej, nieprodukcyjnej przestrzeni nazw.
2. Ustaw wymagania i limity CPU i pamięci. Z uprawnionego klienta w sieci testowej sprawdź status, listę przepisów i przykład przydziału.
3. Sprawdź, czy dostęp spoza wyznaczonej sieci jest blokowany. Sprawdź dzienniki bez zapisywania danych uwierzytelniających lub danych klientów.
4. Zainstaluj badaną wersję, wróć do poprzedniego obrazu i powtórz kontrole. Jeśli dodajesz trwałe przechowywanie, sprawdź osobno tworzenie kopii zapasowej i odtwarzanie danych.

## Kontenery, Kubernetes i Terraform

Zbuduj obraz API z pliku Dockerfile i zainstaluj go za pomocą Helm w istniejącym prywatnym klastrze Kubernetes. Instrukcja Terraform dotyczy istniejących klastrów Amazon EKS, Azure AKS i Google GKE. Nie tworzy infrastruktury w chmurze.

Plik Dockerfile, pakiet Helm i wskazówki Terraform znajdziesz w `deploy/` i `deploy/terraform/`, jeśli te katalogi są dostępne.

Pakiet przeszedł lokalne kontrole Helm. Nie przetestowano kontenera ani wdrożenia w klastrach zarządzanych.
