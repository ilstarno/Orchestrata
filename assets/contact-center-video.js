const video = document.querySelector('[data-ringcx-video]')
const launcher = video?.querySelector('.cc-video-play')
launcher?.addEventListener('click', event => {
  if (event.button !== 0 || event.metaKey || event.ctrlKey || event.shiftKey || event.altKey || video.querySelector('iframe')) return
  event.preventDefault()
  const player = document.createElement('iframe')
  player.src = 'https://www.youtube-nocookie.com/embed/pyfD6IiahsA'
  player.title = launcher.textContent.trim()
  player.allowFullscreen = true
  player.referrerPolicy = 'strict-origin-when-cross-origin'
  video.prepend(player)
  player.focus()
}, { passive: false })
