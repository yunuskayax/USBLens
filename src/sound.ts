import type { Lang } from "./types";

let ctx: AudioContext | null = null;

function audio(): AudioContext | null {
  try {
    ctx ??= new AudioContext();
    if (ctx.state === "suspended") void ctx.resume();
    return ctx;
  } catch {
    return null;
  }
}

/** Short two-note chime: rising for arrival, falling for removal. */
export function chime(arrival: boolean, volume: number) {
  const ac = audio();
  if (!ac || volume <= 0) return;
  const notes = arrival ? [659.25, 987.77] : [987.77, 587.33]; // E5→B5 / B5→D5
  const gain = (volume / 100) * 0.25;
  notes.forEach((freq, i) => {
    const t = ac.currentTime + i * 0.13;
    const osc = ac.createOscillator();
    const g = ac.createGain();
    osc.type = "sine";
    osc.frequency.value = freq;
    g.gain.setValueAtTime(0, t);
    g.gain.linearRampToValueAtTime(gain, t + 0.015);
    g.gain.exponentialRampToValueAtTime(0.0001, t + 0.35);
    osc.connect(g).connect(ac.destination);
    osc.start(t);
    osc.stop(t + 0.4);
  });
}

/** Read the event out loud with the system's text-to-speech voice. */
export function speak(text: string, lang: Lang, volume: number) {
  if (!("speechSynthesis" in window) || volume <= 0) return;
  const u = new SpeechSynthesisUtterance(text);
  u.lang = lang === "tr" ? "tr-TR" : "en-US";
  u.volume = volume / 100;
  const voice = speechSynthesis.getVoices().find((v) => v.lang.toLowerCase().startsWith(lang));
  if (voice) u.voice = voice;
  speechSynthesis.cancel();
  speechSynthesis.speak(u);
}

export function playUsbSound(opts: {
  arrival: boolean;
  style: "chime" | "voice";
  volume: number;
  lang: Lang;
  name: string;
  storage: boolean;
}) {
  if (opts.style === "voice") {
    const what = opts.storage ? (opts.lang === "tr" ? "USB depolama aygıtı" : "USB storage device") : opts.name;
    const verb = opts.lang === "tr" ? (opts.arrival ? "takıldı" : "çıkarıldı") : opts.arrival ? "connected" : "disconnected";
    const text = opts.storage && opts.name ? `${what}: ${opts.name} ${verb}` : `${what} ${verb}`;
    speak(text, opts.lang, opts.volume);
  } else {
    chime(opts.arrival, opts.volume);
  }
}
