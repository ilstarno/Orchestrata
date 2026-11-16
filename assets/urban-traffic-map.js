(() => {
  'use strict'
  const L = window.L
  if (!L) return
  const variants = ['baseline', 'slowdown', 'closure']
  const known = ['route', 'reduced_speed', 'closed_segment', 'origin', 'destination']
  const point = value => Array.isArray(value) && value.length === 2 && value.every(Number.isFinite) &&
    Math.abs(value[0]) <= 180 && Math.abs(value[1]) <= 90
  const latLng = coords => [coords[1], coords[0]]
  function read(data) {
    if (data?.type !== 'FeatureCollection' || !Array.isArray(data.features) || data.features.length !== 7) throw Error('Invalid replay')
    const routes = new Map(), updates = [], endpoints = new Map()
    for (const feature of data.features) {
      const kind = feature?.properties?.kind, coords = feature?.geometry?.coordinates
      if (feature?.type !== 'Feature' || !known.includes(kind)) throw Error('Unknown replay feature')
      if (kind === 'origin' || kind === 'destination') {
        if (feature.geometry.type !== 'Point' || !point(coords) || endpoints.has(kind)) throw Error('Invalid endpoint')
        endpoints.set(kind, latLng(coords))
        continue
      }
      if (feature.geometry.type !== 'LineString' || !Array.isArray(coords) || coords.length < 2 ||
        coords.length > 1600 || !coords.every(point)) throw Error('Invalid replay line')
      if (kind === 'route') {
        const variant = feature.properties.variant
        if (!variants.includes(variant) || routes.has(variant)) throw Error('Invalid route variant')
        routes.set(variant, coords.map(latLng))
      } else {
        const appliesTo = kind === 'reduced_speed' ? ['slowdown', 'closure'] : ['closure']
        if (updates.some(x => x.kind === kind) || !Array.isArray(feature.properties.appliesTo) ||
          appliesTo.join(',') !== feature.properties.appliesTo.join(',')) throw Error('Invalid simulated update')
        updates.push({ kind, appliesTo, points: coords.map(latLng) })
      }
    }
    if (endpoints.size !== 2 || routes.size !== 3 || updates.length !== 2) throw Error('Incomplete replay')
    return { routes, updates, endpoints }
  }
  async function fetchLocal(path) {
    const url = new URL(path, document.baseURI)
    if (url.origin !== location.origin || !url.pathname.endsWith('.geojson')) throw Error('Expected local map')
    const response = await fetch(url, { mode: 'same-origin', redirect: 'error' })
    const cap = 250_000
    if (!response.ok || !response.body || Number(response.headers.get('content-length')) > cap) throw Error('Map unavailable')
    const reader = response.body.getReader(), decoder = new TextDecoder('utf-8', { fatal: true })
    let text = '', size = 0
    try {
      while (true) {
        const chunk = await reader.read()
        if (chunk.done) return read(JSON.parse(text + decoder.decode()))
        size += chunk.value.byteLength
        if (size > cap) throw Error('Map too large')
        text += decoder.decode(chunk.value, { stream: true })
      }
    } finally { await reader.cancel().catch(() => {}) }
  }
  function initialize(panel) {
    if (panel.dataset.ready) return
    const q = selector => panel.querySelector(selector)
    const button = q('[data-enable]'), mapNode = q('[data-map]'), choices = q('[data-cases]')
    const key = q('[data-key]'), status = q('[data-status]'), labels = q('[data-labels]')?.dataset
    if (!button || !mapNode || !choices || !status || !labels || !key) return
    panel.dataset.ready = '1'
    button.hidden = false
    button.addEventListener('click', async () => {
      button.disabled = true
      status.textContent = labels.loading
      let map
      try {
        const data = await fetchLocal(panel.dataset.url)
        mapNode.hidden = false
        map = L.map(mapNode, { scrollWheelZoom: false, maxZoom: 18 })
        const layers = L.layerGroup().addTo(map)
        function render(variant) {
          const points = data.routes.get(variant)
          if (!points) return
          layers.clearLayers()
          const updateStyle = {
            reduced_speed: { color: '#a6530b', weight: 11, dashArray: '7 7' },
            closed_segment: { color: '#b12531', weight: 9, dashArray: '2 6' },
          }
          const visibleUpdates = data.updates.filter(entry => entry.appliesTo.includes(variant))
          for (const edge of visibleUpdates) {
            L.polyline(edge.points, { color: '#fff', weight: 14, interactive: false }).addTo(layers)
            L.polyline(edge.points, { ...updateStyle[edge.kind], interactive: false }).addTo(layers)
          }
          L.polyline(points, { color: '#fff', weight: 6, interactive: false }).addTo(layers)
          L.polyline(points, { color: '#126256', weight: 4, interactive: false }).addTo(layers)
          for (const [label, endpoint] of data.endpoints) {
            const text = document.createElement('span')
            text.textContent = labels[label]
            L.circleMarker(endpoint, { color: '#fff', weight: 2, radius: 6, fillColor: '#163b34', fillOpacity: 1 })
              .bindTooltip(text, { permanent: true, direction: label === 'origin' ? 'left' : 'right', offset: [0, 0] }).addTo(layers)
          }
          const bounds = L.latLngBounds(points)
          for (const edge of visibleUpdates) for (const point of edge.points) bounds.extend(point)
          map.fitBounds(bounds, { padding: [32, 32], maxZoom: 17, animate: false })
          for (const row of panel.querySelectorAll('[data-scenario]')) {
            const active = row.dataset.scenario === variant
            if (active) row.setAttribute('aria-current', 'true')
            else row.removeAttribute('aria-current')
            if (active) status.textContent = row.dataset.announcement
          }
          for (const item of key.querySelectorAll('[data-variant]')) item.hidden = !item.dataset.variant.split(' ').includes(variant)
        }
        render('baseline')
        L.tileLayer('https://tile.openstreetmap.org/{z}/{x}/{y}.png', {
          maxZoom: 18, attribution: '&copy; <a href="https://www.openstreetmap.org/copyright">OpenStreetMap contributors</a>',
        }).addTo(map)
        choices.addEventListener('change', event => render(event.target.value))
        key.hidden = choices.hidden = false
        button.hidden = true
        choices.querySelector('input:checked')?.focus()
      } catch {
        map?.remove()
        mapNode.hidden = true
        button.disabled = false
        status.textContent = labels.error
      }
    })
  }
  document.querySelectorAll('[data-urban-traffic]').forEach(initialize)
})()
