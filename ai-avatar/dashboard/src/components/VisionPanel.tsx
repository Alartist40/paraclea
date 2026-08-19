import type { PipelineState } from '../App'

interface Props {
  state: PipelineState | null
}

export default function VisionPanel({ state }: Props) {
  const detections = state?.vision_detections || []
  const hasDetections = detections.length > 0

  return (
    <div className="rounded-lg border border-slate-800 bg-slate-900 p-4">
      <div className="flex items-center justify-between mb-4">
        <h2 className="text-sm font-semibold uppercase tracking-wider text-slate-400">Vision AI</h2>
        <span className={`text-xs px-2 py-0.5 rounded ${state?.grove_present ? 'bg-emerald-900 text-emerald-300' : 'bg-rose-900 text-rose-300'}`}>
          {state?.grove_present ? 'Grove online' : 'Offline'}
        </span>
      </div>

      {/* Detection canvas visualization */}
      <div className="relative aspect-video bg-slate-950 rounded border border-slate-800 overflow-hidden">
        {!hasDetections && (
          <div className="absolute inset-0 flex items-center justify-center text-slate-600 text-sm">
            No detections
          </div>
        )}
        {hasDetections && (
          <svg className="absolute inset-0 w-full h-full" viewBox="0 0 320 240" preserveAspectRatio="none">
            {detections.map((d, i) => (
              <g key={i}>
                <rect
                  x={d.x}
                  y={d.y}
                  width={d.width}
                  height={d.height}
                  fill="none"
                  stroke="#34d399"
                  strokeWidth="1"
                  rx="2"
                />
                <text
                  x={d.x + 2}
                  y={d.y + 12}
                  fill="#34d399"
                  fontSize="10"
                  fontFamily="monospace"
                >
                  {d.label} {Math.round(d.score * 100)}%
                </text>
              </g>
            ))}
          </svg>
        )}
      </div>

      {/* Detection list */}
      <div className="mt-3 space-y-1 max-h-32 overflow-y-auto">
        {detections.map((d, i) => (
          <div key={i} className="flex items-center justify-between text-xs px-2 py-1 rounded bg-slate-800/50">
            <span className="text-slate-200">{d.label}</span>
            <span className="text-emerald-400 font-mono">{(d.score * 100).toFixed(0)}%</span>
          </div>
        ))}
        {detections.length === 0 && (
          <div className="text-xs text-slate-600 text-center py-2">Waiting for objects...</div>
        )}
      </div>
    </div>
  )
}
