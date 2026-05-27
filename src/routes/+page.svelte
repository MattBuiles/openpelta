<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";

  type Rgb = { r: number; g: number; b: number };
  type PowerInfo = { raw_level: number; charging: boolean };

  let connected = $state(false);
  let firmware = $state("—");
  let power = $state<PowerInfo | null>(null);
  let headsetPresent = $state<boolean | null>(null);
  let lastError = $state("");

  // LED
  let colorHex = $state("#ff0000");
  let effectMode = $state<"Static" | "Breathing" | "Wave" | "Rainbow" | "Off">("Static");
  let intensity = $state(50);

  // Audio / link
  let noiseReduction = $state(false);
  let latency = $state(100);
  let demoMode = $state(false);
  let sidetoneVol = $state<number | null>(null);
  let sidetoneOn = $state<boolean | null>(null);

  const LATENCY_PRESETS = [40, 60, 80, 100];

  function hexToRgb(hex: string): Rgb {
    const n = parseInt(hex.slice(1), 16);
    return { r: (n >> 16) & 0xff, g: (n >> 8) & 0xff, b: n & 0xff };
  }

  async function call<T>(cmd: string, args?: Record<string, unknown>): Promise<T | undefined> {
    try {
      lastError = "";
      return await invoke<T>(cmd, args);
    } catch (e) {
      lastError = String(e);
      return undefined;
    }
  }

  async function refresh() {
    const fw = await call<string>("firmware_version");
    connected = fw !== undefined;
    if (fw !== undefined) firmware = fw;
    power = (await call<PowerInfo>("power_info")) ?? null;
    headsetPresent = (await call<boolean>("headset_present")) ?? null;
    const led = await call<boolean>("led_enabled");
    if (led !== undefined && !led) effectMode = "Off";
    noiseReduction = (await call<boolean>("noise_reduction")) ?? false;
    latency = (await call<number>("latency_mode")) ?? 100;
    const st = await call<[number, boolean]>("sidetone");
    if (st) { sidetoneVol = st[0]; sidetoneOn = st[1]; }
  }

  async function applyLed() {
    const color = hexToRgb(colorHex);
    await call("set_rgb", { mode: effectMode, color, intensity });
  }

  async function toggleNr() {
    noiseReduction = !noiseReduction;
    await call("set_noise_reduction", { on: noiseReduction });
  }

  async function toggleDemo() {
    demoMode = !demoMode;
    await call("set_demo_mode", { on: demoMode });
  }

  async function setLatency(ms: number) {
    latency = ms;
    await call("set_latency_mode", { valueMs: ms });
  }

  $effect(() => { refresh(); });
</script>

<main class="wrap">
  <header>
    <h1>OpenPelta</h1>
    <span class="status" class:ok={connected}>
      {connected ? "Connected" : "Not connected"}
    </span>
  </header>

  {#if lastError}
    <div class="error">{lastError}</div>
  {/if}

  <section class="card">
    <h2>Device</h2>
    <dl>
      <dt>Firmware</dt><dd>{firmware}</dd>
      <dt>Battery (raw)</dt><dd>{power ? power.raw_level : "—"}{power?.charging ? " ⚡" : ""}</dd>
      <dt>Headset present</dt><dd>{headsetPresent === null ? "—" : headsetPresent ? "yes" : "no"}</dd>
      <dt>Sidetone</dt><dd>{sidetoneVol === null ? "—" : `${sidetoneVol}${sidetoneOn ? "" : " (off)"}`}</dd>
    </dl>
    <button onclick={refresh}>Refresh</button>
  </section>

  <section class="card">
    <h2>Lighting</h2>
    <div class="row">
      <label>Color <input type="color" bind:value={colorHex} /></label>
      <label>Mode
        <select bind:value={effectMode}>
          <option value="Off">Off</option>
          <option value="Static">Static</option>
          <option value="Breathing">Breathing</option>
          <option value="Wave">Wave</option>
          <option value="Rainbow">Rainbow</option>
        </select>
      </label>
    </div>
    <label>Intensity {intensity}
      <input type="range" min="0" max="100" bind:value={intensity} />
    </label>
    <button onclick={applyLed}>Apply lighting</button>
  </section>

  <section class="card">
    <h2>Audio &amp; Link</h2>
    <label class="toggle">
      <input type="checkbox" checked={noiseReduction} onchange={toggleNr} />
      Mic noise reduction
    </label>
    <label class="toggle">
      <input type="checkbox" checked={demoMode} onchange={toggleDemo} />
      Demo mode
    </label>
    <div class="latency">
      <span>Wireless latency</span>
      <div class="row">
        {#each LATENCY_PRESETS as ms}
          <button class:active={latency === ms} onclick={() => setLatency(ms)}>{ms} ms</button>
        {/each}
      </div>
    </div>
  </section>

  <footer>
    <small>EQ, sidetone level and mic mute are not yet wired (see docs/protocol.md).</small>
  </footer>
</main>

<style>
  :root {
    font-family: Inter, system-ui, sans-serif;
    color: #e8e8ea;
    background: #16161a;
  }
  .wrap { max-width: 560px; margin: 0 auto; padding: 1.5rem; display: flex; flex-direction: column; gap: 1rem; }
  header { display: flex; align-items: center; justify-content: space-between; }
  h1 { font-size: 1.4rem; margin: 0; }
  h2 { font-size: 0.95rem; margin: 0 0 0.75rem; text-transform: uppercase; letter-spacing: 0.05em; opacity: 0.7; }
  .status { font-size: 0.8rem; padding: 0.2rem 0.6rem; border-radius: 999px; background: #3a2b2b; color: #f0a0a0; }
  .status.ok { background: #1f3a2b; color: #8ce0a8; }
  .error { background: #3a2b2b; color: #f0a0a0; padding: 0.6rem 0.8rem; border-radius: 8px; font-size: 0.85rem; }
  .card { background: #1e1e24; border: 1px solid #2a2a32; border-radius: 12px; padding: 1rem 1.2rem; }
  dl { display: grid; grid-template-columns: auto 1fr; gap: 0.3rem 1rem; margin: 0 0 0.8rem; font-size: 0.9rem; }
  dt { opacity: 0.6; }
  dd { margin: 0; text-align: right; font-variant-numeric: tabular-nums; }
  .row { display: flex; gap: 1rem; align-items: center; flex-wrap: wrap; }
  label { display: flex; flex-direction: column; gap: 0.3rem; font-size: 0.85rem; }
  .row label { flex-direction: row; align-items: center; gap: 0.5rem; }
  .toggle { flex-direction: row; align-items: center; gap: 0.5rem; margin-bottom: 0.5rem; }
  input[type="range"] { width: 100%; }
  .latency { margin-top: 0.5rem; }
  button {
    background: #2a2a32; color: #e8e8ea; border: 1px solid #3a3a44;
    border-radius: 8px; padding: 0.5rem 0.9rem; cursor: pointer; font-size: 0.85rem;
  }
  button:hover { border-color: #5a5a66; }
  button.active { background: #2f4a6a; border-color: #4a7ab0; }
  footer { text-align: center; opacity: 0.5; }
</style>
