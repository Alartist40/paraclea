import { useEffect, useRef } from 'react'

interface Props {
  logs: string[]
}

export default function LogView({ logs }: Props) {
  const bottomRef = useRef<HTMLDivElement>(null)

  useEffect(() => {
    bottomRef.current?.scrollIntoView({ behavior: 'smooth' })
  }, [logs])

  return (
    <div className="rounded-lg border border-slate-800 bg-slate-900 p-4 h-full flex flex-col">
      <h2 className="text-sm font-semibold uppercase tracking-wider text-slate-400 mb-3">Log</h2>
      <div className="flex-1 overflow-y-auto font-mono text-xs space-y-1 max-h-[70vh]">
        {logs.map((line, i) => (
          <div key={i} className="text-slate-400">
            {line}
          </div>
        ))}
        <div ref={bottomRef} />
      </div>
    </div>
  )
}
