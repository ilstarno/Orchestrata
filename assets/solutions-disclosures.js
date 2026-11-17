(() => {
  function revealLinkedSection() {
    if (!location.hash) return;
    let id;
    try {
      id = decodeURIComponent(location.hash.slice(1));
    } catch {
      return;
    }
    const section = document.getElementById(id);
    if (!section || !section.closest('.buyer-orientation, [data-buyer-details]')) return;
    let disclosure = section.closest('details');
    let opened = false;
    while (disclosure) {
      if (!disclosure.open) {
        disclosure.open = true;
        opened = true;
      }
      disclosure = disclosure.parentElement?.closest('details');
    }
    if (opened) section.scrollIntoView();
  }
  revealLinkedSection();
  addEventListener('hashchange', revealLinkedSection);
})();
