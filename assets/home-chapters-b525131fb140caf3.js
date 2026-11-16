// Navigation works as ordinary fragment links even when JavaScript is turned off.
const chapterNav = document.querySelector('[data-home-chapters]')

if (chapterNav) {
  const chapters = Array.from(chapterNav.querySelectorAll('a[data-chapter-link]'), link => ({
    link,
    section: document.getElementById(link.hash.slice(1)),
  })).filter(chapter => chapter.section)
  const header = document.querySelector('.site-header')
  let currentLink, currentSection
  let frame = null

  function updateCurrentChapter() {
    frame = null
    const line = Math.max(0, header?.getBoundingClientRect().bottom ?? 0) + 64
    let next = chapters[0]?.link
    let mostRecentTop = -Infinity

    for (const { link, section } of chapters) {
      const box = section.getBoundingClientRect()
      if (box.height && box.top <= line && box.top > mostRecentTop) {
        next = link
        mostRecentTop = box.top
      }
    }

    if (next !== currentLink) {
      currentLink?.removeAttribute('aria-current')
      next?.setAttribute('aria-current', 'location')
      currentSection?.removeAttribute('data-home-chapter-active')
      currentSection = chapters.find(chapter => chapter.link === next)?.section
      currentSection?.setAttribute('data-home-chapter-active', '')
      currentLink = next
    }
  }

  function requestUpdate() {
    if (frame === null) frame = requestAnimationFrame(updateCurrentChapter)
  }

  addEventListener('scroll', requestUpdate, { passive: true })
  addEventListener('resize', requestUpdate)
  addEventListener('hashchange', requestUpdate)
  addEventListener('pageshow', requestUpdate)
  if ('ResizeObserver' in window) {
    const observer = new ResizeObserver(requestUpdate)
    observer.observe(document.querySelector('.home-chapter-content') ?? document.body)
    if (header) observer.observe(header)
  }
  requestUpdate()
}

const queue = document.querySelector('.home-decision-hero')
const announcement = queue?.querySelector('.home-decision-announcement')

queue?.addEventListener('change', event => {
  const option = event.target
  if (!(option instanceof HTMLInputElement) || option.name !== 'home-decision' || !announcement) return
  const panel = document.getElementById(option.getAttribute('aria-controls') ?? '')
  if (!panel || !queue.contains(panel)) return
  const outcomes = Array.from(panel.querySelectorAll('dl > div'), item => {
    const label = item.querySelector('dt')?.textContent
    const count = item.querySelector('dd')?.textContent
    return label && count ? label + ': ' + count : ''
  }).filter(Boolean)
  announcement.textContent = outcomes.join('. ')
})
