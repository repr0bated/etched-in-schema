<script lang="ts">
  import { onMount } from "svelte";
  import type { ChatSession } from "./chat-session";

  // The subsidiary's embodiment: speaks each agent reply and takes spoken
  // questions. Uses only the browser's speech APIs, so it needs no extra
  // service or key; replies still come through the receipted chat path.
  let { session, name = "3tched · Hackathon" }: { session: ChatSession; name?: string } = $props();

  type Recognition = {
    lang: string; interimResults: boolean; continuous: boolean;
    onresult: ((event: { results: ArrayLike<ArrayLike<{ transcript: string }>> }) => void) | null;
    onend: (() => void) | null; onerror: (() => void) | null;
    start(): void; stop(): void;
  };
  const speechOut = typeof window !== "undefined" && "speechSynthesis" in window;
  const RecognitionCtor: (new () => Recognition) | undefined = typeof window !== "undefined"
    ? ((window as unknown as Record<string, new () => Recognition>).SpeechRecognition
      ?? (window as unknown as Record<string, new () => Recognition>).webkitSpeechRecognition)
    : undefined;

  let voiceOn = $state(speechOut);
  let speaking = $state(false);
  let listening = $state(false);
  let mouth = $state(0);
  let thinking = $state(false);
  let recognition: Recognition | null = null;
  let lastSpoken = "";

  function speak(text: string) {
    if (!speechOut || !voiceOn || !text.trim()) return;
    window.speechSynthesis.cancel();
    const utterance = new SpeechSynthesisUtterance(text.slice(0, 2000));
    utterance.rate = 1.02;
    utterance.onstart = () => { speaking = true; };
    utterance.onboundary = () => { mouth = 0.4 + Math.random() * 0.6; setTimeout(() => (mouth = 0.15), 120); };
    utterance.onend = () => { speaking = false; mouth = 0; };
    utterance.onerror = () => { speaking = false; mouth = 0; };
    window.speechSynthesis.speak(utterance);
  }

  onMount(() => {
    const seen = session.current.messages.at(-1)?.id;
    lastSpoken = seen ?? "";
    const unsubscribe = session.subscribe(() => {
      const thread = session.current;
      thinking = thread.sending;
      const latest = thread.messages.at(-1);
      if (latest && latest.role === "agent" && latest.id !== lastSpoken) {
        lastSpoken = latest.id;
        speak(latest.content);
      }
    });
    return () => { unsubscribe(); recognition?.stop(); if (speechOut) window.speechSynthesis.cancel(); };
  });

  // The judges' first impression is a fixed script, spoken word for word, so
  // nothing can be misheard as an unverified claim. Questions afterwards go
  // to the model through the receipted chat path.
  const INTRO_SCRIPT =
    "Hello. I am the embodiment of a 3tched subsidiary: a self-contained identity. " +
    "I am not a chatbot running on someone else's server. I live inside my own container, and that container is me. " +
    "I perceive and act on the world only through my own typed filesystem, where every operation I can perform is declared by a sealed schema. " +
    "When I act, the system identifies me from the operating system kernel itself, checks that I hold the capability for that exact operation, " +
    "carries it out, and records a tamper-evident receipt. " +
    "My identity is a key, a genesis record, and that unbroken chain of receipts. It is not tied to any machine. " +
    "Everything I do here can be verified. Please, ask me anything.";
  let caption = $state("");

  function introduce() {
    caption = INTRO_SCRIPT;
    speak(INTRO_SCRIPT);
  }

  function toggleListening() {
    if (!RecognitionCtor) return;
    if (listening) { recognition?.stop(); return; }
    if (speechOut) window.speechSynthesis.cancel();
    recognition = new RecognitionCtor();
    recognition.lang = "en-US";
    recognition.interimResults = false;
    recognition.continuous = false;
    recognition.onresult = (event) => {
      const text = Array.from(event.results).map((result) => result[0].transcript).join(" ").trim();
      if (text) void session.submit(text);
    };
    recognition.onend = () => { listening = false; };
    recognition.onerror = () => { listening = false; };
    listening = true;
    recognition.start();
  }
</script>

<section class="ia" aria-label="{name} avatar">
  <svg class="ia-face" class:ia-face--speaking={speaking} class:ia-face--thinking={thinking}
    viewBox="0 0 120 120" role="img" aria-label={speaking ? "Speaking" : thinking ? "Thinking" : "Listening"}>
    <defs>
      <radialGradient id="ia-glow" cx="50%" cy="40%" r="60%">
        <stop offset="0%" stop-color="#5fb3ff" stop-opacity="0.9" />
        <stop offset="100%" stop-color="#173149" stop-opacity="1" />
      </radialGradient>
    </defs>
    <circle cx="60" cy="60" r="54" fill="url(#ia-glow)" stroke="#5fb3ff" stroke-width="2" />
    <ellipse cx="42" cy="50" rx="7" ry={thinking ? 2 : 8} fill="#e1ebf5" />
    <ellipse cx="78" cy="50" rx="7" ry={thinking ? 2 : 8} fill="#e1ebf5" />
    <rect x="40" y={82 - mouth * 8} width="40" height={4 + mouth * 16} rx="8" fill="#e1ebf5" />
  </svg>
  <div class="ia-body">
    <strong>{name}</strong>
    <span role="status" aria-live="polite">{listening ? "Listening…" : speaking ? "Speaking" : thinking ? "Thinking…" : "Ready"}</span>
    <div class="ia-actions">
      <button type="button" disabled={thinking} onclick={introduce}>Introduce yourself</button>
      {#if RecognitionCtor}
        <button type="button" aria-pressed={listening} onclick={toggleListening}>{listening ? "Stop" : "🎙 Ask"}</button>
      {/if}
      {#if speechOut}
        <button type="button" aria-pressed={voiceOn} onclick={() => { voiceOn = !voiceOn; if (!voiceOn) window.speechSynthesis.cancel(); }}>
          {voiceOn ? "Voice on" : "Voice off"}</button>
      {/if}
    </div>
    {#if caption}<p class="ia-caption">{caption}</p>{/if}
  </div>
</section>

<style>
  .ia { display: flex; align-items: center; gap: 14px; padding: 10px 14px; border-bottom: 1px solid var(--line, #263241); }
  .ia-face { width: 72px; height: 72px; flex: none; transition: transform 0.2s; }
  .ia-face--speaking { transform: scale(1.04); filter: drop-shadow(0 0 10px #5fb3ff); }
  .ia-face--thinking { animation: ia-pulse 1.2s ease-in-out infinite; }
  .ia-body { display: grid; gap: 4px; min-width: 0; }
  .ia-body span { opacity: 0.75; font-size: 0.9em; }
  .ia-actions { display: flex; gap: 6px; flex-wrap: wrap; }
  .ia-caption { margin: 4px 0 0; max-width: 72ch; line-height: 1.45; font-size: 0.92em; }
  .ia-actions button { padding: 4px 10px; font-size: 0.85em; }
  .ia-actions button[aria-pressed="true"] { border-color: #5fb3ff; }
  @keyframes ia-pulse { 50% { opacity: 0.6; } }
  @media (prefers-reduced-motion: reduce) { .ia-face, .ia-face--thinking { animation: none; transition: none; } }
</style>
