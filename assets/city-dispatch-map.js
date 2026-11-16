(() => {
  'use strict'
  const L = window.L
  if (!L) return
  const isIndex = (value, length) => Number.isSafeInteger(value) && value >= 0 && value < length
  const point = value => Array.isArray(value) && value.length === 2 && value.every(Number.isFinite) &&
    Math.abs(value[0]) <= 180 && Math.abs(value[1]) <= 90
  const latLng = p => [p[1], p[0]]
  const id = (index) => 'STOP-' + String(index + 1).padStart(5, '0')
  const courierColor = index => 'hsl(' + ((index * 137.508) % 360).toFixed(2) + ' 74% 35%)'
  const heldColor = '#9a430a'
  function parse(workload, geo) {
    const { nodes, couriers, stops, summary } = workload ?? {}
    if (!Array.isArray(nodes) || !nodes.length || nodes.length > 110_000 || !nodes.every(point) ||
      !Array.isArray(couriers) || !couriers.length || couriers.length > 250 ||
      !Array.isArray(stops) || !stops.length || stops.length > 75_000) throw Error('Invalid workload')
    const known = new Map()
    for (const [index, courier] of couriers.entries()) {
      if (typeof courier?.courierId !== 'string' || !/^[A-Z]{2,8}-\d{1,4}$/.test(courier.courierId) ||
        known.has(courier.courierId) || !Number.isSafeInteger(courier.assignedCount) || courier.assignedCount < 0)
        throw Error('Invalid courier')
      known.set(courier.courierId, index)
    }
    const byCourier = new Uint32Array(couriers.length)
    let upstreamAssigned = 0
    for (const row of stops) {
      if (!Array.isArray(row) || row.length !== 5 || (row[0] !== null && !isIndex(row[0], nodes.length))) throw Error('Invalid stop')
      if (row[1] === null) {
        if (row[2] !== null || row[3] !== null || typeof row[4] !== 'string') throw Error('Invalid held stop')
      } else {
        if (!isIndex(row[1], couriers.length) || !Number.isSafeInteger(row[2]) || row[2] < 0 ||
          !Number.isFinite(row[3]) || row[3] < 0 || row[4] !== null) throw Error('Invalid assigned stop')
        byCourier[row[1]]++
        upstreamAssigned++
      }
    }
    if (summary?.requestedCount !== stops.length || summary.assignedCount !== upstreamAssigned ||
      summary.heldCount !== stops.length - upstreamAssigned ||
      couriers.some((c, i) => c.assignedCount !== byCourier[i])) throw Error('Inconsistent workload counts')

    if (geo?.type !== 'FeatureCollection' || !Array.isArray(geo.features) || geo.features.length > couriers.length) throw Error('Invalid routes')
    const lines = new Map()
    let vertices = 0
    for (const feature of geo.features) {
      const index = known.get(feature?.properties?.courierId)
      const coords = feature?.geometry?.coordinates
      if (feature?.type !== 'Feature' || index === undefined || lines.has(index) ||
        feature.geometry?.type !== 'LineString' || !Array.isArray(coords) || coords.length < 2 ||
        (vertices += coords.length) > 750_000 || !coords.every(point)) throw Error('Invalid route')
      lines.set(index, coords.map(latLng))
    }
    return { nodes: nodes.map(latLng), couriers, stops, lines }
  }

  async function getLocal(locationPath, extension, max) {
    const url = new URL(locationPath, document.baseURI)
    if (url.origin !== location.origin || !url.pathname.endsWith(extension)) throw Error('Expected local workload')
    const response = await fetch(url, { redirect: 'error', mode: 'same-origin' })
    if (!response.ok || !response.body || Number(response.headers.get('content-length')) > max) throw Error('Workload unavailable')
    const reader = response.body.getReader(), decoder = new TextDecoder('utf-8', { fatal: true })
    try {
      let bytes = 0, chunks = []
      while (true) {
        const next = await reader.read()
        if (next.done) break
        bytes += next.value.byteLength
        if (bytes > max) throw Error('Workload too large')
        chunks.push(decoder.decode(next.value, { stream: true }))
      }
      return JSON.parse(chunks.join('') + decoder.decode())
    } finally { await reader.cancel().catch(() => {}) }
  }

  function selectStops(data, status, courier, query) {
    const matching = []
    let held = 0, assigned = 0, unmapped = 0
    for (const [index, row] of data.stops.entries()) {
      const isHeld = row[1] === null
      if ((status === 'assigned' && isHeld) || (status === 'held' && !isHeld) ||
        (courier !== null && row[1] !== courier) ||
        (query && !id(index).includes(query))) continue
      if (isHeld) held++; else assigned++
      if (row[0] === null) unmapped++
      matching.push(index)
    }
    return { matching, held, assigned, unmapped }
  }

  function initialize(panel) {
    if (panel.dataset.loaded) return
    const q = selector => panel.querySelector(selector)
    const button = q('[data-open]'), mapNode = q('[data-map]'), toolbar = q('[data-toolbar]')
    const summary = q('[data-summary]'), detail = q('[data-detail]'), labels = q('[data-labels]')?.dataset
    const filter = q('[data-filter]'), courier = q('[data-courier]'), search = q('[data-search]'), fit = q('[data-fit]')
    if (!button || !mapNode || !toolbar || !summary || !detail || !labels || !filter || !courier || !search || !fit) return
    panel.dataset.loaded = 'true'
    button.hidden = false
    const number = new Intl.NumberFormat(panel.dataset.locale)
    button.addEventListener('click', async () => {
      button.disabled = true
      summary.textContent = labels.load
      let map
      try {
        const [workload, geo] = await Promise.all([
          getLocal(panel.dataset.workload, '.json', 8_000_000), getLocal(panel.dataset.routes, '.geojson', 24_000_000),
        ])
        const data = parse(workload, geo)
        mapNode.hidden = false
        map = L.map(mapNode, { scrollWheelZoom: false, minZoom: 9, maxZoom: 18, preferCanvas: true })
        L.tileLayer('https://tile.openstreetmap.org/{z}/{x}/{y}.png', {
          maxZoom: 18, attribution: '&copy; <a href="https://www.openstreetmap.org/copyright">OpenStreetMap contributors</a>',
        }).addTo(map)
        const layers = L.layerGroup().addTo(map)
        const lines = new Map()
        const colors = data.couriers.map((_, index) => courierColor(index))
        for (const [index, coords] of data.lines) {
          const mode = data.couriers[index].mode
          lines.set(index, L.polyline(coords, { color: colors[index], opacity: .54, weight: 2, interactive: false,
            dashArray: mode === 'walking' ? '2 6' : mode === 'bicycle' ? '6 5' : undefined }))
        }
        const canvas = document.createElement('canvas')
        canvas.className = 'city-dispatch__points'
        canvas.setAttribute('aria-hidden', 'true')
        mapNode.append(canvas)
        const ctx = canvas.getContext('2d')
        if (!ctx) throw Error('Canvas unavailable')
        for (const [index, row] of data.couriers.entries()) {
          const option = document.createElement('option')
          option.value = String(index)
          option.textContent = row.courierId + ' · ' + number.format(row.assignedCount)
          courier.append(option)
        }

        let frame = 0, timer = 0, result, selected = null, query = ''
        const draw = () => {
          frame = 0
          if (!result) return
          const size = map.getSize(), scale = Math.min(window.devicePixelRatio || 1, 2)
          const w = Math.round(size.x * scale), h = Math.round(size.y * scale)
          if (canvas.width !== w || canvas.height !== h) { canvas.width = w; canvas.height = h }
          ctx.setTransform(scale, 0, 0, scale, 0, 0)
          ctx.clearRect(0, 0, size.x, size.y)
          const radius = query ? 5 : selected !== null ? 3 : map.getZoom() >= 15 ? 2 : 1.35
          for (const index of result.matching) {
            const row = data.stops[index]
            if (row[0] === null) continue
            const p = map.latLngToContainerPoint(data.nodes[row[0]])
            if (p.x < -6 || p.y < -6 || p.x > size.x + 6 || p.y > size.y + 6) continue
            if (row[1] === null) {
              ctx.strokeStyle = heldColor
              ctx.lineWidth = radius > 2 ? 2 : 1
              ctx.strokeRect(p.x - radius, p.y - radius, 2 * radius, 2 * radius)
            } else {
              ctx.fillStyle = colors[row[1]]
              ctx.beginPath()
              ctx.arc(p.x, p.y, radius, 0, 2 * Math.PI)
              ctx.fill()
            }
          }
        }
        const queueDraw = () => { if (!frame) frame = requestAnimationFrame(draw) }
        const fitCurrent = () => {
          const b = L.latLngBounds([])
          for (const index of result.matching) {
            const node = data.stops[index][0]
            if (node !== null) b.extend(data.nodes[node])
          }
          if (b.isValid()) map.fitBounds(b, { padding: [24, 24], maxZoom: query ? 17 : 14, animate: false })
        }
        const refresh = (refit = false) => {
          selected = courier.value === '' ? null : Number(courier.value)
          query = search.value.trim().toUpperCase()
          result = selectStops(data, filter.value, selected, query)
          layers.clearLayers()
          if (filter.value !== 'held' && !query)
            for (const [index, line] of lines) if (selected === null || selected === index) {
              line.setStyle({ opacity: selected === null ? .54 : .9, weight: selected === null ? 2 : 4 })
              layers.addLayer(line)
            }
          const f = value => number.format(value)
          summary.textContent = labels.total + ': ' + f(data.stops.length) + ' · ' + labels.visible + ': ' +
            f(result.matching.length - result.unmapped) + ' · ' + labels.assigned + ': ' + f(result.assigned) + ' · ' +
            labels.held + ': ' + f(result.held) + (result.unmapped ? ' · ' + labels.noLocation + ': ' + f(result.unmapped) : '')
          detail.textContent = !result.matching.length ? labels.noMatch : query && result.matching.length === 1 ?
            id(result.matching[0]) + ' · ' + (data.stops[result.matching[0]][1] === null ? labels.held :
              data.couriers[data.stops[result.matching[0]][1]].courierId) : labels.notice
          if (refit) fitCurrent()
          queueDraw()
        }
        map.setView(data.nodes[0], 12, { animate: false })
        map.on('move zoom resize', queueDraw)
        search.addEventListener('input', () => { clearTimeout(timer); timer = setTimeout(() => refresh(true), 130) })
        filter.addEventListener('change', () => { if (filter.value === 'held') courier.value = ''; refresh(true) })
        courier.addEventListener('change', () => { if (courier.value && filter.value === 'held') filter.value = 'assigned'; refresh(true) })
        fit.addEventListener('click', fitCurrent)
        refresh(true)
        toolbar.hidden = false
        button.hidden = true
        filter.focus()
      } catch {
        map?.remove()
        mapNode.querySelector('canvas.city-dispatch__points')?.remove()
        mapNode.hidden = true
        toolbar.hidden = true
        courier.replaceChildren(courier.options[0])
        button.disabled = false
        summary.textContent = labels.error
      }
    })
  }
  document.querySelectorAll('[data-city-dispatch]').forEach(initialize)
})()
