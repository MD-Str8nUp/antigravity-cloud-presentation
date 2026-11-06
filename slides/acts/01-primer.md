---
class: text-center
---

<!-- S01 -->
<div class="h-full flex flex-col items-center justify-center">

  <h1 class="text-5xl font-extrabold tracking-tight text-white mb-3">
    Vibe Coding vs. Skill-Driven Engineering
  </h1>
  <p class="text-2xl text-gray-300 font-light max-w-3xl">
    How Rules, Composable Agent Skills, and Google Cloud Skills Shape Antigravity Across Context Boundaries
  </p>

  <div class="mt-10 text-gray-400 text-base">
    <span class="font-semibold text-gray-200">Brett Morgan</span> &middot; Developer Relations Engineer, Google
  </div>

  <div class="mt-10 text-xs text-gray-500">
    Press <kbd>Space</kbd> or <kbd>&rarr;</kbd> to advance &middot; <kbd>[</kbd> <kbd>p</kbd> <kbd>]</kbd> on terminal recordings
  </div>
</div>

<!--
Welcome everyone! Today we are exploring two very different ways of working with an AI coding agent like Google Antigravity: rapid, exploratory vibe coding on one hand, and structured, skill-driven engineering on the other. Rather than arguing that one approach is always right and the other is always wrong, we recorded both approaches with full terminal recordings and conversation traces—followed by a lead-in to productionizing on Google Cloud with the google/skills skill set—so you can see exactly when each workflow shines and where it breaks down.
-->

---
layout: default
---

<!-- S01a -->
<div class="flex items-center gap-3 mb-4">
  <h2 class="text-2xl font-bold m-0">Follow Along: Antigravity &amp; Interactive Deck</h2>
</div>

<div class="grid grid-cols-2 gap-6 mt-4">
  <QrCodeCard
    badge="Agentic IDE & CLI"
    title="Google Antigravity"
    src="/images/qrcode_antigravity.google.png"
    url="https://antigravity.google/"
    subtitle="Download Google Antigravity and explore rules, composable skills, and MCP integrations."
    accent="cyan"
  />
  <QrCodeCard
    badge="Live Slides, Casts & 5-Lesson Codelab"
    title="Presentation & Codelab"
    src="/images/qrcode_domesticmouse.github.io.png"
    url="https://domesticmouse.github.io/antigravity-cloud-presentation/"
    subtitle="Follow the interactive Slidev deck, replay terminal recordings, and try the hands-on Codelab."
    accent="emerald"
  />
</div>

<!--
Before we dive in, if you want to follow along on your own laptop or phone, here are two QR codes. On the left is antigravity.google where you can grab Google Antigravity, and on the right is the live link to this presentation deck, which includes all of our interactive terminal recordings and the five-lesson hands-on codelab.
-->

---
layout: default
---

<!-- S02 -->
<div class="flex items-center gap-3 mb-4">
  <h2 class="text-2xl font-bold m-0">The Core Tension: Speed vs. Comprehension Debt</h2>
</div>

<div class="grid grid-cols-2 gap-6 mt-6">
  <div class="p-5 rounded-xl bg-red-950/15 border border-red-500/30 space-y-3">
    <div class="text-xs font-mono text-red-400 uppercase">The /clear Boundary Problem</div>
    <h3 class="text-lg font-bold text-white">When Intent Lives Only in Chat</h3>
    <ul class="text-xs text-gray-300 space-y-2 list-disc pl-4">
      <li>Modern agents can generate 2,000+ lines of working application code in 15 minutes from a three-sentence prompt.</li>
      <li>Every unasked domain assumption—schema trade-offs, edge cases, state transitions—is decided silently inside the model's context window.</li>
      <li><strong>Result:</strong> The moment you run <code>/clear</code> or return the next day, neither you nor the next agent session has a durable record of <em>what</em> was meant vs. <em>what</em> was accidentally coded.</li>
    </ul>
  </div>

  <div class="p-5 rounded-xl bg-emerald-950/15 border border-emerald-500/30 space-y-3">
    <div class="text-xs font-mono text-emerald-400 uppercase">Engineering Judgment</div>
    <h3 class="text-lg font-bold text-white">Choosing Ceremony Intentionally</h3>
    <ul class="text-xs text-gray-300 space-y-2 list-disc pl-4">
      <li><strong>Vibe Coding</strong> is genuinely great when you don't yet know what you want and need a throwaway prototype in 15 minutes to explore an idea.</li>
      <li><strong>Skill-Driven Engineering</strong> pays off as soon as the project matters and needs to survive across sessions, teammates, and production deployments.</li>
      <li><strong>Result:</strong> Treat ceremony as a dial you turn up when durability and maintenance justify it.</li>
    </ul>
  </div>
</div>

<!--
Here is the tension every developer hits with agentic coding tools. Generating two thousand lines of working code in fifteen minutes is now effortless. But when every design decision and domain edge case lives only inside a fleeting chat transcript, the moment you clear the context window or come back next week, you face comprehension debt. Our goal today is to look at how to treat engineering ceremony as a dial—using fast vibe coding when exploring, and turning up structured skills when building for longevity.
-->

---
layout: default
---

<!-- S03 -->
<div class="flex items-center gap-3 mb-5">
  <h2 class="text-2xl font-bold m-0">The Mission: Taming a Study Full of Audio Electronics Parts</h2>
</div>

<div class="grid grid-cols-12 gap-6 items-center h-88">
  <div class="col-span-5 h-full flex items-center justify-center rounded-2xl bg-gray-950/60 border border-gray-800/80 p-4 relative overflow-hidden">
    <svg viewBox="0 0 260 380" class="w-full h-full max-h-[340px]" fill="none" xmlns="http://www.w3.org/2000/svg">
      <rect x="20" y="14" width="220" height="352" rx="18" fill="#0b0c10" stroke="#374151" stroke-width="2.5" />
      <rect x="26" y="20" width="208" height="340" rx="14" fill="#121317" stroke="#1f2937" stroke-width="1.5" />
      <circle cx="42" cy="34" r="3.5" fill="#ef4444" opacity="0.8" />
      <circle cx="54" cy="34" r="3.5" fill="#f59e0b" opacity="0.8" />
      <circle cx="66" cy="34" r="3.5" fill="#10b981" opacity="0.8" />
      <rect x="82" y="29" width="136" height="10" rx="5" fill="#181b22" />
      <rect x="38" y="50" width="184" height="22" rx="7" fill="#181b22" stroke="#374151" stroke-width="1.2" />
      <circle cx="50" cy="60.5" r="4" stroke="#22d3ee" stroke-width="1.5" />
      <line x1="53" y1="63.5" x2="56.5" y2="67" stroke="#22d3ee" stroke-width="1.5" stroke-linecap="round" />
      <rect x="64" y="58" width="84" height="5" rx="2.5" fill="#374151" />
      <rect x="38" y="84" width="184" height="78" rx="10" fill="#161920" stroke="#0891b2" stroke-width="1.2" />
      <g stroke="#22d3ee" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
        <rect x="60" y="100" width="44" height="46" rx="4" fill="#083344" />
        <path d="M75 100 A7 7 0 0 0 89 100" fill="#161920" />
        <line x1="50" y1="108" x2="60" y2="108" />
        <line x1="50" y1="118" x2="60" y2="118" />
        <line x1="50" y1="128" x2="60" y2="128" />
        <line x1="50" y1="138" x2="60" y2="138" />
        <line x1="104" y1="108" x2="114" y2="108" />
        <line x1="104" y1="118" x2="114" y2="118" />
        <line x1="104" y1="128" x2="114" y2="128" />
        <line x1="104" y1="138" x2="114" y2="138" />
      </g>
      <rect x="128" y="102" width="64" height="7" rx="3.5" fill="#cbd5e1" opacity="0.85" />
      <rect x="128" y="116" width="48" height="6" rx="3" fill="#22d3ee" opacity="0.55" />
      <rect x="128" y="134" width="38" height="10" rx="4" fill="#083344" stroke="#06b6d4" stroke-width="1" />
      <rect x="38" y="172" width="184" height="78" rx="10" fill="#161920" stroke="#b45309" stroke-width="1.2" />
      <g stroke="#fbbf24" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round">
        <path d="M50 211 H60 L65 199 L73 223 L81 199 L89 223 L97 199 L102 211 H114" />
      </g>
      <rect x="128" y="190" width="64" height="7" rx="3.5" fill="#cbd5e1" opacity="0.85" />
      <rect x="128" y="204" width="52" height="6" rx="3" fill="#fbbf24" opacity="0.55" />
      <rect x="128" y="222" width="38" height="10" rx="4" fill="#451a03" stroke="#d97706" stroke-width="1" />
      <rect x="38" y="260" width="184" height="78" rx="10" fill="#161920" stroke="#047857" stroke-width="1.2" />
      <g stroke="#34d399" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round">
        <line x1="50" y1="299" x2="76" y2="299" />
        <line x1="76" y1="281" x2="76" y2="317" />
        <path d="M90 281 C83 289 83 309 90 317" />
        <line x1="86" y1="299" x2="114" y2="299" />
        <line x1="64" y1="285" x2="70" y2="285" stroke-width="1.8" />
        <line x1="67" y1="282" x2="67" y2="288" stroke-width="1.8" />
      </g>
      <rect x="128" y="278" width="64" height="7" rx="3.5" fill="#cbd5e1" opacity="0.85" />
      <rect x="128" y="292" width="44" height="6" rx="3" fill="#34d399" opacity="0.55" />
      <rect x="128" y="310" width="38" height="10" rx="4" fill="#064e3b" stroke="#10b981" stroke-width="1" />
    </svg>
  </div>
  <div class="col-span-7 flex flex-col justify-center">
    <ol class="space-y-4 text-sm text-gray-200 list-decimal pl-6 leading-relaxed m-0">
      <li>
        <strong class="text-white">A Study Full of Parts, Zero Index</strong>
        <ul class="list-disc pl-5 mt-1 space-y-1 text-gray-300">
          <li>Binders of SMD cut-tape strips, multi-drawer cabinets, and anti-static bags of op-amps (<code>TL072</code>, <code>TL074</code>), PT2399 delay chips, resistors, and capacitors scattered across the study.</li>
          <li>What looks like a simple two-table CRUD app hides real domain edge cases once the same part lives across multiple bins and packages (<code>SMT</code> vs. <code>THT</code>).</li>
        </ul>
      </li>
      <li>
        <strong class="text-white">The &ldquo;Buy It Again&rdquo; Tax &amp; DIY Kit BOM Collisions</strong>
        <ul class="list-disc pl-5 mt-1 space-y-1 text-gray-300">
          <li>Re-ordering parts from LCSC, Mouser, or Tayda that I already own because hunting through 40 unindexed drawers takes longer than buying a fresh reel.</li>
          <li>Planning multiple guitar pedal or synth <strong>Kit Builds</strong> where shared stock gets double-counted across competing Bills of Materials (BOMs).</li>
        </ul>
      </li>
      <li>
        <strong class="text-white">The Goal: A Workbench Inventory &amp; Kit Build Tracker</strong>
        <ul class="list-disc pl-5 mt-1 space-y-1 text-gray-300">
          <li>Instant search by electrical value (<code>10k</code>, <code>100nF</code>), footprint (<code>0603</code>, <code>DIP-8</code>), or part code, pinpointing the exact <strong class="text-cyan-300">Storage Location</strong>, <strong class="text-cyan-300">BOM Fulfillment</strong> state, and <strong class="text-cyan-300">Net Shortfall</strong> shopping list.</li>
        </ul>
      </li>
    </ol>
  </div>
</div>

<!--
So what are we actually going to build to test this thesis? Let me set the scene. My study at home is full of audio electronics components—SMD binders of cut-tape resistors and capacitors, multi-drawer cabinets of TL072 and TL074 op-amps, and bags of potentiometers and jacks for DIY guitar pedals and synth modules. Half the time, it's faster to re-order a twenty-cent part from Mouser or Tayda than hunt through forty drawers—and when I plan three pedal builds at once, I need to know which kits I can actually fulfill from stock on hand and what my true net shortfall is. It sounds like a simple CRUD app at first glance, but underneath are rich domain edge cases.
-->

---
layout: default
---

<!-- S04 -->
<div class="flex items-center gap-3 mb-6">
  <h2 class="text-2xl font-bold m-0">Three Ways to Shape Antigravity (<code>agy</code>)</h2>
</div>

<ol class="space-y-4 text-sm text-gray-200 list-decimal pl-6 mt-5 leading-relaxed">
  <li>
    <strong class="text-white">Rules (<code>AGENTS.md</code> &amp; Global User Rules) — Always-On Guardrails</strong>
    <ul class="list-disc pl-5 mt-1 space-y-1 text-gray-300">
      <li>Loaded into every turn automatically; ideal for short behavioral invariants like <em>"Ask clarifying questions, one at a time."</em></li>
      <li>Rules tell the agent to pause and ask, but they do not teach a multi-stage methodology or define artifact formats.</li>
    </ul>
  </li>
  <li>
    <strong class="text-white">Skills (<code>.agents/skills/*/SKILL.md</code>) — On-Demand Procedural Playbooks</strong>
    <ul class="list-disc pl-5 mt-1 space-y-1 text-gray-300">
      <li>Discovered via YAML frontmatter or invoked as slash commands (<code>/wayfinder</code>, <code>/prototype</code>, <code>/to-spec</code>, <code>/implement-spec</code>).</li>
      <li>Load detailed templates, checklists, and subagent workflows into context only when needed.</li>
    </ul>
  </li>
  <li>
    <strong class="text-white">MCP Servers (Model Context Protocol) — Live External Capabilities</strong>
    <ul class="list-disc pl-5 mt-1 space-y-1 text-gray-300">
      <li>Give the agent hands and eyes—such as <code>chrome-devtools-mcp</code> to launch a real headless browser, click UI buttons, and inspect console errors.</li>
    </ul>
  </li>
</ol>

<!--
Before we look at the first recording, let's distinguish the three customization levers in Antigravity. First, Rules in AGENTS.md or your global config are always-on guardrails—great for a one-line habit like asking clarifying questions one at a time. Second, Skills in .agents/skills are procedural playbooks loaded on demand when a task matches their description or when you type a slash command. Third, MCP servers connect the agent to live tools like Chrome DevTools for real browser testing.
-->

---
layout: default
---

<!-- S05 -->
<div class="flex items-center gap-3 mb-4">
  <h2 class="text-2xl font-bold m-0">Two Alternative Approaches + Productionizing with <code>google/skills</code></h2>
</div>

<div class="p-3 rounded-xl bg-gray-900/90 border border-cyan-500/30 text-xs font-mono text-cyan-200 mb-4">
  <span class="text-gray-400">Initial Prompt:</span> "I'd like to build a web application to track my audio electronics components inventory and DIY kit builds."
</div>

<ol class="space-y-3.5 text-sm text-gray-200 list-decimal pl-6 leading-relaxed">
  <li>
    <strong class="text-white">Approach 1: The Baseline Run (Rapid Vibe Coding — <code>01-baseline.cast</code>)</strong>
    <ul class="list-disc pl-5 mt-1 space-y-1 text-gray-300">
      <li>Guided only by the global one-question-at-a-time rule and ambient <code>modern-web-guidance</code>; no engineering workflow skills installed.</li>
      <li>Agent chooses Node.js + Express + <code>node:sqlite</code> + vanilla JS and builds a complete app in 16 minutes—then faces a fresh-context <code>/clear</code> review.</li>
    </ul>
  </li>
  <li>
    <strong class="text-white">Approach 2: The Skilled Run (<code>mattpocock/skills</code> — <code>02-skilled.cast</code>)</strong>
    <ul class="list-disc pl-5 mt-1 space-y-1 text-gray-300">
      <li>Same prompt + Canonical Q&amp;A, with 38 engineering workflow skills installed (<code>/wayfinder</code>, <code>/grilling</code>, <code>/domain-modeling</code>, <code>/prototype</code>, <code>/to-spec</code>, <code>/to-tickets</code>, <code>/implement-spec</code>, <code>/code-review</code>).</li>
      <li>Executes across 10 <code>/clear</code> sessions and 11 subagents to build a 41-test Rust + Axum + Maud + HTMX + local Turso application.</li>
    </ul>
  </li>
  <li>
    <strong class="text-white">Next Step: Productionizing with <code>google/skills</code> (+ Interactive Codelab)</strong>
    <ul class="list-disc pl-5 mt-1 space-y-1 text-gray-300">
      <li>Composes <code>mattpocock/skills</code> with <code>google/skills</code> to adapt the <code>InventoryRepository</code> seam for scale-to-zero <strong>Cloud Run</strong> + <strong>Cloud Firestore</strong>, culminating in a 5-lesson hands-on <strong>Codelab</strong>.</li>
    </ul>
  </li>
</ol>

<!--
Here is our roadmap. In Approach 1, the Baseline Run, we let Antigravity vibe-code the application with only our global clarifying-question rule. In Approach 2, the Skilled Run, we install Matt Pocock's workflow skills and watch how ten fresh-context sessions build a deeply modeled, test-driven Rust and HTMX application. Finally, we lead into how to layer in the google/skills skill set to take that architecture to scale-to-zero Cloud Run and Firestore, and walk through the interactive Codelab where you can try these workflows on your own ideas.
-->
