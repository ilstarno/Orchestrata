## Wybierz punkt startowy

Przeczytaj [przykład przydziału (po angielsku)](/Orchestrata/examples/assignment/) bez instalacji. Dla istniejącego serwera użyj [pierwszego żądania HTTP (po angielsku)](/Orchestrata/docs/solve-api/). Klient nie wymaga Rusta.

Rust to język programowania silnika i serwera. Cargo buduje programy Rust i zarządza zależnościami. Narzędzia te są potrzebne tylko do budowania ze źródeł. [Instrukcja serwera (po angielsku)](/Orchestrata/docs/backend-setup/) opisuje instalację i sprawdzanie wersji.

Wszystkie poniższe polecenia dotyczące źródeł uruchom w głównym katalogu checkoutu Orchestrata. Przykłady powłoki używają Linuxa, macOS lub WSL.

## Uruchom lokalny serwer

[Instrukcja serwera (po angielsku)](/Orchestrata/docs/backend-setup/) prowadzi przez instalację i uruchomienie. Witryna i serwer są oddzielne. Pierwsze żądanie nie wymaga interfejsu przeglądarkowego ani bazy danych.

```sh
ORCHESTRATA_HTTP_BIND=127.0.0.1:3000 \
  cargo run --manifest-path backend/Cargo.toml --locked -p orchestrata-api
```

## Użyj biblioteki natywnej

Wybierz bibliotekę języka w [instrukcji SDK (po angielsku)](/Orchestrata/docs/sdk/). Korzysta ona z publicznych źródeł silnika, pasującej biblioteki natywnej i lokalnej instalacji pakietu.

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
