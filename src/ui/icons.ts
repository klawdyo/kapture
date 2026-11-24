// Catálogo de ícones (viewBox 24x24). f = preenchido; senão só contorno.
export const ICONS = {
  screen: { f: false, d: 'M3 5h18v11H3zM8 20h8M12 16v4' },
  window: { f: false, d: 'M3 5h18v14H3zM3 9h18' },
  area: { f: false, d: 'M4 8V4h4M16 4h4v4M20 16v4h-4M8 20H4v-4' },
  cam: { f: false, d: 'M3 7h12v10H3zM15 11l6-3v8l-6-3' },
  mic: { f: false, d: 'M12 3a3 3 0 013 3v6a3 3 0 01-6 0V6a3 3 0 013-3zM5 11a7 7 0 0014 0M12 18v3' },
  sys: { f: false, d: 'M4 9h4l5-4v14l-5-4H4zM16 9a4 4 0 010 6' },
  gear: { f: false, d: 'M12.2 2h-.4a2 2 0 00-2 2v.2a2 2 0 01-1 1.7l-.4.3a2 2 0 01-2 0l-.2-.1a2 2 0 00-2.7.7l-.2.4a2 2 0 00.7 2.7l.2.1a2 2 0 011 1.7v.5a2 2 0 01-1 1.7l-.2.1a2 2 0 00-.7 2.7l.2.4a2 2 0 002.7.7l.2-.1a2 2 0 012 0l.4.3a2 2 0 011 1.7V20a2 2 0 002 2h.4a2 2 0 002-2v-.2a2 2 0 011-1.7l.4-.3a2 2 0 012 0l.2.1a2 2 0 002.7-.7l.2-.4a2 2 0 00-.7-2.7l-.2-.1a2 2 0 01-1-1.7v-.5a2 2 0 011-1.7l.2-.1a2 2 0 00.7-2.7l-.2-.4a2 2 0 00-2.7-.7l-.2.1a2 2 0 01-2 0l-.4-.3a2 2 0 01-1-1.7V4a2 2 0 00-2-2zM12 9a3 3 0 100 6 3 3 0 000-6z' },
  min: { f: false, d: 'M6 12h12' },
  close: { f: false, d: 'M6 6l12 12M18 6L6 18' },
  chev: { f: false, d: 'M7 10l5 5 5-5' },
  rec: { f: true, d: 'M12 6a6 6 0 100 12 6 6 0 000-12z' },
  stop: { f: true, d: 'M7 7h10v10H7z' },
  pause: { f: true, d: 'M8 6h3v12H8zM13 6h3v12h-3z' },
  play: { f: true, d: 'M8 5l11 7-11 7z' },
} as const

/** Nome de ícone válido. */
export type IconName = keyof typeof ICONS

/** Opção de lista com ícone opcional. */
export interface UiOpt { value: string; label: string; icon?: IconName }
