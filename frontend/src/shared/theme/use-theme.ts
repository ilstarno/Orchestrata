import { useState } from 'react'

export type Theme = 'dark' | 'light'

export function useTheme() {
  const [theme, setTheme] = useState<Theme>('dark')
  const toggleTheme = () => setTheme(current => current === 'dark' ? 'light' : 'dark')

  return { theme, toggleTheme }
}
