import { useEffect, useRef, useState } from 'react'
import StatusPanel from './components/StatusPanel'
import PersonaEditor from './components/PersonaEditor'
import VisionPanel from './components/VisionPanel'
import LocalVisionPanel from './components/LocalVisionPanel'
import BirdWatchPanel from './components/BirdWatchPanel'
import LogView from './components/LogView'
import CommandBar from './components/CommandBar'

export interface EmotionState {
  joy: number
  sadness: number
  anger: number
  fear: number
  surprise: number
  trust: number
  mouth_smile: number
}

export interface Detection {
  label: string
  score: number
  x: number
  y: number
  width: number
  height: number
}

export interface LocalDetection {
  class_id: number
  class_name: string
  confidence: number
  bbox: [number, number, number, number]
}

export interface BirdTrack {
  track_id: number
  species: string
  confidence: number
  centroid_x: number
  centroid_y: number
  x: number
  y: number
  w: number
  h: number
  frame_count: number
  status: string
}

export interface PipelineState {
  is_speaking: boolean
  last_user_text: string
  last_ai_text: string
  emotion: EmotionState
  vision_detections?: Detection[]
  vision_fps?: number
  grove_present?: boolean
  local_detections?: LocalDetection[]
  local_fps?: number
  bird_tracks?: BirdTrack[]
}

function App() {
  const [connected, setConnected] = useState(false)
  const [state, setState] = useState<PipelineState | null>(null)
  const [logs, setLogs] = useState<string[]>([])
  const wsRef = useRef<WebSocket | null>(null)

  useEffect(() => {
    const connect = () => {
      const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:'
      const ws = new WebSocket(`${protocol}//${window.location.host}/ws`)
      wsRef.current = ws

      ws.onopen = () => {
        setConnected(true)
        addLog('WebSocket connected')
      }

      ws.onclose = () => {
        setConnected(false)
        addLog('WebSocket disconnected — reconnecting in 2s...')
        setTimeout(connect, 2000)
      }

      ws.onerror = (e) => {
        addLog(`WebSocket error: ${e.type}`)
      }

      ws.onmessage = (ev) => {
        try {
          const msg = JSON.parse(ev.data)
          if (msg.type === 'state') {
            setState(msg.data)
          } else if (msg.type === 'pong') {
            // heartbeat
          }
        } catch {
          // ignore non-JSON
        }
      }
    }

    connect()
    const ping = setInterval(() => {
      wsRef.current?.send(JSON.stringify({ type: 'ping' }))
    }, 5000)

    return () => {
      clearInterval(ping)
      wsRef.current?.close()
    }
  }, [])

  const addLog = (line: string) => {
    const ts = new Date().toLocaleTimeString()
    setLogs((prev) => [...prev.slice(-199), `[${ts}] ${line}`])
  }

  return (
    <div className="min-h-screen bg-slate-950 text-slate-100">
      <header className="border-b border-slate-800 bg-slate-900 px-6 py-4 flex items-center justify-between">
        <div className="flex items-center gap-3">
          <div className={`w-3 h-3 rounded-full ${connected ? 'bg-emerald-400 animate-pulse' : 'bg-rose-500'}`} />
          <h1 className="text-xl font-bold tracking-tight">Paraclea Dashboard</h1>
        </div>
        <div className="text-sm text-slate-400 flex gap-4">
          <span>{state?.grove_present ? '📷 ESP32 Vision' : 'No ESP32'}</span>
          <span>{(state?.local_fps ?? 0) > 0 ? '🎥 Local Vision' : 'Local cam off'}</span>
        </div>
      </header>

      <main className="p-6 grid grid-cols-12 gap-6">
        {/* Status + Commands */}
        <div className="col-span-12 lg:col-span-3 space-y-6">
          <StatusPanel state={state} />
          <CommandBar />
          <BirdWatchPanel tracks={state?.bird_tracks} />
        </div>

        {/* Vision panels */}
        <div className="col-span-12 lg:col-span-5 space-y-6">
          <VisionPanel state={state} />
          <LocalVisionPanel state={state} />
        </div>

        {/* Persona + Logs */}
        <div className="col-span-12 lg:col-span-4 space-y-6">
          <PersonaEditor />
          <LogView logs={logs} />
        </div>
      </main>
    </div>
  )
}

export default App
