(() => {
  const key = 'orchestrata-theme'
  const system = matchMedia('(prefers-color-scheme: dark)')
  const root = document.documentElement
  function readTheme() {
    try {
      const stored = localStorage.getItem(key)
      if (stored === 'light' || stored === 'dark') return stored
    } catch {}
    return 'system'
  }
  let theme = readTheme()
  function render() {
    if (theme === 'system') delete root.dataset.theme
    else root.dataset.theme = theme
    const dark = theme === 'dark' || (theme === 'system' && system.matches)
    document.querySelector('meta[name="theme-color"]')?.setAttribute('content', dark ? '#111816' : '#ffffff')
  }
  render()
  system.addEventListener('change', render)
  document.addEventListener('DOMContentLoaded', () => {
    const setting = document.getElementById('try-theme-setting')
    if (!setting) return
    setting.value = theme
    setting.parentElement.hidden = false
    setting.addEventListener('change', () => {
      theme = setting.value
      render()
      try { localStorage.setItem(key, theme) } catch {}
    })
    window.addEventListener('storage', event => {
      if (event.key !== null && event.key !== key) return
      theme = readTheme()
      setting.value = theme
      render()
    })
  }, { once: true })
})()
