---
layout: center
class: text-center
---

<!-- S11 -->
<div class="space-y-4">
  <h1 class="text-4xl font-extrabold text-white">
    The Skilled Run (Part 1): Wayfinding &amp; Domain Modeling
  </h1>
  <p class="text-xl font-mono text-cyan-300">
    mattpocock/skills &middot; /wayfinder &middot; /grilling &middot; /domain-modeling
  </p>
  <p class="text-base text-gray-400 max-w-xl mx-auto">
    Turning fleeting chat assumptions into durable on-disk specifications before writing a single line of application code
  </p>
</div>

<!--
Now let's look at Run 2: the Skilled Run. We start with the exact same project goal, but first we install Matt Pocock's agent skills collection and drive the work through /wayfinder, /grilling, /domain-modeling, /prototype, /to-spec, and /to-tickets.
-->

---
layout: default
---

<!-- S12 -->
<div class="flex items-center gap-3 mb-6">
  <h2 class="text-2xl font-bold m-0">How <code>/wayfinder</code> Solves Context Window Amnesia</h2>
</div>

<ol class="space-y-4 text-sm text-gray-200 list-decimal pl-6 mt-5 leading-relaxed">
  <li>
    <strong class="text-white">Plan Decisions on Disk, One Ticket per Session</strong>
    <ul class="list-disc pl-5 mt-1 space-y-1 text-gray-300">
      <li>Instead of holding an entire system design inside one giant chat thread, <code>/wayfinder</code> charts a shared map (<code>map.md</code>) and slices open questions into focused decision tickets.</li>
      <li>Each session resolves <strong>one</strong> decision ticket, writes the resolution to disk, and is followed by <code>/clear</code>.</li>
    </ul>
  </li>
  <li>
    <strong class="text-white">Active Domain Modeling (<code>/grilling</code> + <code>/domain-modeling</code>)</strong>
    <ul class="list-disc pl-5 mt-1 space-y-1 text-gray-300">
      <li>Probes real-world edge cases, challenges overloaded terms, and updates <code>GLOSSARY.md</code> and <code>docs/adr/</code> inline as decisions crystallize.</li>
    </ul>
  </li>
  <li>
    <strong class="text-white">Visual Prototyping &amp; Vertical Slice Specifications (<code>/prototype</code> &rarr; <code>/to-spec</code> &rarr; <code>/to-tickets</code>)</strong>
    <ul class="list-disc pl-5 mt-1 space-y-1 text-gray-300">
      <li>Generates throwaway HTML UI variants to react to before locking <code>spec.md</code> and dependency-ordered implementation tickets.</li>
    </ul>
  </li>
</ol>

<!--
Why does /wayfinder work so well across /clear boundaries? Because it refuses to keep decisions trapped in chat memory. First, it charts a map of open architectural and domain questions on disk. Then, in each session, it combines /grilling and /domain-modeling to resolve a single ticket, immediately updating GLOSSARY.md and ADRs before you run /clear.
-->

---
layout: cast-full
title: "Skilled Run (Session 1): Installing mattpocock/skills & Charting the Wayfinder Map"
cast: /casts/02-skilled.cast
startChapter: 1
endChapter: 28
showChapterBar: true
autoPlay: false
speed: 2.0
idleTimeLimit: 2.0
fit: both
---

<!-- S13 -->
<!--
Let's watch Session 1 of the Skilled Run. At stop points 1 and 2, we run npx skills add mattpocock/skills to install 38 skills into .agents/skills. Then at stop point 5, we invoke /wayfinder. Notice how the agent interviews us across 11 breadth-first questions—covering BOM scope, structured schemas, single-user local operation, and our standing preference for Rust, HTMX, and local Turso—before writing GLOSSARY.md, ADR 0001, map.md, and 5 decision tickets at stop point 28.
-->

---
layout: cast-full
title: "Skilled Run (Sessions 2–5): Resolving Decision Tickets 01–04 Across /clear"
cast: /casts/02-skilled.cast
startChapter: 29
endChapter: 80
showChapterBar: true
autoPlay: false
speed: 2.0
idleTimeLimit: 2.0
fit: both
---

<!-- S14 -->
<!--
Here in Sessions 2 through 5, we clear the context window before every single ticket and simply type /wayfinder. Each fresh session reads map.md and GLOSSARY.md from disk, claims the next unblocked decision ticket, and grills us on the edge cases: Ticket 01 resolves the 9-category taxonomy and multi-location Stock Lots; Ticket 02 resolves the 5-state BOM fulfillment lifecycle; Ticket 03 resolves CSV import and multi-build shopping list shortfall math; and Ticket 04 locks the 5 deep Rust modules and Turso database architecture.
-->

---
layout: default
---

<!-- S15 -->
<div class="flex items-center gap-3 mb-6">
  <h2 class="text-2xl font-bold m-0">Domain Nuance Surfaced Only by <code>/domain-modeling</code></h2>
</div>

<ol class="space-y-4 text-sm text-gray-200 list-decimal pl-6 mt-5 leading-relaxed">
  <li>
    <strong class="text-white">Hierarchical <code>Storage Location</code> + Multi-Location <code>Stock Lots</code></strong>
    <ul class="list-disc pl-5 mt-1 space-y-1 text-gray-300">
      <li>Baseline used a single flat text column; <code>/grilling</code> asked: <em>"What happens when you have 20 TL072s in Cabinet A and another 50 on a tape reel in Box C?"</em></li>
    </ul>
  </li>
  <li>
    <strong class="text-white">5-State <code>Fulfillment Status</code> &amp; Split BOM Allocations</strong>
    <ul class="list-disc pl-5 mt-1 space-y-1 text-gray-300">
      <li>Models <code>Unresolved</code> &rarr; <code>Matched</code> &rarr; <code>Allocated</code> &rarr; <code>Shortage</code> &rarr; <code>Installed</code>, including splitting a BOM line across two substitute parts.</li>
    </ul>
  </li>
  <li>
    <strong class="text-white">Cross-Build <code>Net Shortfall</code> Shopping List Math</strong>
    <ul class="list-disc pl-5 mt-1 space-y-1 text-gray-300">
      <li>Computes shortages across <em>all</em> selected builds simultaneously so shared stock isn't double-counted when ordering from Mouser or Tayda.</li>
    </ul>
  </li>
</ol>

<!--
Look at the domain richness that emerged from those four Wayfinder sessions—none of which was in our initial prompt! When the agent asked what happens if we have two bags of the same op-amp in different drawers, we arrived at hierarchical Storage Locations and Stock Lots. When it probed partial builds and substitutions, we arrived at a 5-state BOM fulfillment model and cross-build Net Shortfall math.
-->

---
layout: cast-full
title: "Skilled Run (Sessions 6–8): 3-Variant HTML /prototype, /to-spec & /to-tickets"
cast: /casts/02-skilled.cast
startChapter: 81
endChapter: 102
showChapterBar: true
autoPlay: false
speed: 2.0
idleTimeLimit: 2.0
fit: both
---

<!-- S16 -->
<!--
In Sessions 6, 7, and 8, we finish planning. At stop points 82 through 87, Ticket 05 invokes the /prototype skill to build a self-contained HTML file with three interactive layout variants—Variant A Workbench Split-Pane, Variant B Dense Datasheet Grid, and Variant C Kanban Cards—so we can inspect them in a browser and pick Variant A. Then at stop point 94 we run /to-spec after /clear to compile spec.md, and at stop point 99 we run /to-tickets to slice the build into 8 vertical implementation tickets.
-->

---
layout: skill-breakdown
way: 2
title: Durable Intent Artifacts Committed Before Code
---

<!-- S17 -->
<div class="space-y-4">

```text
experiments/audio_inventory_skilled/
├── GLOSSARY.md                                  # 14 canonical domain terms + Avoid lists
├── docs/adr/0001-rust-htmx-local-turso-stack.md # Architecture Decision Record
└── .scratch/audio-inventory/
    ├── map.md                                   # Wayfinder index + 5 closed decision links
    ├── prototype-ui.html                        # Interactive 3-variant workbench prototype
    ├── spec.md                                  # 213-line consolidated system specification
    └── issues/0001..0008-*.md                   # 8 dependency-ordered vertical build slices
```

  <div class="grid grid-cols-2 gap-4 text-xs pt-2">
    <div class="p-3 rounded-lg bg-gray-900/80 border border-gray-800">
      <span class="text-amber-300 font-semibold">Zero Context Drift Across 8 /clear Boundaries:</span> Every subsequent session reads <code>GLOSSARY.md</code>, <code>ADR 0001</code>, and <code>spec.md</code> from disk in seconds instead of re-deriving intent from chat history.
    </div>
    <div class="p-3 rounded-lg bg-gray-900/80 border border-gray-800">
      <span class="text-amber-300 font-semibold">Deep Module Boundaries Pre-Defined:</span> <code>spec.md</code> locks 5 deep Rust modules (<code>catalog</code>, <code>spec_parser</code>, <code>builds</code>, <code>shopping_list</code>, <code>imports</code>) before implementation starts.
    </div>
  </div>
</div>

<!--
Pause and look at what sits on disk at the end of Session 8, before a single line of Rust application code has been written: a 14-term GLOSSARY.md, ADR 0001, a 3-variant HTML UI prototype, a 213-line specification, and 8 vertical slice tickets. Any engineer—or any fresh agent session after /clear—can pick up this repository and know every invariant.
-->
