const API_BASE = import.meta.env.VITE_API_BASE || '/api/v1';

export interface PersonaFragment {
  soul: string;
  identity: string;
  memory: string;
  user: string;
  heartbeat: string;
  tools: string;
  agents: string;
}

export interface BotStatus {
  stt_engine: string;
  tts_engine: string;
  vad_running: boolean;
  vision_running: boolean;
  wakeword_running: boolean;
  persona_count: number;
}

export async function getStatus(): Promise<BotStatus> {
  const res = await fetch(`${API_BASE}/status`);
  if (!res.ok) throw new Error(await res.text());
  return res.json();
}

export async function getPersona(botID: string): Promise<PersonaFragment> {
  const res = await fetch(`${API_BASE}/persona/${botID}`);
  if (!res.ok) throw new Error(await res.text());
  return res.json();
}

export async function savePersonaFragment(
  botID: string,
  fragment: string,
  content: string
): Promise<void> {
  const res = await fetch(`${API_BASE}/persona/${botID}/${fragment}`, {
    method: 'POST',
    headers: { 'Content-Type': 'text/plain' },
    body: content,
  });
  if (!res.ok) throw new Error(await res.text());
}

export async function command(action: 'mute' | 'unmute' | 'stop_tts' | 'shutdown'): Promise<void> {
  const res = await fetch(`${API_BASE}/command`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ action }),
  });
  if (!res.ok) throw new Error(await res.text());
}
