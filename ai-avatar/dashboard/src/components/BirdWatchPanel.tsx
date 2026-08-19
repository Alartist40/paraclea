import { useEffect, useState } from 'react'
import type { BirdTrack } from '../App'

interface Props {
  tracks: BirdTrack[] | undefined
}

interface Sighting {
  id: string
  species: string
  confidence: number
  timestamp: string
  bbox_x: number
  bbox_y: number
  bbox_w: number
  bbox_h: number
}

export default function BirdWatchPanel({ tracks }: Props) {
  const [sightings, setSightings] = useState<Sighting[]>([])
  const [species, setSpecies] = useState<string[]>([])

  useEffect(() => {
    fetch('/api/birdwatch/sightings?limit=10')
      .then((r) => r.json())
      .then((data) => setSightings(Array.isArray(data) ? data : []))
      .catch(() => setSightings([]))

    fetch('/api/birdwatch/species')
      .then((r) => r.json())
      .then((data) => setSpecies(Array.isArray(data) ? data : []))
      .catch(() => setSpecies([]))
  }, [tracks])

  const activeTracks = (tracks || []).filter((t) => t.status === 'active')

  return (
    <div className="rounded-lg border border-slate-800 bg-slate-900 p-4">
      <div className="flex items-center justify-between mb-3">
        <h2 className="text-sm font-semibold uppercase tracking-wider text-slate-400">Birdwatch</h2>
        <span className="text-xs px-2 py-0.5 rounded bg-slate-800 text-slate-400">
          {species.length} species
        </span>
      </div>

      {/* Active tracks */}
      {activeTracks.length > 0 && (
        <div className="mb-3 space-y-1">
          <div className="text-xs uppercase text-slate-500 mb-1">Active tracks</div>
          {activeTracks.map((t) => (
            <div key={t.track_id} className="flex items-center justify-between text-xs px-2 py-1 rounded bg-slate-800/50">
              <span className="text-slate-200">🐦 {t.species} #{t.track_id}</span>
              <span className="text-amber-400 font-mono">{(t.confidence * 100).toFixed(0)}% · {t.frame_count}fr</span>
            </div>
          ))}
        </div>
      )}

      {/* Recent sightings */}
      <div className="space-y-1 max-h-40 overflow-y-auto">
        <div className="text-xs uppercase text-slate-500 mb-1">Recent sightings</div>
        {sightings.map((s) => (
          <div key={s.id} className="flex items-center justify-between text-xs px-2 py-1 rounded bg-slate-800/30">
            <span className="text-slate-300">{s.species}</span>
            <span className="text-slate-500 font-mono">{new Date(s.timestamp).toLocaleTimeString()}</span>
          </div>
        ))}
        {sightings.length === 0 && (
          <div className="text-xs text-slate-600 text-center py-2">No birds logged yet</div>
        )}
      </div>
    </div>
  )
}
