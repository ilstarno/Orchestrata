const demo = document.querySelector('.home-focus-demo')
const status = demo?.querySelector('[data-dispatch-announcement]')
if (demo && status) {
  demo.addEventListener('change', event => {
    const control = event.target
    if (!(control instanceof HTMLInputElement) || control.name !== 'dispatch-replay' || !control.checked) return
    const panel = document.getElementById(control.getAttribute('aria-controls') || '')
    if (!panel) return
    const heading = panel.querySelector('h3')?.textContent?.trim() || ''
    const outcome = panel.querySelector('.home-focus-outcome')?.textContent?.trim() || ''
    status.textContent = [heading, outcome].filter(Boolean).join('. ')
  })
}
