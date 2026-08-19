import type { PipelineState } from '../App'

interface Props {
  state: PipelineState | null
}

export default function LocalVisionPanel({ state }: Props) {
  const detections = state?.local_detections || []
  const hasDetections = detections.length > 0
  const fps = state?.local_fps ?? 0

  return (
    <div className="rounded-lg border border-slate-800 bg-slate-900 p-4">
      <div className="flex items-center justify-between mb-4">
        <h2 className="text-sm font-semibold uppercase tracking-wider text-slate-400">Local Vision (YOLOv5)</h2>
        <span className={`text-xs px-2 py-0.5 rounded ${fps > 0 ? 'bg-emerald-900 text-emerald-300' : 'bg-slate-800 text-slate-500'}`}>
          {fps > 0 ? `${fps.toFixed(0)} FPS` : 'Standby'}
        </span>
      </div>

      {/* Detection canvas */}
      <div className="relative aspect-video bg-slate-950 rounded border border-slate-800 overflow-hidden">
        {!hasDetections && (
          <div className="absolute inset-0 flex items-center justify-center text-slate-600 text-sm">
            {fps > 0 ? 'No objects detected' : 'Camera offline / stub mode'}
          </div>
        )}
        {hasDetections && (
          <svg className="absolute inset-0 w-full h-full" viewBox="0 0 640 480" preserveAspectRatio="none">
            {detections.map((d, i) => (
              <g key={i}>
                <rect
                  x={d.bbox[0]}
                  y={d.bbox[1]}
                  width={d.bbox[2]}
                  height={d.bbox[3]}
                  fill="none"
                  stroke={d.class_name === 'face' ? '#f472b6' : '#34d399'}
                  strokeWidth="2"
                  rx="2"
                />
                <text
                  x={d.bbox[0] + 2}
                  y={d.bbox[1] + 14}
                  fill={d.class_name === 'face' ? '#f472b6' : '#34d399'}
                  fontSize="12"
                  fontFamily="monospace"
                  fontWeight="bold"
                >
                  {d.class_name} {(d.confidence * 100).toFixed(0)}%
                </text>
              </g>
            ))}
          </svg>
        )}
      </div>

      {/* Detection list */}
      <div className="mt-3 space-y-1 max-h-40 overflow-y-auto">
        {detections.map((d, i) => (
          <div key={i} className="flex items-center justify-between text-xs px-2 py-1 rounded bg-slate-800/50">
            <span className="flex items-center gap-2">
              <span className={`w-2 h-2 rounded-full ${d.class_name === 'face' ? 'bg-pink-400' : 'bg-emerald-400'}`} />
              <span className="text-slate-200">{d.class_name}</span>
            </span>
            <span className="text-emerald-400 font-mono">{(d.confidence * 100).toFixed(0)}%</span>
          </div>
        ))}
        {detections.length === 0 && fps > 0 && (
          <div className="text-xs text-slate-600 text-center py-2">Scanning for objects...</div>
        )}
      </div>
    </div>
  )
}
