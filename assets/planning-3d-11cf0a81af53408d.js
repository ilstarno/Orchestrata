// Load neither geometry nor planning data until visitors approach a 3D viewer.
const roots = document.querySelectorAll('[data-planning-3d]')
async function start(root) {
  const status = root.querySelector('[data-planning-load]')
  status.hidden = false
  try {
    const path = root.querySelector('[data-planning-data-src]').dataset.planningDataSrc
    const [{mountViewer},config] = await Promise.all([
      import('./planning-3d-viewer-11d9a35322a861f4.js'),
      fetch(path).then(response => {
        if (!response.ok) throw new Error('Scene data unavailable')
        return response.json()
      }),
    ])
    if (!mountViewer(root, config)) throw new Error('Canvas unavailable')
  } catch {
    status.textContent = status.dataset.unavailable
  }
}
if ('IntersectionObserver' in window) {
  const observer = new IntersectionObserver(entries => {
    for (const {target,isIntersecting} of entries) if (isIntersecting) {
      observer.unobserve(target)
      start(target)
    }
  },{rootMargin:'240px'})
  roots.forEach(root => observer.observe(root))
} else roots.forEach(start)
