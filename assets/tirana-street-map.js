(() => {
  'use strict'
  const panel = document.querySelector('[data-tirana-street-map]')
  if (!panel || !window.L) return
  const button = panel.querySelector('button[data-show-streets]')
  const mapTarget = panel.querySelector('[data-street-viewport]')
  const controls = panel.querySelector('[data-street-controls]')
  const select = panel.querySelector('select')
  const status = panel.querySelector('[role="status"]')
  if (!button || !mapTarget || !controls || !select || !status) return

  const colors = ['#0b7662', '#104eaa', '#aa3f37', '#603ea0', '#8a5004', '#007099', '#a82268', '#324356']
  const idPattern = /^TIA-00[1-8]$/
  const modes = ['vehicle', 'bicycle', 'walking']
  const point = coord => Array.isArray(coord) && coord.length === 2 && coord.every(Number.isFinite)
    && coord[0] >= 19.6 && coord[0] <= 20 && coord[1] >= 41.1 && coord[1] <= 41.6
  const modeLabel = { vehicle: panel.dataset.vehicle, bicycle: panel.dataset.bicycle, walking: panel.dataset.walking }
  const routeColor = id => colors[Number(id.slice(-1)) - 1]
  const number = new Intl.NumberFormat(document.documentElement.lang, { maximumFractionDigits: 1 })
  let map

  function showMap(data) {
    if (data.type !== 'FeatureCollection' || !Array.isArray(data.features) || data.features.length > 144) throw new Error('Invalid collection')
    const features = data.features
    for (const item of features) {
      const info = item.properties
      if (item.type !== 'Feature' || !info || !idPattern.test(info.courierId) || !modes.includes(info.mode)) throw new Error('Invalid courier')
      if (item.geometry?.type === 'LineString') {
        if (!Array.isArray(item.geometry.coordinates) || item.geometry.coordinates.length < 3 ||
          item.geometry.coordinates.length > 20_000 || !item.geometry.coordinates.every(point) ||
          !Number.isFinite(info.distanceMeters) || info.distanceMeters <= 0 || !Number.isFinite(info.durationSeconds) || info.durationSeconds <= 0) throw new Error('Invalid route')
      } else if (item.geometry?.type !== 'Point' || !point(item.geometry.coordinates) ||
                 !Number.isSafeInteger(info.sequence) || info.sequence < 0 || info.sequence > 20) throw new Error('Invalid stop')
    }
    const routes = features.filter(item => item.geometry.type === 'LineString')
    if (routes.length !== 8 || new Set(routes.map(item => item.properties.courierId)).size !== 8) throw new Error('Missing route')

    mapTarget.hidden = false
    map = L.map(mapTarget, { scrollWheelZoom: false, minZoom: 11, maxZoom: 17, preferCanvas: true })
    L.tileLayer('https://tile.openstreetmap.org/{z}/{x}/{y}.png', {
      maxZoom: 17, attribution: '&copy; <a href="https://www.openstreetmap.org/copyright">OpenStreetMap contributors</a>',
    }).addTo(map)
    const layer = L.layerGroup().addTo(map)
    for (const { properties: info } of routes) {
      const option = document.createElement('option')
      option.value = info.courierId
      option.textContent = info.courierId + ' · ' + modeLabel[info.mode]
      select.append(option)
    }

    const refresh = () => {
      layer.clearLayers()
      const selected = features.filter(item => !select.value || item.properties.courierId === select.value)
      const display = L.geoJSON({ type: 'FeatureCollection', features: selected }, {
        style: ({ properties: info }) => ({
          color: routeColor(info.courierId), opacity: .9, weight: info.mode === 'vehicle' ? 4 : 5,
          dashArray: info.mode === 'walking' ? '2 6' : info.mode === 'bicycle' ? '8 6' : undefined,
        }),
        pointToLayer: ({ properties: info }, latLng) => L.circleMarker(latLng, {
          radius: /^(start|end|depot)$/.test(info.kind) ? 7 : 5,
          weight: 2, color: '#fff', fillColor: routeColor(info.courierId), fillOpacity: 1,
        }),
        onEachFeature: ({ geometry, properties: info }, item) => {
          const label = document.createElement('span')
          label.textContent = geometry.type === 'LineString' ? info.courierId + ' · ' + modeLabel[info.mode]
            : info.courierId + ' · ' + panel.dataset.stops + ' ' + info.sequence
          item.bindTooltip(label)
        },
      }).addTo(layer)
      map.fitBounds(display.getBounds(), { padding: [22, 22], maxZoom: 15, animate: false })
      const route = routes.find(item => item.properties.courierId === select.value)?.properties
      if (route) {
        status.textContent = route.courierId + ' · ' + modeLabel[route.mode] + ' · '
          + panel.dataset.distance + ' ' + number.format(route.distanceMeters / 1000) + ' km · '
          + panel.dataset.duration + ' ' + number.format(route.durationSeconds / 60) + ' min'
      } else status.textContent = panel.dataset.summary
    }
    select.addEventListener('change', refresh)
    controls.hidden = false
    refresh()
  }

  button.hidden = false
  button.addEventListener('click', async () => {
    button.disabled = true
    status.textContent = panel.dataset.loading
    try {
      const url = new URL(panel.dataset.routeUrl, document.baseURI)
      if (url.origin !== location.origin) throw new Error('Expected same-origin route artifact')
      const response = await fetch(url)
      if (!response.ok) throw new Error('Could not load routes')
      showMap(await response.json())
      button.hidden = true
      select.focus()
    } catch {
      if (map) map.remove()
      select.replaceChildren(select.options[0])
      mapTarget.hidden = true
      controls.hidden = true
      button.disabled = false
      status.textContent = panel.dataset.error
    }
  }, { passive: true })
})()
