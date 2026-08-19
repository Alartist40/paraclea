import { useEffect, useState } from 'react'

const FILES = ['SOUL.md', 'IDENTITY.md', 'USER.md', 'MEMORY.md', 'TOOLS.md', 'HEARTBEAT.md', 'AGENTS.md']
const DEFAULT_BOT = 'default'

export default function PersonaEditor() {
  const [activeFile, setActiveFile] = useState(FILES[0])
  const [content, setContent] = useState('')
  const [saving, setSaving] = useState(false)
  const [saved, setSaved] = useState(false)

  useEffect(() => {
    fetch(`/api/persona?bot=${DEFAULT_BOT}`)
      .then((r) => r.json())
      .then((data) => setContent(data[activeFile] || ''))
      .catch(() => setContent(''))
  }, [activeFile])

  const save = async () => {
    setSaving(true)
    try {
      const res = await fetch('/api/persona', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ bot_id: DEFAULT_BOT, files: { [activeFile]: content } }),
      })
      if (res.ok) {
        setSaved(true)
        setTimeout(() => setSaved(false), 1500)
      }
    } finally {
      setSaving(false)
    }
  }

  return (
    <div className="rounded-lg border border-slate-800 bg-slate-900 p-4">
      <div className="flex items-center justify-between mb-3">
        <h2 className="text-sm font-semibold uppercase tracking-wider text-slate-400">Persona</h2>
        <button
          onClick={save}
          disabled={saving}
          className={`text-xs px-3 py-1 rounded transition ${saved ? 'bg-emerald-700 text-emerald-100' : 'bg-slate-700 hover:bg-slate-600 text-slate-200'}`}
        >
          {saving ? 'Saving...' : saved ? 'Saved!' : 'Save'}
        </button>
      </div>

      <div className="flex gap-1 mb-2 overflow-x-auto">
        {FILES.map((f) => (
          <button
            key={f}
            onClick={() => setActiveFile(f)}
            className={`text-xs px-2 py-1 rounded whitespace-nowrap ${
              activeFile === f ? 'bg-slate-700 text-slate-100' : 'text-slate-500 hover:text-slate-300'
            }`}
          >
            {f.replace('.md', '')}
          </button>
        ))}
      </div>

      <textarea
        value={content}
        onChange={(e) => setContent(e.target.value)}
        className="w-full h-48 bg-slate-950 border border-slate-800 rounded p-3 text-sm text-slate-200 font-mono resize-none focus:outline-none focus:border-slate-600"
        spellCheck={false}
      />
    </div>
  )
}
