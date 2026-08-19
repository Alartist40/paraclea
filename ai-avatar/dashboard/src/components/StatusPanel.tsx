import type { PipelineState } from '../App'

interface Props {
  state: PipelineState | null
}

export default function StatusPanel({ state }: Props) {
  const e = state?.emotion
  const localFps = state?.local_fps ?? 0
  const localCount = state?.local_detections?.length ?? 0

  return (
    <div className="rounded-lg border border-slate-800 bg-slate-900 p-4">
      <h2 className="text-sm font-semibold uppercase tracking-wider text-slate-400 mb-4">Pipeline State</h2>

      <div className="space-y-3">
        <div className="flex items-center justify-between">
          <span className="text-slate-300">STT</span>
          <span className="text-xs px-2 py-0.5 rounded bg-emerald-900 text-emerald-300">Active</span>
        </div>
        <div className="flex items-center justify-between">
          <span className="text-slate-300">TTS</span>
          <span className="text-xs px-2 py-0.5 rounded bg-emerald-900 text-emerald-300">Active</span>
        </div>
        <div className="flex items-center justify-between">
          <span className="text-slate-300">Speaking</span>
          <span className={`text-xs px-2 py-0.5 rounded ${state?.is_speaking ? 'bg-amber-900 text-amber-300' : 'bg-slate-800 text-slate-400'}`}>
            {state?.is_speaking ? 'Yes' : 'Idle'}
          </span>
        </div>
        <div className="flex items-center justify-between">
          <span className="text-slate-300">ESP32 FPS</span>
          <span className="text-xs font-mono text-slate-400">{state?.vision_fps?.toFixed(1) ?? '—'}</span>
        </div>
        <div className="flex items-center justify-between">
          <span className="text-slate-300">Local FPS</span>
          <span className="text-xs font-mono text-slate-400">{localFps > 0 ? localFps.toFixed(1) : '—'}</span>
        </div>
        <div className="flex items-center justify-between">
          <span className="text-slate-300">Objects</span>
          <span className="text-xs font-mono text-emerald-400">{localCount}</span>
        </div>
        <div className="flex items-center justify-between">
          <span className="text-slate-300">Bird tracks</span>
          <span className="text-xs font-mono text-amber-400">{(state?.bird_tracks || []).filter(t => t.status === 'active').length}</span>
        </div>
      </div>

      {e && (
        <div className="mt-4 pt-4 border-t border-slate-800">
          <h3 className="text-xs font-semibold uppercase tracking-wider text-slate-500 mb-2">Emotion</h3>
          <div className="grid grid-cols-2 gap-2 text-xs">
            <EmotionBar label="Joy" value={e.joy} color="bg-emerald-500" />
            <EmotionBar label="Sadness" value={e.sadness} color="bg-blue-500" />
            <EmotionBar label="Anger" value={e.anger} color="bg-rose-500" />
            <EmotionBar label="Fear" value={e.fear} color="bg-violet-500" />
            <EmotionBar label="Surprise" value={e.surprise} color="bg-amber-500" />
            <EmotionBar label="Trust" value={e.trust} color="bg-cyan-500" />
          </div>
        </div>
      )}

      <div className="mt-4 pt-4 border-t border-slate-800 space-y-2">
        <div>
          <span className="text-xs uppercase text-slate-500">Last user</span>
          <p className="text-sm text-slate-200 mt-0.5 line-clamp-2">{state?.last_user_text || '—'}</p>
        </div>
        <div>
          <span className="text-xs uppercase text-slate-500">Last AI</span>
          <p className="text-sm text-slate-200 mt-0.5 line-clamp-2">{state?.last_ai_text || '—'}</p>
        </div>
      </div>
    </div>
  )
}

function EmotionBar({ label, value, color }: { label: string; value: number; color: string }) {
  const pct = Math.round((value || 0) * 100)
  return (
    <div className="flex items-center gap-2">
      <span className="w-14 text-slate-400">{label}</span>
      <div className="flex-1 h-1.5 bg-slate-800 rounded-full overflow-hidden">
        <div className={`h-full ${color}`} style={{ width: `${pct}%` }} />
      </div>
      <span className="w-6 text-right text-slate-500">{pct}%</span>
    </div>
  )
}
