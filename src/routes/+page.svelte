<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";

  type Rgb = { r: number; g: number; b: number };
  type PowerInfo = { percent: number; charging: boolean; raw: number[] };
  type Tab = "lighting" | "audio" | "eq" | "power" | "about";

  let tab = $state<Tab>("lighting");
  let connected = $state(false);
  let busy = $state(false);
  let lastError = $state("");

  // device state
  let firmware = $state("—");
  let power = $state<PowerInfo | null>(null);
  let headsetPresent = $state<boolean | null>(null);
  let sidetoneVol = $state<number | null>(null);
  let sidetoneOn = $state<boolean | null>(null);

  // lighting
  let colorHex = $state("#ff2d55");
  let effectMode = $state<"Off" | "Static" | "Breathing" | "Wave" | "Rainbow">("Static");
  let intensity = $state(50);

  // audio / link
  let noiseReduction = $state(false);
  let demoMode = $state(false);
  let latency = $state(100);

  // EQ (host-side, via Equalizer APO / EasyEffects)
  const EQ_FREQS = [31, 62, 125, 250, 500, 1000, 2000, 4000, 8000, 16000];
  let eqGains = $state<number[]>(Array(10).fill(0));
  let preamp = $state(0);
  let eqStatus = $state("");
  let eqBackend = $state<string | null>(null);
  let installing = $state(false);

  const LATENCY_PRESETS = [40, 60, 80, 100];
  const EFFECTS: { id: typeof effectMode; label: string }[] = [
    { id: "Off", label: "Off" },
    { id: "Static", label: "Static" },
    { id: "Breathing", label: "Breathing" },
    { id: "Wave", label: "Wave" },
    { id: "Rainbow", label: "Rainbow" },
  ];

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
    busy = true;
    // Force a fresh device handle in case the headset was unplugged or the
    // wireless link dropped while the app was running.
    await call<boolean>("reconnect");
    eqBackend = (await call<string | null>("audio_backend_status")) ?? null;
    const fw = await call<string>("firmware_version");
    connected = fw !== undefined;
    if (fw !== undefined) firmware = fw;
    power = (await call<PowerInfo>("power_info")) ?? null;
    headsetPresent = (await call<boolean>("headset_present")) ?? null;
    noiseReduction = (await call<boolean>("noise_reduction")) ?? false;
    latency = (await call<number>("latency_mode")) ?? 100;
    const st = await call<[number, boolean]>("sidetone");
    if (st) { sidetoneVol = st[0]; sidetoneOn = st[1]; }
    busy = false;
  }

  async function applyLed() {
    // Static has no device-side brightness, so bake intensity into the color.
    // Animated effects take intensity as their own parameter.
    const base = hexToRgb(colorHex);
    const f = intensity / 100;
    const color =
      effectMode === "Static"
        ? { r: Math.round(base.r * f), g: Math.round(base.g * f), b: Math.round(base.b * f) }
        : base;
    await call("set_rgb", { mode: effectMode, color, intensity });
  }

  async function applyEq() {
    eqStatus = "applying…";
    const config = {
      bands: EQ_FREQS.map((f, i) => ({ freq_hz: f, gain_db: eqGains[i], q: 1.0 })),
      preamp_db: preamp,
      surround_enabled: false,
    };
    const ok = await call("set_eq", { config });
    eqStatus = ok === undefined ? "" : "applied ✓";
  }
  function resetEq() {
    eqGains = Array(10).fill(0);
    preamp = 0;
    eqStatus = "";
  }
  async function installEq() {
    installing = true;
    const msg = await call<string>("install_eq_backend");
    eqStatus = msg ?? "";
    installing = false;
    // Re-check after a delay; user still needs to walk the installer.
    setTimeout(async () => {
      eqBackend = (await call<string | null>("audio_backend_status")) ?? null;
    }, 4000);
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

  let batteryPct = $derived(power ? Math.min(100, Math.max(0, power.percent)) : 0);

  async function checkBackend() {
    eqBackend = (await call<string | null>("audio_backend_status")) ?? null;
  }

  $effect(() => { refresh(); });
  // Re-detect the audio backend every time the user enters the EQ tab so a
  // freshly-installed APO is picked up without needing a manual refresh.
  $effect(() => {
    if (tab === "eq") checkBackend();
  });
</script>

<div class="app">
  <!-- ambient -->
  <div class="grain"></div>

  <header class="bar">
    <div class="brand">
      <span class="logo">◆</span>
      <span class="word">OPEN<b>PELTA</b></span>
    </div>
    <div class="conn" class:on={connected}>
      <span class="dot"></span>{connected ? "LINKED" : "OFFLINE"}
    </div>
  </header>

  {#if lastError}
    <div class="err">⚠ {lastError}</div>
  {/if}

  <nav class="tabs">
    {#each ["lighting", "audio", "eq", "power", "about"] as t}
      <button class="tab" class:active={tab === t} onclick={() => (tab = t as Tab)}>{t}</button>
    {/each}
  </nav>

  <main>
    {#if tab === "lighting"}
      <section class="panel">
        <div class="swatch" style:--c={colorHex}>
          <input type="color" bind:value={colorHex} aria-label="LED color" />
          <div class="swatch-meta">
            <span class="mono lg">{colorHex.toUpperCase()}</span>
            <span class="cap">LED color</span>
          </div>
        </div>

        <div class="field">
          <span class="cap">Effect</span>
          <div class="seg">
            {#each EFFECTS as e}
              <button class:active={effectMode === e.id} onclick={() => (effectMode = e.id)}>{e.label}</button>
            {/each}
          </div>
        </div>

        <div class="field">
          <span class="cap">Intensity <em class="mono">{intensity}</em></span>
          <input class="slider" type="range" min="0" max="100" bind:value={intensity} />
        </div>

        <button class="apply" onclick={applyLed}>Apply lighting</button>
      </section>
    {/if}

    {#if tab === "audio"}
      <section class="panel">
        <button class="toggle" class:on={noiseReduction} onclick={toggleNr}>
          <span class="knob"></span>
          <span class="t-label">Mic noise reduction</span>
          <span class="mono state">{noiseReduction ? "ON" : "OFF"}</span>
        </button>
        <button class="toggle" class:on={demoMode} onclick={toggleDemo}>
          <span class="knob"></span>
          <span class="t-label">Demo mode</span>
          <span class="mono state">{demoMode ? "ON" : "OFF"}</span>
        </button>

        <div class="field">
          <span class="cap">Wireless latency</span>
          <div class="seg">
            {#each LATENCY_PRESETS as ms}
              <button class:active={latency === ms} onclick={() => setLatency(ms)}>{ms}<small>ms</small></button>
            {/each}
          </div>
        </div>

        <p class="note">Sidetone level is a USB-audio control, not a vendor
          command — read-only for now (see <span class="mono">docs/protocol.md</span>).</p>
      </section>
    {/if}

    {#if tab === "eq"}
      <section class="panel">
        {#if eqBackend}
          <div class="backend-ok">
            <span class="cap">Backend</span>
            <span class="mono">{eqBackend}</span>
          </div>
        {:else}
          <div class="backend-missing">
            <p>No system EQ backend detected.</p>
            <div class="row-btns">
              <button class="ghost" onclick={checkBackend}>Check again</button>
              <button class="apply" onclick={installEq} disabled={installing}>
                {installing ? "launching installer…" : "Install Equalizer APO"}
              </button>
            </div>
            <small class="hint">Downloads Equalizer APO from SourceForge and launches its installer.
              In the wizard, pick the <span class="mono">ROG Pelta</span> output device and reboot.
              The app will detect it automatically next time you open this tab.</small>
          </div>
        {/if}

        <div class="field">
          <span class="cap">Preamp <em class="mono">{preamp > 0 ? "+" : ""}{preamp} dB</em></span>
          <input class="slider" type="range" min="-12" max="12" step="0.5" bind:value={preamp} />
        </div>
        <div class="eq">
          {#each EQ_FREQS as f, i}
            <div class="eq-band">
              <input
                class="vrange"
                type="range" min="-12" max="12" step="0.5"
                bind:value={eqGains[i]}
              />
              <span class="eq-gain mono">{eqGains[i] > 0 ? "+" : ""}{eqGains[i]}</span>
              <span class="eq-freq mono">{f >= 1000 ? f / 1000 + "k" : f}</span>
            </div>
          {/each}
        </div>
        <div class="row-btns">
          <button class="ghost" onclick={resetEq}>Reset</button>
          <button class="apply" onclick={applyEq}>Apply EQ</button>
        </div>
        {#if eqStatus}<span class="hint">{eqStatus}</span>{/if}
        <p class="note">EQ is applied by the system audio backend (Equalizer APO
          on Windows). If none is installed, applying returns an error.</p>
      </section>
    {/if}

    {#if tab === "power"}
      <section class="panel">
        <div class="battery">
          <div class="bat-top">
            <span class="cap">Battery</span>
            <span class="mono lg">{power ? power.percent + "%" : "—"}{power?.charging ? " ⚡" : ""}</span>
          </div>
          <div class="bat-track"><div class="bat-fill" style:width="{batteryPct}%"></div></div>
          <span class="hint">raw: {power ? power.raw.map((b) => b.toString(16).padStart(2, "0")).join(" ") : "—"} · % unconfirmed</span>
        </div>

        <dl class="readout">
          <dt>Headset</dt>
          <dd class:ok={headsetPresent}>{headsetPresent === null ? "—" : headsetPresent ? "PRESENT" : "ABSENT"}</dd>
          <dt>Charging</dt>
          <dd>{power ? (power.charging ? "YES" : "NO") : "—"}</dd>
          <dt>Sidetone</dt>
          <dd>{sidetoneVol === null ? "—" : `${sidetoneVol}${sidetoneOn ? "" : " · off"}`}</dd>
          <dt>Latency</dt>
          <dd>{latency} ms</dd>
        </dl>
      </section>
    {/if}

    {#if tab === "about"}
      <section class="panel about">
        <h2 class="disp">ROG PELTA</h2>
        <dl class="readout">
          <dt>Firmware</dt><dd>{firmware}</dd>
          <dt>Driver</dt><dd>OpenPelta 0.1</dd>
        </dl>
        <p class="note">Open-source replacement for ASUS Armoury Crate Gear.
          Protocol reverse-engineered from scratch — see the project repo.</p>
        <p class="note warn">⚠ Firmware updates are intentionally not supported
          here. Flashing over USB carries a brick risk and the OTA protocol is
          unverified — use ASUS's official updater for firmware.</p>
      </section>
    {/if}
  </main>

  <footer>
    <button class="refresh" onclick={refresh} disabled={busy}>
      {busy ? "syncing…" : "↻ refresh"}
    </button>
  </footer>
</div>

<style>
  :global(html), :global(body) { margin: 0; height: 100%; background: #08080a; }
  * { box-sizing: border-box; }

  .app {
    --bg: #0a0a0c;
    --panel: #141418;
    --panel-2: #1a1a20;
    --line: #26262e;
    --text: #e8e8ee;
    --dim: #74747f;
    --accent: #ff2d55;
    --ok: #34e0a1;
    --font-disp: "Bahnschrift", "DIN Alternate", "Oswald", system-ui, sans-serif;
    --font-mono: "Cascadia Mono", "JetBrains Mono", "Consolas", ui-monospace, monospace;

    position: relative;
    min-height: 100vh;
    color: var(--text);
    background:
      radial-gradient(120% 80% at 50% -10%, #16161c 0%, var(--bg) 60%);
    font-family: var(--font-disp);
    letter-spacing: 0.02em;
    overflow: hidden;
    display: flex;
    flex-direction: column;
  }

  /* cheap static grain (single repeating gradient, no image) */
  .grain {
    position: absolute; inset: 0; pointer-events: none; opacity: 0.025;
    background-image: repeating-linear-gradient(0deg, #fff 0 1px, transparent 1px 3px);
    mix-blend-mode: overlay;
  }

  .bar {
    display: flex; align-items: center; justify-content: space-between;
    padding: 0.85rem 1.1rem; border-bottom: 1px solid var(--line);
  }
  .brand { display: flex; align-items: center; gap: 0.5rem; }
  .logo { color: var(--accent); font-size: 0.7rem; transform: rotate(0deg); }
  .word { font-size: 1.05rem; letter-spacing: 0.18em; font-weight: 400; }
  .word b { font-weight: 700; }
  .conn {
    display: flex; align-items: center; gap: 0.45rem;
    font-family: var(--font-mono); font-size: 0.66rem; letter-spacing: 0.15em;
    color: var(--dim);
  }
  .conn .dot { width: 7px; height: 7px; border-radius: 50%; background: var(--dim); }
  .conn.on { color: var(--ok); }
  .conn.on .dot { background: var(--ok); box-shadow: 0 0 8px var(--ok); animation: pulse 2.4s ease-in-out infinite; }
  @keyframes pulse { 0%,100% { opacity: 1; } 50% { opacity: 0.35; } }

  .err {
    margin: 0.6rem 1.1rem -0.2rem; padding: 0.5rem 0.7rem;
    background: #2a1418; border: 1px solid #5a2230; border-radius: 6px;
    color: #ff9aa9; font-family: var(--font-mono); font-size: 0.72rem;
  }

  .tabs { display: flex; gap: 0.2rem; padding: 0.7rem 1.1rem 0; }
  .tab {
    flex: 1; background: transparent; border: 0; color: var(--dim);
    font-family: var(--font-disp); font-size: 0.78rem; letter-spacing: 0.16em;
    text-transform: uppercase; padding: 0.55rem 0; cursor: pointer;
    border-bottom: 2px solid transparent; transition: color 0.15s, border-color 0.15s;
  }
  .tab:hover { color: var(--text); }
  .tab.active { color: var(--text); border-bottom-color: var(--accent); }

  main { flex: 1; padding: 1.1rem; }
  .panel {
    background: linear-gradient(180deg, var(--panel) 0%, var(--panel-2) 100%);
    border: 1px solid var(--line); border-radius: 12px; padding: 1.2rem;
    display: flex; flex-direction: column; gap: 1.15rem;
  }

  .cap { font-family: var(--font-mono); font-size: 0.62rem; letter-spacing: 0.18em;
    text-transform: uppercase; color: var(--dim); }
  .mono { font-family: var(--font-mono); }
  .mono.lg { font-size: 1rem; letter-spacing: 0.05em; }

  /* color swatch */
  .swatch { display: flex; align-items: center; gap: 1rem; }
  .swatch input[type="color"] {
    -webkit-appearance: none; appearance: none; width: 64px; height: 64px;
    border: 1px solid var(--line); border-radius: 12px; background: none; cursor: pointer;
    padding: 0; box-shadow: 0 0 0 4px color-mix(in srgb, var(--c) 22%, transparent);
  }
  .swatch input::-webkit-color-swatch-wrapper { padding: 4px; }
  .swatch input::-webkit-color-swatch { border: none; border-radius: 8px; }
  .swatch-meta { display: flex; flex-direction: column; gap: 0.3rem; }

  .field { display: flex; flex-direction: column; gap: 0.55rem; }

  /* segmented control */
  .seg { display: flex; gap: 0.3rem; }
  .seg button {
    flex: 1; background: #101015; border: 1px solid var(--line); color: var(--dim);
    border-radius: 7px; padding: 0.5rem 0.3rem; cursor: pointer;
    font-family: var(--font-disp); font-size: 0.78rem; letter-spacing: 0.06em;
    transition: all 0.14s;
  }
  .seg button small { font-size: 0.6em; opacity: 0.6; margin-left: 1px; }
  .seg button:hover { color: var(--text); border-color: #3a3a44; }
  .seg button.active {
    color: #fff; border-color: var(--accent);
    background: color-mix(in srgb, var(--accent) 16%, #101015);
    box-shadow: inset 0 0 12px color-mix(in srgb, var(--accent) 20%, transparent);
  }

  /* slider */
  .slider { -webkit-appearance: none; appearance: none; width: 100%; height: 4px;
    background: var(--line); border-radius: 4px; outline: none; }
  .slider::-webkit-slider-thumb {
    -webkit-appearance: none; width: 16px; height: 16px; border-radius: 50%;
    background: var(--accent); cursor: pointer; border: 2px solid #0a0a0c;
    box-shadow: 0 0 10px color-mix(in srgb, var(--accent) 60%, transparent);
  }
  .field em { color: var(--text); font-style: normal; }

  .apply {
    margin-top: 0.2rem; background: var(--accent); color: #fff; border: 0;
    border-radius: 8px; padding: 0.7rem; cursor: pointer;
    font-family: var(--font-disp); font-size: 0.85rem; letter-spacing: 0.12em;
    text-transform: uppercase; transition: filter 0.14s, transform 0.05s;
  }
  .apply:hover { filter: brightness(1.12); }
  .apply:active { transform: translateY(1px); }

  /* toggles */
  .toggle {
    display: flex; align-items: center; gap: 0.8rem; width: 100%;
    background: #101015; border: 1px solid var(--line); border-radius: 9px;
    padding: 0.7rem 0.9rem; cursor: pointer; color: var(--text);
    font-family: var(--font-disp); font-size: 0.88rem; transition: border-color 0.15s;
  }
  .toggle:hover { border-color: #3a3a44; }
  .toggle .knob {
    position: relative; width: 38px; height: 20px; border-radius: 999px;
    background: #2a2a32; flex: none; transition: background 0.18s;
  }
  .toggle .knob::after {
    content: ""; position: absolute; top: 2px; left: 2px; width: 16px; height: 16px;
    border-radius: 50%; background: #6a6a76; transition: transform 0.18s, background 0.18s;
  }
  .toggle.on .knob { background: color-mix(in srgb, var(--ok) 35%, #101015); }
  .toggle.on .knob::after { transform: translateX(18px); background: var(--ok); }
  .toggle .t-label { flex: 1; text-align: left; }
  .toggle .state { font-size: 0.66rem; letter-spacing: 0.12em; color: var(--dim); }
  .toggle.on .state { color: var(--ok); }

  .note { font-family: var(--font-mono); font-size: 0.68rem; line-height: 1.5;
    color: var(--dim); margin: 0.2rem 0 0; }
  .note.warn { color: #ffb98a; }

  /* EQ */
  .eq { display: flex; justify-content: space-between; gap: 0.3rem; height: 180px; padding: 0.3rem 0; }
  .eq-band { display: flex; flex-direction: column; align-items: center; gap: 0.35rem; flex: 1; }
  .vrange {
    -webkit-appearance: slider-vertical; appearance: slider-vertical;
    writing-mode: vertical-lr; direction: rtl;
    width: 6px; flex: 1; background: var(--line); border-radius: 4px; cursor: pointer; accent-color: var(--accent);
  }
  .eq-gain { font-size: 0.6rem; color: var(--text); }
  .eq-freq { font-size: 0.58rem; color: var(--dim); letter-spacing: 0.04em; }
  .row-btns { display: flex; gap: 0.5rem; }
  .row-btns .apply { flex: 1; margin: 0; }
  .ghost {
    background: transparent; border: 1px solid var(--line); color: var(--dim);
    border-radius: 8px; padding: 0.7rem 1.1rem; cursor: pointer;
    font-family: var(--font-disp); font-size: 0.85rem; letter-spacing: 0.1em;
    text-transform: uppercase; transition: all 0.14s;
  }
  .ghost:hover { color: var(--text); border-color: #3a3a44; }

  .backend-ok {
    display: flex; justify-content: space-between; align-items: center;
    padding: 0.45rem 0.75rem; background: color-mix(in srgb, var(--ok) 12%, #101015);
    border: 1px solid color-mix(in srgb, var(--ok) 30%, var(--line));
    border-radius: 7px; margin-bottom: 0.2rem;
  }
  .backend-ok .mono { color: var(--ok); font-size: 0.78rem; }

  .backend-missing {
    display: flex; flex-direction: column; gap: 0.55rem;
    padding: 0.85rem; border: 1px dashed #3a3a44; border-radius: 8px;
    background: #101015;
  }
  .backend-missing p { margin: 0; font-size: 0.85rem; color: var(--text); }
  .backend-missing .apply { margin: 0; }

  /* battery */
  .battery { display: flex; flex-direction: column; gap: 0.5rem; }
  .bat-top { display: flex; align-items: baseline; justify-content: space-between; }
  .bat-track { height: 10px; background: #101015; border: 1px solid var(--line);
    border-radius: 6px; overflow: hidden; }
  .bat-fill { height: 100%; background: linear-gradient(90deg, var(--accent), #ff6b8a);
    transition: width 0.4s ease; }
  .hint { font-family: var(--font-mono); font-size: 0.6rem; color: var(--dim); letter-spacing: 0.1em; }

  /* readout list */
  .readout { display: grid; grid-template-columns: auto 1fr; gap: 0.5rem 1rem; margin: 0; }
  .readout dt { font-family: var(--font-mono); font-size: 0.66rem; letter-spacing: 0.14em;
    text-transform: uppercase; color: var(--dim); align-self: center; }
  .readout dd { margin: 0; text-align: right; font-family: var(--font-mono);
    font-size: 0.82rem; color: var(--text); }
  .readout dd.ok { color: var(--ok); }

  .about .disp { font-size: 1.6rem; letter-spacing: 0.14em; margin: 0 0 0.4rem; font-weight: 700; }

  footer { padding: 0.8rem 1.1rem 1.1rem; }
  .refresh {
    width: 100%; background: transparent; border: 1px solid var(--line);
    color: var(--dim); border-radius: 8px; padding: 0.6rem; cursor: pointer;
    font-family: var(--font-mono); font-size: 0.72rem; letter-spacing: 0.12em;
    transition: all 0.15s;
  }
  .refresh:hover:not(:disabled) { color: var(--text); border-color: #3a3a44; }
  .refresh:disabled { opacity: 0.5; cursor: default; }
</style>
