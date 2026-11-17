/** Format valid JSON by changing only whitespace outside its original tokens. */
export function formatExampleJSON(source) {
  // Large payloads stay in their original scrollable block.
  if (source.length > 128_000) return source;
  try { JSON.parse(source); } catch { return source; }
  const tokens = source.match(/"(?:[^"\\]|\\[\s\S])*"|-?(?:0|[1-9]\d*)(?:\.\d+)?(?:[eE][+-]?\d+)?|true|false|null|[{}\[\],:]/g);
  if (!tokens) return source;
  const result = [];
  let depth = 0;
  const line = () => "\n" + "  ".repeat(depth);
  const opens = token => token === "{" || token === "[";
  const closes = token => token === "}" || token === "]";
  for (const [i, token] of tokens.entries()) {
    const paired = opens(tokens[i - 1]) && closes(token);
    if (closes(token) && !paired) { depth--; result.push(line()); }
    result.push(token);
    if (opens(token) && !closes(tokens[i + 1])) { depth++; result.push(line()); }
    else if (token === ",") result.push(line());
    else if (token === ":") result.push(" ");
    // Do not amplify deeply nested samples into huge displays.
    if (depth > 48) return source;
  }
  return result.join("");
}

const words = {
  en: ["Example", "Command", "Code", "Copy", "Copy original", "Copied", "Copy unavailable. Select and copy the original text.", "Show original", "Format JSON"],
  de: ["Beispiel", "Befehl", "Code", "Kopieren", "Original kopieren", "Kopiert", "Kopieren nicht möglich. Markieren und kopieren Sie den Originaltext.", "Original anzeigen", "JSON formatieren"],
  fr: ["Exemple", "Commande", "Code", "Copier", "Copier l’original", "Copié", "Copie impossible. Sélectionnez et copiez le texte d’origine.", "Voir l’original", "Mettre en forme le JSON"],
  it: ["Esempio", "Comando", "Codice", "Copia", "Copia l’originale", "Copiato", "Impossibile copiare. Seleziona e copia il testo originale.", "Mostra l’originale", "Formatta JSON"],
  nl: ["Voorbeeld", "Opdracht", "Code", "Kopiëren", "Origineel kopiëren", "Gekopieerd", "Kopiëren lukt niet. Selecteer en kopieer de oorspronkelijke tekst.", "Origineel tonen", "JSON opmaken"],
  pl: ["Przykład", "Polecenie", "Kod", "Kopiuj", "Kopiuj oryginał", "Skopiowano", "Nie udało się skopiować. Zaznacz i skopiuj oryginalny tekst.", "Pokaż oryginał", "Sformatuj JSON"],
};
const languages = { sh: "Shell", bash: "Bash", shell: "Shell", json: "JSON", csharp: "C#", java: "Java", go: "Go", python: "Python", php: "PHP", javascript: "JavaScript", xml: "XML", rust: "Rust", toml: "TOML", yaml: "YAML", http: "HTTP", text: "" };

export function enhanceExamples(doc, clipboard = () => globalThis.navigator?.clipboard) {
  const labels = words[doc.documentElement.lang.toLowerCase().split("-")[0]] || words.en;
  const make = (tag, className, value = "") => {
    const el = doc.createElement(tag);
    el.className = className;
    el.textContent = value;
    return el;
  };

  doc.querySelectorAll("[data-doc-reader] pre > code").forEach((code, i) => {
    const pre = code.parentElement;
    if (pre.closest(".docs-example")) return;
    const raw = code.textContent;
    const lang = Array.from(code.classList).find(name => name.startsWith("language-"))?.slice(9);
    const shown = lang === "json" ? formatExampleJSON(raw) : raw;
    const formatted = shown !== raw;
    let showingOriginal = !formatted;
    let pending = false;
    let timer;
    const id = `docs-reader-example-${i + 1}`;
    const title = labels[lang === "json" ? 0 : /^(sh|shell|bash)$/.test(lang) ? 1 : 2];
    const label = make("span", "docs-example-label", languages[lang] ? `${title} (${languages[lang]})` : title);
    label.id = id;
    const frame = make("figure", "docs-example");
    const bar = make("figcaption", "docs-example-bar");
    const actions = make("span", "docs-example-actions");
    const status = make("span", "docs-example-status");
    status.setAttribute("role", "status");
    const copy = make("button", "docs-example-button", labels[formatted ? 4 : 3]);
    copy.type = "button";
    const displayOriginal = value => {
      showingOriginal = value;
      code.textContent = value ? raw : shown;
      if (toggle) toggle.textContent = labels[value ? 8 : 7];
    };
    const toggle = formatted ? make("button", "docs-example-button", labels[7]) : null;
    if (toggle) {
      toggle.type = "button";
      toggle.addEventListener("click", () => displayOriginal(!showingOriginal));
      actions.append(toggle);
    }
    actions.append(copy);
    copy.addEventListener("click", async () => {
      if (pending) return;
      pending = true;
      clearTimeout(timer);
      status.textContent = "";
      try {
        const target = clipboard();
        if (!target?.writeText) throw new Error();
        await target.writeText(raw);
        status.textContent = labels[5];
        timer = setTimeout(() => { status.textContent = ""; }, 3500);
      } catch {
        displayOriginal(true);
        status.textContent = labels[6];
        pre.focus();
      } finally { pending = false; }
    });

    bar.append(label, actions);
    frame.append(bar);
    pre.before(frame);
    pre.setAttribute("role", "region");
    pre.setAttribute("aria-labelledby", id);
    frame.append(pre, status);
    if (formatted) displayOriginal(false);
  });
}

if (typeof document !== "undefined") enhanceExamples(document);
