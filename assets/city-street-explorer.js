(() => {
  'use strict'
  const L = window.L
  if (!L) return
  const own = (object, key) => Object.hasOwn(object, key)
  const kinds = { start: '↗', pickup: '+', delivery: '↓', end: '■', depot: '◆' }
  const modes = ['vehicle', 'bicycle', 'walking']
  const point = value => Array.isArray(value) && value.length === 2 && value.every(Number.isFinite) &&
    Math.abs(value[0]) <= 180 && Math.abs(value[1]) <= 90
  const asLatLng = ([x, y]) => [y, x]
  const id = value => typeof value === 'number' && Number.isSafeInteger(value) && value > 0 && value <= 999
    ? String(value) : typeof value === 'string' && /^(?:[A-Z]{2,8}-)?\d{1,3}$/.test(value) ? value : null
  const same = (a, b) => a[0] === b[0] && a[1] === b[1]
  function read(data, inputs) {
    if (data?.type !== 'FeatureCollection' || !Array.isArray(data.features) ||
        !data.features.length || data.features.length > 200) throw Error('Invalid route file')
    const groups = new Map()
    let vertices = 0
    for (const item of data.features) {
      const { geometry, properties: prop } = item ?? {}
      const key = id(prop?.courierId)
      const box = inputs.get(key) ?? (key ? inputs.get(String(Number(key.split('-').at(-1)))) : undefined)
      if (item?.type !== 'Feature' || !key || !box || !modes.includes(prop.mode)) throw Error('Invalid courier')
      const group = groups.get(key) ?? { key, box, mode: prop.mode, stops: [], path: null }
      if (group.mode !== prop.mode) throw Error('Inconsistent courier mode')
      groups.set(key, group)
      const coords = geometry?.coordinates
      if (geometry?.type === 'LineString') {
        if (group.path || !Array.isArray(coords) || coords.length < 2 || (vertices += coords.length) > 30_000 ||
          !coords.every(point) || ![prop.distanceMeters, prop.durationSeconds].every(value => Number.isFinite(value) && value > 0))
          throw Error('Invalid line')
        group.path = coords
      } else {
        if (geometry?.type !== 'Point' || !point(coords) || !Number.isSafeInteger(prop.sequence) ||
          prop.sequence < 0 || prop.sequence > 80 || !own(kinds, prop.kind)) throw Error('Invalid stop')
        group.stops.push({ kind: prop.kind, sequence: prop.sequence, coord: coords })
      }
    }
    if (!groups.size || groups.size !== inputs.size || new Set([...groups.values()].map(group => group.box)).size !== inputs.size)
      throw Error('Missing courier')
    for (const group of groups.values()) {
      group.stops.sort((a, b) => a.sequence - b.sequence)
      if (!group.path || group.stops.length < 4 || new Set(group.stops.map(stop => stop.sequence)).size !== group.stops.length ||
        group.stops[0].kind !== 'start' || group.stops.at(-1).kind !== 'end' ||
        !same(group.stops[0].coord, group.path[0]) || !same(group.stops.at(-1).coord, group.path.at(-1)))
        throw Error('Missing route or ordered stops')
      let pickups = 0, deliveries = 0, cursor = 0
      for (const stop of group.stops) {
        const index = group.path.findIndex((coordinate, i) => i >= cursor && same(coordinate, stop.coord))
        if (index < 0) throw Error('Stop is not in line order')
        cursor = index
        if (stop.kind === 'pickup') pickups++
        if (stop.kind === 'delivery' && ++deliveries > pickups) throw Error('Drop-off must follow pickup')
      }
      if (!deliveries || deliveries !== pickups) throw Error('Missing paired stops')
      group.latlngs = group.path.map(asLatLng)
    }
    return [...groups.values()]
  }

  const sizeLimit = 1_048_576
  async function routeFile(value) {
    const url = new URL(value, document.baseURI)
    if (url.origin !== location.origin || !url.pathname.endsWith('.geojson')) throw Error('Expected local route file')
    const response = await fetch(url, { redirect: 'error', mode: 'same-origin' })
    if (!response.ok || Number(response.headers.get('content-length')) > sizeLimit || !response.body) throw Error('Route file unavailable')
    const reader = response.body.getReader()
    const decoder = new TextDecoder('utf-8', { fatal: true })
    let body = '', bytes = 0
    try {
      while (true) {
        const next = await reader.read()
        if (next.done) break
        bytes += next.value.byteLength
        if (bytes > sizeLimit) throw Error('Route file is too large')
        body += decoder.decode(next.value, { stream: true })
      }
      return JSON.parse(body + decoder.decode())
    } finally { await reader.cancel().catch(() => {}) }
  }

  function node(tag, label, style) {
    const element = document.createElement(tag)
    if (label != null) element.textContent = label
    if (style) element.className = style
    return element
  }

  function initialize(panel) {
    if (panel.dataset.ready) return
    const q = selector => panel.querySelector(selector)
    const button = q('[data-enable]'), fieldset = q('[data-choices]'), state = q('[data-state]')
    const canvas = q('[data-map]'), itineraries = q('[data-itineraries]'), lists = q('[data-lists]')
    const labels = q('[data-labels]')?.dataset
    if (!button || !fieldset || !state || !canvas || !itineraries || !lists || !labels) return
    const inputs = new Map([...fieldset.querySelectorAll('input[type="checkbox"]')].map(box => [id(box.value), box]))
    if (inputs.has(null) || !inputs.size || inputs.size > 20) return
    const language = panel.dataset.language
    const number = new Intl.NumberFormat(language)
    const color = box => {
      const value = box.dataset.color ?? ''
      if (/^#[0-9a-f]{6}$/i.test(value)) return value
      const hsl = /^hsl\((\d+(?:\.\d+)?) (\d+)% (\d+)%\)$/.exec(value)
      if (hsl && Number(hsl[1]) <= 360 && Number(hsl[2]) <= 100 && Number(hsl[3]) <= 100)
        return 'hsl(' + hsl[1] + ' ' + hsl[2] + '% ' + Math.min(38, Number(hsl[3])) + '%)'
      return '#176153'
    }
    for (const box of inputs.values()) box.parentElement.style.setProperty('--courier-hue', color(box))
    panel.dataset.ready = '1'
    button.hidden = false
    let map
    button.addEventListener('click', async () => {
      button.disabled = true
      state.textContent = labels.load
      try {
        const groups = read(await routeFile(panel.dataset.geojson), inputs)
        canvas.hidden = false
        map = L.map(canvas, { scrollWheelZoom: false, minZoom: 10, maxZoom: 18 })
        L.tileLayer('https://tile.openstreetmap.org/{z}/{x}/{y}.png', {
          maxZoom: 18, attribution: '&copy; <a href="https://www.openstreetmap.org/copyright">OpenStreetMap contributors</a>',
        }).addTo(map)
        const layers = L.layerGroup().addTo(map)
        const refresh = () => {
          const selected = groups.filter(group => group.box.checked)
          layers.clearLayers()
          lists.replaceChildren()
          const shown = selected.length ? selected : groups
          const bounds = L.latLngBounds([])
          for (const group of shown) {
            const hue = color(group.box)
            const dashArray = group.mode === 'walking' ? '2 8' : group.mode === 'bicycle' ? '8 7' : undefined
            L.polyline(group.latlngs, { color: '#fff', dashArray, opacity: .96, weight: 8, interactive: false }).addTo(layers)
            L.polyline(group.latlngs, { color: hue, dashArray, weight: 5, interactive: false }).addTo(layers)
            for (const location of group.latlngs) bounds.extend(location)
            if (!selected.length) continue
            const list = node('ol')
            const block = node('div', null, 'street-explorer__route')
            block.style.setProperty('--courier-hue', hue)
            block.append(node('h4', labels.courier + ' ' + group.key + ' · ' + labels[group.mode]), list)
            for (const stop of group.stops) {
              const latlng = asLatLng(stop.coord)
              const description = number.format(stop.sequence) + ' · ' + labels[stop.kind]
              const symbol = node('span', number.format(stop.sequence) + ' ' + kinds[stop.kind], 'street-explorer__marker')
              symbol.style.setProperty('--courier-hue', hue)
              const icon = L.divIcon({ html: symbol, className: 'street-explorer__icon', iconSize: [38, 28], iconAnchor: [19, 14] })
              const marker = L.marker(latlng, { icon, keyboard: false })
                .bindTooltip(node('span', group.key + ' · ' + description)).addTo(layers)
              const row = node('li')
              const focusStop = node('button', description)
              focusStop.type = 'button'
              focusStop.addEventListener('click', () => {
                map.panTo(latlng, { animate: false })
                marker.openTooltip()
              })
              row.append(focusStop)
              list.append(row)
            }
            lists.append(block)
          }
          map.fitBounds(bounds, { padding: [30, 30], maxZoom: 16, animate: false })
          itineraries.hidden = !selected.length
          state.textContent = selected.length ? labels.selected + ': ' + selected.map(group => group.key).join(', ') : labels.none
        }
        groups[0].box.checked = true
        refresh()
        fieldset.addEventListener('change', refresh)
        fieldset.hidden = false
        button.hidden = true
        groups[0].box.focus()
      } catch {
        map?.remove()
        map = undefined
        canvas.hidden = true
        fieldset.hidden = true
        itineraries.hidden = true
        button.disabled = false
        state.textContent = labels.error
      }
    })
  }

  document.querySelectorAll('[data-street-explorer]').forEach(initialize)
})()
