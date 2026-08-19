import { useState } from 'react'

export default function CommandBar() {
  const [busy, setBusy] = useState<string | null>(null)

  const send = async (command: string) => {
    setBusy(command)
    try {
      await fetch('/api/command', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ command }),
      })
    } catch {
      // ignore
    } finally {
      setBusy(null)
    }
  }

  const btn = (label: string, cmd: string, variant: 'neutral' | 'warn' | 'danger' = 'neutral') => {
    const colors = {
      neutral: 'bg-slate-700 hover:bg-slate-600',
      warn: 'bg-amber-900/60 hover:bg-amber-900 text-amber-200',
      danger: 'bg-rose-900/60 hover:bg-rose-900 text-rose-200',
    }
    return (
      <button
        key={cmd}
        onClick={() => send(cmd)}
        disabled={busy === cmd}
        className={`flex-1 text-xs px-3 py-2 rounded transition ${colors[variant]}`}
      >
        {busy === cmd ? '...' : label}
      </button>
    )
  }

  return (
    <div className="rounded-lg border border-slate-800 bg-slate-900 p-4">
      <h2 className="text-sm font-semibold uppercase tracking-wider text-slate-400 mb-3">Commands</h2>
      <div className="grid grid-cols-2 gap-2">
        {btn('Mute', 'mute')}
        {btn('Unmute', 'unmute')}
        {btn('Stop TTS', 'stop_tts', 'warn')}
        {btn('Shutdown', 'shutdown', 'danger')}
      </div>
    </div>
  )
}
