/* Shared Course Runtime: Prism.js (with keep-markup), Mermaid.js (dark), Interactive Exercises, Quizzes, Verification Checklists, and Simulators */
(function () {
  "use strict";

  function loadStylesheet(href) {
    var link = document.createElement("link");
    link.rel = "stylesheet";
    link.href = href;
    document.head.appendChild(link);
  }

  function loadScript(src, onLoad) {
    var script = document.createElement("script");
    script.src = src;
    script.defer = true;
    if (onLoad) {
      script.addEventListener("load", onLoad);
    }
    document.head.appendChild(script);
  }

  function initCodeBlockKeyboardScrolling() {
    var preBlocks = document.querySelectorAll("pre");
    preBlocks.forEach(function (pre) {
      if (pre.classList.contains("mermaid")) return;
      if (!pre.hasAttribute("tabindex")) {
        pre.setAttribute("tabindex", "0");
      }
      if (!pre.hasAttribute("role")) {
        pre.setAttribute("role", "region");
      }
      if (!pre.hasAttribute("aria-label")) {
        pre.setAttribute("aria-label", "Code snippet");
      }
    });
  }

  function initPrismAndMermaid() {
    loadStylesheet("https://cdn.jsdelivr.net/npm/prismjs@1.29.0/themes/prism-tomorrow.min.css");
    window.Prism = window.Prism || {};
    window.Prism.manual = true;

    loadScript("https://cdn.jsdelivr.net/npm/prismjs@1.29.0/prism.min.js", function () {
      loadScript("https://cdn.jsdelivr.net/npm/prismjs@1.29.0/plugins/keep-markup/prism-keep-markup.min.js", function () {
        var langs = ["clike", "javascript", "rust", "sql", "bash", "yaml", "json", "markdown"];
        var loaded = 0;
        langs.forEach(function (lang) {
          loadScript(
            "https://cdn.jsdelivr.net/npm/prismjs@1.29.0/components/prism-" + lang + ".min.js",
            function () {
              loaded += 1;
              if (loaded === langs.length && window.Prism && window.Prism.highlightAll) {
                window.Prism.highlightAll();
              }
            }
          );
        });
      });
    });

    if (document.querySelector(".mermaid")) {
      loadScript("https://cdn.jsdelivr.net/npm/mermaid@10.9.1/dist/mermaid.min.js", function () {
        if (window.mermaid) {
          window.mermaid.initialize({
            startOnLoad: true,
            theme: "dark",
            securityLevel: "loose"
          });
        }
      });
    }
  }

  function normalizeToken(val) {
    return (val || "").trim().replace(/\s+/g, " ");
  }

  function initExercises() {
    var cards = document.querySelectorAll(".exercise-card");
    cards.forEach(function (card) {
      var checkBtn = card.querySelector("[data-action='check-exercise']");
      var hintBtn = card.querySelector("[data-action='hint-exercise']");
      var feedbackEl = card.querySelector(".widget-feedback");
      var inputs = card.querySelectorAll("input.code-blank");
      var generalHint = card.getAttribute("data-hint") || "";
      var successMsg = card.getAttribute("data-success") || "All blanks matched!";

      function showFeedback(kind, text) {
        if (!feedbackEl) return;
        feedbackEl.className = "widget-feedback is-visible is-" + kind;
        feedbackEl.textContent = text;
      }

      if (checkBtn) {
        checkBtn.addEventListener("click", function () {
          var allCorrect = true;
          var firstErrorHint = "";
          inputs.forEach(function (input) {
            var rawAnswers = (input.getAttribute("data-answer") || "").split("||");
            var candidates = rawAnswers.map(normalizeToken);
            var userVal = normalizeToken(input.value);
            if (candidates.indexOf(userVal) !== -1) {
              input.classList.remove("is-wrong");
              input.classList.add("is-correct");
            } else {
              allCorrect = false;
              input.classList.remove("is-correct");
              input.classList.add("is-wrong");
              if (!firstErrorHint) {
                firstErrorHint = input.getAttribute("data-error-hint") || generalHint;
              }
            }
          });

          if (allCorrect) {
            showFeedback("success", successMsg);
          } else {
            showFeedback("error", firstErrorHint || "Check the highlighted blank(s) and try again.");
          }
        });
      }

      if (hintBtn) {
        hintBtn.addEventListener("click", function () {
          showFeedback("hint", generalHint || "Inspect the code listing above for the exact identifier or command.");
        });
      }
    });
  }

  function initQuizzes() {
    var quizCards = document.querySelectorAll(".quiz-card");
    quizCards.forEach(function (card) {
      var qBlocks = card.querySelectorAll(".quiz-question-block");
      qBlocks.forEach(function (block) {
        var buttons = block.querySelectorAll(".quiz-option-btn");
        var feedback = block.querySelector(".widget-feedback");
        var wrongHint = block.getAttribute("data-wrong-hint") || "";

        buttons.forEach(function (btn) {
          btn.addEventListener("click", function () {
            var isCorrect = btn.getAttribute("data-correct") === "true";
            var explain = btn.getAttribute("data-explain") || "";

            buttons.forEach(function (b) {
              b.classList.remove("is-correct", "is-wrong");
            });

            if (isCorrect) {
              btn.classList.add("is-correct");
              if (feedback) {
                feedback.className = "widget-feedback is-visible is-success";
                feedback.textContent = explain;
              }
            } else {
              btn.classList.add("is-wrong");
              if (feedback) {
                feedback.className = "widget-feedback is-visible is-error";
                feedback.textContent = explain + (wrongHint ? " Hint: " + wrongHint : "");
              }
            }
          });
        });
      });
    });
  }

  function initVerifyChecklists() {
    var boxes = document.querySelectorAll(".verify-card input[type='checkbox'][data-storage-key]");
    boxes.forEach(function (box) {
      var key = box.getAttribute("data-storage-key");
      try {
        if (window.localStorage.getItem(key) === "1") {
          box.checked = true;
        }
      } catch (_) {}

      box.addEventListener("change", function () {
        try {
          window.localStorage.setItem(key, box.checked ? "1" : "0");
        } catch (_) {}
      });
    });
  }

  function initSimulators() {
    var compMounts = document.querySelectorAll("[data-simulator='run-comparison-explorer']");
    var stages = [
      {
        id: "discovery",
        label: "1. Discovery & Requirements",
        baseline: "4 multiple-choice questions in 1m 40s (Storage/Stack, Key Features + 'Allocate parts' write-in, Allocation Model, Audio DIY Focus).",
        skilled: "11 Map-charting questions + 22 deep-dive domain/architecture questions across 6 /clear-separated /wayfinder sessions.",
        tradeoff: "Vibe coding gets you to a prototype in 16 minutes; /wayfinder takes ~37 minutes of structured Q&A to uncover hierarchical Storage Locations and multi-location Stock Lots before coding."
      },
      {
        id: "artifacts",
        label: "2. What Survives /clear?",
        baseline: "0 markdown docs (no GLOSSARY.md, no ADRs, no spec.md). All domain rules live only in chat history or raw SQL/JS.",
        skilled: "GLOSSARY.md (11 terms), ADR 0001, .scratch/audio-inventory/map.md, 5 decision records, prototype-ui.html, spec.md, and 8 vertical-slice tickets.",
        tradeoff: "If the prototype is disposable, zero docs is fine. If you plan to maintain or extend the app across sessions, durable repo artifacts prevent context amnesia."
      },
      {
        id: "ui-design",
        label: "3. UI Exploration",
        baseline: "Agent picked a single 3-tab workbench layout unprompted during its one-shot build (guided by ambient modern-web-guidance).",
        skilled: "Ticket 05 (/prototype) built a single HTML file with 3 switchable layouts (?variant=a|b|c) so the human could test-drive and pick Variant A in Chrome first.",
        tradeoff: "/prototype borrows the best part of vibe coding (fast visual feedback) without committing to backend templates prematurely."
      },
      {
        id: "execution",
        label: "4. Build & Verification",
        baseline: "Single 14-minute turn writing ~2,500 lines of JS/HTML/CSS with 0 automated test files. Manual '/clear -> Please review the codebase' found 8 bugs.",
        skilled: "/implement-spec spawned 11 subagents across isolated git worktrees (4 in parallel for Tickets 04-07), ran 41 Turso tests, and executed parallel 2-axis /code-review.",
        tradeoff: "Worktree TDD + 2-axis code review caught schema drift, Installed Stock Permanence, and CSV transaction atomicity automatically before completion."
      }
    ];

    compMounts.forEach(function (mount) {
      var listEl = mount.querySelector(".sim-tree-list");
      var inspectorEl = mount.querySelector(".sim-inspector");
      if (!listEl || !inspectorEl) return;

      function selectStage(item) {
        var allBtns = listEl.querySelectorAll(".sim-node-btn");
        allBtns.forEach(function (b) {
          b.setAttribute("aria-pressed", b.getAttribute("data-stage-id") === item.id ? "true" : "false");
        });

        inspectorEl.innerHTML =
          "<h4>" + item.label + "</h4>" +
          "<dl class='sim-kv'>" +
          "<dt>Baseline Run</dt><dd>" + item.baseline + "</dd>" +
          "<dt>Skilled Run</dt><dd>" + item.skilled + "</dd>" +
          "<dt>When to Choose</dt><dd><strong>" + item.tradeoff + "</strong></dd>" +
          "</dl>";
      }

      stages.forEach(function (item, idx) {
        var btn = document.createElement("button");
        btn.type = "button";
        btn.className = "sim-node-btn";
        btn.setAttribute("data-stage-id", item.id);
        btn.setAttribute("aria-pressed", idx === 0 ? "true" : "false");
        btn.textContent = item.label;
        btn.addEventListener("click", function () {
          selectStage(item);
        });
        listEl.appendChild(btn);
      });

      selectStage(stages[0]);
    });

    var cloudMounts = document.querySelectorAll("[data-simulator='cloud-seam-simulator']");
    var cloudLayers = [
      {
        id: "database-onboarding",
        label: "1. Database Selection (cloud-databases-onboarding)",
        localState: "Local Turso / SQLite file (data/inventory.db) on developer laptop.",
        cloudTarget: "Cloud Firestore (Native Mode) in us-central1 — serverless document store billed per operation with 50k reads / 20k writes free daily.",
        skillRule: "cloud-databases-onboarding + google-cloud-waf-cost-optimization avoid always-on Cloud SQL/AlloyDB VMs for spiky personal workloads so idle storage/compute stays $0/month."
      },
      {
        id: "repository-seam",
        label: "2. Persistence Seam (/codebase-design + /tdd)",
        localState: "5 deep domain modules (spec_parser, catalog, builds, imports, shopping_list) tested against ephemeral :memory: Turso.",
        cloudTarget: "Repository trait seam (TursoRepository for fast offline cargo test + FirestoreRepository using atomic transactions for Stock Lots & BOM Fulfillment).",
        skillRule: "Keeping the local/in-memory adapter behind a repository seam means you never need live GCP credentials just to run unit tests."
      },
      {
        id: "cloud-run-contract",
        label: "3. Container Contract (cloud-run-basics)",
        localState: "Binds to 127.0.0.1:3000 in local development.",
        cloudTarget: "Binds to 0.0.0.0:$PORT (default 8080) inside a multi-stage Dockerfile deployed with --min-instances=0 --region=us-central1.",
        skillRule: "CRITICAL RULE in cloud-run-basics: binding to 127.0.0.1 or ignoring $PORT crashes Cloud Run containers on boot."
      },
      {
        id: "gcloud-guardrails",
        label: "4. CLI Validation Guardrails (gcloud)",
        localState: "Ad-hoc shell commands or LLM parametric guesses.",
        cloudTarget: "4-Step Plan: (1) gcloud help <leaf_command>, (2) verify --dry-run & explicit --project/--region, (3) --filter/--format data reduction, (4) human approval for IAM/APIs.",
        skillRule: "Prevents hallucinated flags, interactive TTY hangs (via --quiet + explicit --region), and context-window overflow on gcloud list commands."
      }
    ];

    cloudMounts.forEach(function (mount) {
      var listEl = mount.querySelector(".sim-tree-list");
      var inspectorEl = mount.querySelector(".sim-inspector");
      if (!listEl || !inspectorEl) return;

      function selectLayer(item) {
        var allBtns = listEl.querySelectorAll(".sim-node-btn");
        allBtns.forEach(function (b) {
          b.setAttribute("aria-pressed", b.getAttribute("data-layer-id") === item.id ? "true" : "false");
        });

        inspectorEl.innerHTML =
          "<h4>" + item.label + "</h4>" +
          "<dl class='sim-kv'>" +
          "<dt>Local Skilled Run</dt><dd>" + item.localState + "</dd>" +
          "<dt>Cloud Production</dt><dd>" + item.cloudTarget + "</dd>" +
          "<dt>Skill Guardrail</dt><dd><strong>" + item.skillRule + "</strong></dd>" +
          "</dl>";
      }

      cloudLayers.forEach(function (item, idx) {
        var btn = document.createElement("button");
        btn.type = "button";
        btn.className = "sim-node-btn";
        btn.setAttribute("data-layer-id", item.id);
        btn.setAttribute("aria-pressed", idx === 0 ? "true" : "false");
        btn.textContent = item.label;
        btn.addEventListener("click", function () {
          selectLayer(item);
        });
        listEl.appendChild(btn);
      });

      selectLayer(cloudLayers[0]);
    });
  }

  document.addEventListener("DOMContentLoaded", function () {
    initCodeBlockKeyboardScrolling();
    initPrismAndMermaid();
    initExercises();
    initQuizzes();
    initVerifyChecklists();
    initSimulators();
  });
})();
