## Obsadzić cztery zmiany mimo nieobecności

O godz. 14.00 każdy z dwóch sklepów potrzebuje dwóch pracowników. Ben zgłasza nieobecność. Porównaj pierwotny plan z drugim obliczeniem, w którym Ben nie jest już dostępny na dwóch zmianach, które mógł obsadzić. Zmiany, reguły, wagi i pozostałe możliwości przydziału nie zmieniają się.

### Potrzebne dane

Dla każdej zmiany podaj lokalizację, zadanie oraz osoby dostępne i uprawnione do jego wykonywania. Dodaj przydziały stałe i preferencje, od których można odstąpić. Zachowaj te same indeksy pracowników w obu obliczeniach, aby móc porównać wyniki.

| Pole przepisu | Typ | Przykładowa wartość |
| --- | --- | --- |
| `shift-names` | Lista ciągów znaków | Północ osoba odpowiedzialna za klucze, Północ sala sprzedaży, Południe kasa, Południe magazyn |
| `worker-names` | Lista ciągów znaków | Ana, Ben, Clara, Diego, Elena; indeksy od 0 do 4 |
| `eligible-pairs` | Lista liczb całkowitych | Pary indeksów zmiany i pracownika; po zgłoszeniu nieobecności Bena usuń 1,1 i 2,1 |
| `pinned-pairs` | Opcjonalna lista liczb całkowitych | 0,0 przydziela Anie odpowiedzialność za klucze w sklepie Północ |
| `preferred-assignments` | Opcjonalna lista liczb całkowitych | Trójki: indeks zmiany, indeks pracownika i waga, np. 1,1,5; 2,2,3; 3,3,2 |

W danych wejściowych każda zmiana jest zmienną. Stały przydział dopuszcza tylko jedną osobę: dla `north_keyholder` jest to `domain: [0]`. Obowiązkowa reguła `all_different` pozostaje taka sama w obu obliczeniach. Trzy reguły `prefer_value` zachowują pierwotne wagi, nawet jeśli danej preferencji nie można już spełnić.

### Porównanie planów

| Zmiana | Osoby dostępne i uprawnione przed nieobecnością | Plan pierwotny | Po zgłoszeniu nieobecności |
| --- | --- | --- | --- |
| Północ osoba odpowiedzialna za klucze | Ana | Ana | Ana |
| Północ sala sprzedaży | Ben, Clara | Ben | Clara |
| Południe kasa | Ben, Clara, Elena | Clara | Elena |
| Południe magazyn | Diego, Elena | Diego | Diego |

Oba plany obsadzają wszystkie cztery zmiany. Żadna osoba nie zostaje przydzielona do dwóch zmian naraz. Pierwotny wynik to `[0, 1, 2, 3]`, ocena 0/0 dla wymagań i preferencji oraz 12 sprawdzonych kombinacji. Po zgłoszeniu nieobecności wynik to `[0, 2, 4, 3]`, ocena 0/−8 i 4 sprawdzone kombinacje. Na −8 składają się dwie niespełnione preferencje: 5 punktów za Bena na sali sprzedaży w sklepie Północ i 3 za Clarę przy kasie w sklepie Południe. Żaden warunek obowiązkowy nie zostaje naruszony.

### Uruchomienie obu wariantów

Zapisz [pierwotne dane wejściowe](baseline-problem.json) i [zmienione dane wejściowe](adjusted-problem.json):

```sh
orchestrata-cli baseline-problem.json
orchestrata-cli adjusted-problem.json
```

Porównaj [pierwotny wynik](baseline-expected.json) i [zmieniony wynik](adjusted-expected.json). Oba warianty możesz też uruchomić przez [lokalny interfejs API](../../docs/api/).

### Czego nie obejmuje ten przykład

Cztery zmiany są ustalone i nakładają się na siebie. Przykład nie prognozuje zapotrzebowania, nie synchronizuje kalendarzy, nie nalicza płac ani nie uwzględnia nadgodzin czy okresów odpoczynku. Nie kontroluje zgodności z prawem pracy ani nie buduje grafiku na kilka tygodni.
