(() => {
  'use strict'
  const panel = document.querySelector('[data-tirana-street-map]')
  if (!panel || panel.dataset.streetState) return
  const L = window.L
  const mapTarget = panel.querySelector('[data-street-viewport]')
  const controls = panel.querySelector('[data-street-controls]')
  const select = panel.querySelector('select')
  const status = panel.querySelector('[role="status"]')
  const chooser = panel.querySelector('[data-street-chooser]')
  const legend = panel.querySelector('[data-street-legend]')
  const itinerary = panel.querySelector('[data-street-itinerary]')
  const stopList = itinerary?.querySelector('ol')
  if (!mapTarget || !controls || !select || !status || !chooser || !legend || !itinerary || !stopList) return

  const colors = ['#147362', '#2459a4', '#ad3a32', '#6940a5', '#90530c', '#00788e', '#a31766', '#364457']
  const idPattern = /^TIA-00[1-8]$/
  const modes = ['vehicle', 'bicycle', 'walking']
  const symbols = { start: '↗', pickup: '+', delivery: '↓', end: '■', depot: '◆' }
  const kindLabels = Object.fromEntries(Object.keys(symbols).map(kind => [kind, panel.dataset[kind]]))
  const point = coord => Array.isArray(coord) && coord.length === 2 && coord.every(Number.isFinite)
    && coord[0] >= 19.6 && coord[0] <= 20 && coord[1] >= 41.1 && coord[1] <= 41.6
  const modeLabel = { vehicle: panel.dataset.vehicle, bicycle: panel.dataset.bicycle, walking: panel.dataset.walking }
  const routeColor = id => colors[Number(id.slice(-1)) - 1]
  const number = new Intl.NumberFormat(document.documentElement.lang, { maximumFractionDigits: 1 })
  const toLatLng = ([lon, lat]) => [lat, lon]
  const dash = mode => mode === 'walking' ? '2 9' : mode === 'bicycle' ? '9 7' : undefined
  const region = (mode, pane) => ({ pane, dashArray: dash(mode), lineCap: 'round', lineJoin: 'round', interactive: false, smoothFactor: 0 })
  const clock = new Intl.NumberFormat(document.documentElement.lang, { minimumIntegerDigits: 2, useGrouping: false })
  const timeLabel = seconds => clock.format(Math.floor(seconds / 3600)) + ':' + clock.format(Math.floor(seconds / 60) % 60)
  const nonnegative = number => Number.isSafeInteger(number) && number >= 0
  const samePoint = (a, b) => point(a) && a[0] === b[0] && a[1] === b[1]
  const dispatch = new Map()
  let map
  let requestedCourier
  let observer
  panel.dataset.streetState = 'waiting'

  async function loadDispatch(groups) {
    try {
      if (!panel.dataset.dispatchUrl) return false
      const url = new URL(panel.dataset.dispatchUrl, document.baseURI)
      if (url.origin !== location.origin) return false
      const response = await fetch(url)
      if (!response.ok) return false
      const data = await response.json()
      if (!Array.isArray(data?.runs) || !Array.isArray(data?.hubs) || data.runs.length > 8 || data.hubs.length > 8) return false
      for (const group of groups) {
        const run = data.runs.find(row => row.courierId === group.id && row.mode === group.mode)
        if (!Array.isArray(run?.stops) || run.stops.length !== group.stops.length) continue
        const stops = new Map()
        for (const item of group.stops) {
          const info = item.properties
          const row = run.stops.find(stop => stop.sequence === info.sequence && stop.kind === info.kind &&
            stop.osmNodeId === info.osmNodeId && samePoint(stop.coordinate, item.geometry.coordinates))
          if (!row || !nonnegative(row.serviceStartSeconds) || row.serviceStartSeconds >= 86400 || !nonnegative(row.loadGrams)) continue
          stops.set(info.sequence, row)
        }
        const hub = data.hubs.find(row => row.courierId === group.id && /^HUB-0[1-8]$/.test(row.id) &&
          group.stops.some(item => item.properties.kind === 'start' && row.sequence === item.properties.sequence &&
            samePoint(row.coordinate, item.geometry.coordinates)))
        dispatch.set(group.id, { stops, hub })
      }
      return dispatch.size > 0
    } catch { return false }
  }

  function readRoutes(data) {
    if (data?.type !== 'FeatureCollection' || !Array.isArray(data.features) || data.features.length > 144) throw new Error('Invalid collection')
    const groups = new Map()
    for (const item of data.features) {
      const info = item?.properties
      if (item?.type !== 'Feature' || !info || !idPattern.test(info.courierId) || !modes.includes(info.mode)) throw new Error('Invalid courier')
      const group = groups.get(info.courierId) ?? { id: info.courierId, mode: info.mode, stops: [], line: null }
      if (group.mode !== info.mode) throw new Error('Inconsistent mode')
      groups.set(group.id, group)
      if (item.geometry?.type === 'LineString') {
        if (group.line || !Array.isArray(item.geometry.coordinates) || item.geometry.coordinates.length < 3 ||
          item.geometry.coordinates.length > 20_000 || !item.geometry.coordinates.every(point) ||
          !Number.isFinite(info.distanceMeters) || info.distanceMeters <= 0 || !Number.isFinite(info.durationSeconds) || info.durationSeconds <= 0) throw new Error('Invalid route')
        group.line = item
      } else {
        if (item.geometry?.type !== 'Point' || !point(item.geometry.coordinates) ||
          !Number.isSafeInteger(info.sequence) || info.sequence < 0 || info.sequence > 20 ||
          !Object.hasOwn(symbols, info.kind)) throw new Error('Invalid stop')
        group.stops.push(item)
      }
    }
    if (groups.size !== 8) throw new Error('Missing route')
    for (const group of groups.values()) {
      if (!group.line || !group.stops.length) throw new Error('Missing route or stop')
      group.stops.sort((a, b) => a.properties.sequence - b.properties.sequence)
      if (new Set(group.stops.map(item => item.properties.sequence)).size !== group.stops.length) throw new Error('Duplicate stops')
    }
    return [...groups.values()].sort((a, b) => a.id.localeCompare(b.id))
  }

  function markerSymbol(kind, value, color) {
    const span = document.createElement('span')
    span.className = 'tirana-street-symbol tirana-street-symbol--' + kind
    span.style.setProperty('--courier-color', color)
    span.textContent = value
    return span
  }

  function stopDescription(info) {
    return number.format(info.sequence) + ' · ' + kindLabels[info.kind]
  }

  function showMap(data) {
    const groups = readRoutes(data)
    mapTarget.hidden = false
    map = L.map(mapTarget, { scrollWheelZoom: false, minZoom: 11, maxZoom: 18, preferCanvas: true })
    L.tileLayer('https://tile.openstreetmap.org/{z}/{x}/{y}.png', {
      maxZoom: 18, attribution: '&copy; <a href="https://www.openstreetmap.org/copyright">OpenStreetMap contributors</a>',
    }).addTo(map)
    map.createPane('tirana-selection').style.zIndex = '430'
    const context = L.layerGroup().addTo(map)
    const selection = L.layerGroup().addTo(map)
    const points = L.layerGroup().addTo(map)
    const pickers = new Map()
    for (const group of groups) {
      const option = document.createElement('option')
      option.value = group.id
      option.textContent = group.id + ' · ' + modeLabel[group.mode]
      select.append(option)
      const item = document.createElement('button')
      item.type = 'button'
      item.className = 'tirana-street-courier'
      item.style.setProperty('--courier-color', routeColor(group.id))
      item.dataset.mode = group.mode
      item.textContent = group.id
      item.title = modeLabel[group.mode]
      item.setAttribute('aria-label', option.textContent)
      item.setAttribute('aria-pressed', 'false')
      item.addEventListener('click', () => {
        select.value = select.value === group.id ? '' : group.id
        refresh()
      })
      chooser.append(item)
      pickers.set(group.id, item)
    }

    function refreshLegend() {
      legend.replaceChildren()
      for (const kind of Object.keys(symbols)) {
        const present = groups.some(group => group.stops.some(item => item.properties.kind === kind) ||
          (kind === 'depot' && dispatch.get(group.id)?.hub))
        if (!present) continue
        const item = document.createElement('li')
        item.append(markerSymbol(kind, symbols[kind], '#364457'), document.createTextNode(kindLabels[kind]))
        legend.append(item)
      }
    }

    function addStop(item, group, showItinerary = true) {
      const info = item.properties
      const color = routeColor(group.id)
      const hub = dispatch.get(group.id)?.hub
      const depot = hub?.sequence === info.sequence ? hub : undefined
      const detail = dispatch.get(group.id)?.stops.get(info.sequence)
      const kind = depot ? 'depot' : info.kind
      const numbered = markerSymbol(kind, symbols[kind], color)
      const order = document.createElement('b')
      order.className = 'tirana-map-stop-order'
      order.textContent = number.format(info.sequence)
      numbered.append(order)
      const location = toLatLng(item.geometry.coordinates)
      const marker = L.marker(location, {
        icon: L.divIcon({ html: numbered, className: 'tirana-map-stop', iconSize: [30, 30], iconAnchor: [15, 15] }),
        keyboard: false,
      })
      const label = document.createElement('span')
      label.textContent = group.id + ' · ' + stopDescription(info) + (depot ? ' · ' + kindLabels.depot + ' ' + depot.id : '')
      marker.bindTooltip(label, { direction: 'top', offset: [0, -8] }).addTo(points)
      if (!showItinerary) return
      const row = document.createElement('li')
      const control = document.createElement('button')
      control.type = 'button'
      control.className = 'tirana-street-step'
      const title = document.createElement('span')
      title.textContent = kindLabels[info.kind] + (depot ? ' · ' + kindLabels.depot + ' ' + depot.id : '')
      if (detail) {
        const metrics = document.createElement('small')
        metrics.textContent = panel.dataset.serviceTime + ' ' + timeLabel(detail.serviceStartSeconds) + ' · ' +
          panel.dataset.load + ' ' + number.format(detail.loadGrams / 1000) + ' kg'
        title.append(metrics)
      }
      control.append(markerSymbol(kind, number.format(info.sequence), color), title)
      control.addEventListener('click', () => {
        map.panTo(location, { animate: false })
        marker.openTooltip()
      })
      row.append(control)
      stopList.append(row)
    }

    function refresh(fit = true) {
      context.clearLayers()
      selection.clearLayers()
      points.clearLayers()
      stopList.replaceChildren()
      const active = groups.find(group => group.id === select.value)
      const paths = []
      for (const group of groups) {
        pickers.get(group.id).setAttribute('aria-pressed', String(group === active))
        const coords = group.line.geometry.coordinates.map(toLatLng)
        const selected = group === active
        if (!active || selected) paths.push(...coords)
        const target = selected ? selection : context
        const width = selected ? 7 : active ? 3 : 5
        if (!active || selected) L.polyline(coords, { ...region(group.mode, selected ? 'tirana-selection' : 'overlayPane'), color: '#fff', opacity: 1, weight: width + 3 }).addTo(target)
        L.polyline(coords, { ...region(group.mode, selected ? 'tirana-selection' : 'overlayPane'), color: routeColor(group.id), opacity: active && !selected ? .32 : 1, weight: width }).addTo(target)
      }
      if (fit) map.fitBounds(paths, { padding: [28, 28], maxZoom: active ? 17 : 15, animate: false })
      if (active) {
        active.stops.forEach(item => addStop(item, active))
        const info = active.line.properties
        status.textContent = active.id + ' · ' + modeLabel[active.mode] + ' · '
          + panel.dataset.distance + ' ' + number.format(info.distanceMeters / 1000) + ' km · '
          + panel.dataset.travel + ' ' + number.format(info.durationSeconds / 60) + ' min'
      } else {
        for (const group of groups) for (const item of group.stops) addStop(item, group, false)
        status.textContent = panel.dataset.selectHint
      }
      itinerary.hidden = !active
      panel.dispatchEvent(new CustomEvent('tirana:courier-selected', { bubbles: true, detail: { courierId: active?.id ?? null } }))
    }
    panel.addEventListener('tirana:select-courier', event => {
      const courierId = event.detail?.courierId
      if (courierId !== null && !pickers.has(courierId)) return
      select.value = courierId ?? ''
      refresh()
    })
    select.addEventListener('change', () => refresh())
    controls.hidden = false
    legend.hidden = false
    select.value = groups.some(group => group.id === requestedCourier) ? requestedCourier : ''
    refreshLegend()
    refresh()
    void loadDispatch(groups).then(loaded => {
      if (!loaded) return
      refreshLegend()
      refresh(false)
    })
  }

  async function load() {
    if (panel.dataset.streetState !== 'waiting') return
    observer?.disconnect()
    panel.dataset.streetState = 'loading'
    status.textContent = panel.dataset.loading
    try {
      if (!L) throw new Error('Map library unavailable')
      const url = new URL(panel.dataset.routeUrl, document.baseURI)
      if (url.origin !== location.origin) throw new Error('Expected same-origin route artifact')
      const response = await fetch(url, { mode: 'same-origin', redirect: 'error' })
      if (!response.ok) throw new Error('Could not load routes')
      showMap(await response.json())
      panel.dataset.streetState = 'ready'
    } catch {
      map?.remove()
      map = undefined
      select.replaceChildren(select.options[0])
      chooser.replaceChildren()
      legend.replaceChildren()
      mapTarget.hidden = true
      controls.hidden = true
      legend.hidden = true
      itinerary.hidden = true
      panel.dataset.streetState = 'error'
      status.textContent = panel.dataset.error
    } finally {
      mapTarget.removeAttribute('aria-busy')
    }
  }

  const disclosure = panel.closest?.('details[data-fleet-secondary]')

  // Decision links work before, during, and after viewport-driven initialization.
  document.querySelectorAll('[data-select-courier]').forEach(link => link.addEventListener('click', event => {
    if (event.metaKey || event.ctrlKey || event.shiftKey || event.altKey) return
    const id = link.dataset.selectCourier
    if (!idPattern.test(id)) return
    requestedCourier = id
    if (disclosure) disclosure.open = true
    if (panel.dataset.streetState === 'ready')
      panel.dispatchEvent(new CustomEvent('tirana:select-courier', { detail: { courierId: id } }))
    else void load()
  }))

  mapTarget.setAttribute('aria-busy', 'true')
  mapTarget.hidden = false
  if (disclosure) {
    const reveal = () => {
      if (!disclosure.open) return
      if (map) map.invalidateSize()
      else void load()
    }
    disclosure.addEventListener('toggle', reveal)
    reveal()
  } else if (typeof IntersectionObserver === 'function') {
    observer = new IntersectionObserver(entries => {
      if (entries.some(entry => entry.isIntersecting)) return load()
    }, { rootMargin: '240px 0px' })
    observer.observe(mapTarget)
  } else void load()
})()
