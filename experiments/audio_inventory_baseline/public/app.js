(() => {
  'use strict';

  // --- Application State ---
  const state = {
    activeView: 'inventory',
    metrics: {},
    components: [],
    projects: [],
    selectedProjectId: null,
    selectedProjectDetail: null,
    shopping: { linked_items: [], unlinked_items: [] }
  };

  // --- Modern Web Guidance: Dialog closedby="any" Fallback & returnValue reset ---
  function initDialogs() {
    const dialogs = document.querySelectorAll('dialog');
    for (const dialog of dialogs) {
      dialog.addEventListener('close', () => {
        dialog.returnValue = '';
      });
    }

    if (!('closedBy' in HTMLDialogElement.prototype)) {
      const outside = (e, dialog) => {
        const r = dialog.getBoundingClientRect();
        return e.clientX < r.left || e.clientX > r.right || e.clientY < r.top || e.clientY > r.bottom;
      };
      for (const dialog of document.querySelectorAll('dialog[closedby="any"]')) {
        let pressedOutside = false;
        dialog.addEventListener('pointerdown', (e) => {
          pressedOutside = e.target === dialog && outside(e, dialog);
        });
        dialog.addEventListener('pointerup', (e) => {
          if (pressedOutside && e.target === dialog && outside(e, dialog)) {
            dialog.close();
          }
          pressedOutside = false;
        });
      }
    }

    document.addEventListener('click', (e) => {
      const closeBtn = e.target.closest('[data-close-dialog]');
      if (closeBtn) {
        const dlgId = closeBtn.getAttribute('data-close-dialog');
        const dlg = document.getElementById(dlgId);
        if (dlg && dlg.open) dlg.close('cancel');
      }
    });
  }

  // --- Modern Web Guidance: Two-state Dark Mode Toggle (`dark-mode-toggle`) ---
  function initThemeToggle() {
    const meta = document.querySelector('meta[name="color-scheme"]');
    const btn = document.getElementById('theme-toggle-btn');
    if (!meta || !btn) return;

    const getSystemScheme = () =>
      window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light';

    const getRenderedScheme = () => {
      const content = meta.content;
      if (content === 'dark' || content === 'light') return content;
      return getSystemScheme();
    };

    btn.addEventListener('click', () => {
      const currentRendered = getRenderedScheme();
      const target = currentRendered === 'dark' ? 'light' : 'dark';
      const sys = getSystemScheme();

      try {
        if (target !== sys) {
          localStorage.setItem('color-scheme', target);
          meta.content = target;
        } else {
          localStorage.removeItem('color-scheme');
          meta.content = 'light dark';
        }
      } catch {
        meta.content = target !== sys ? target : 'light dark';
      }
      announceStatus(`Switched to ${target} theme`);
    });

    window.addEventListener('storage', (e) => {
      if (e.key === 'color-scheme') {
        meta.content = e.newValue ?? 'light dark';
      }
    });
  }

  // --- Modern Web Guidance: Form Validation After Interaction & ARIA Sync ---
  function initFormValidation() {
    const syncAria = (el) => {
      if (!el || typeof el.matches !== 'function') return;
      if (CSS.supports('selector(:user-invalid)')) {
        el.setAttribute('aria-invalid', el.matches(':user-invalid') ? 'true' : 'false');
      }
    };

    document.addEventListener('blur', (e) => syncAria(e.target), true);
    document.addEventListener('input', (e) => {
      if (e.target && e.target.hasAttribute && e.target.hasAttribute('aria-invalid')) {
        syncAria(e.target);
      }
    });

    if (!CSS.supports('selector(:user-invalid)')) {
      const dirtyState = new WeakMap();
      const updateState = (input) => {
        const isValid = input.checkValidity();
        input.classList.toggle('user-invalid-fallback', !isValid);
        input.classList.toggle('user-valid-fallback', isValid);
        if (!isValid) {
          input.setAttribute('aria-invalid', 'true');
        } else {
          input.removeAttribute('aria-invalid');
        }
      };

      const handleEvent = (event) => {
        const input = event.target;
        if (event.type === 'reset') {
          const controls = input.elements || [];
          for (const control of controls) {
            dirtyState.delete(control);
            control.classList.remove('user-invalid-fallback', 'user-valid-fallback');
            control.removeAttribute('aria-invalid');
          }
          return;
        }
        if (!input || !input.checkValidity) return;
        const st = dirtyState.get(input) || { hasInteracted: false, hasBlurred: false };
        if (event.type === 'input' || event.type === 'change') {
          st.hasInteracted = true;
          dirtyState.set(input, st);
          if (st.hasBlurred) updateState(input);
        } else if (event.type === 'blur') {
          st.hasBlurred = true;
          dirtyState.set(input, st);
          if (st.hasInteracted) updateState(input);
        }
      };

      document.addEventListener('blur', handleEvent, true);
      document.addEventListener('input', handleEvent);
      document.addEventListener('change', handleEvent);
      document.addEventListener('reset', handleEvent, true);
    }
  }

  // --- Accessible Status / Toast Announcer ---
  let toastTimer = null;
  function announceStatus(message, isError = false) {
    const region = document.getElementById('status-announcer');
    if (!region) return;
    region.textContent = message;
    region.classList.toggle('is-error', Boolean(isError));
    region.classList.add('is-visible');
    if (toastTimer) clearTimeout(toastTimer);
    toastTimer = setTimeout(() => {
      region.classList.remove('is-visible', 'is-error');
    }, 3600);
  }

  // --- Safe HTML Escaping & URL Validation ---
  function esc(val) {
    return String(val ?? '')
      .replace(/&/g, '&amp;')
      .replace(/</g, '&lt;')
      .replace(/>/g, '&gt;')
      .replace(/"/g, '&quot;')
      .replace(/'/g, '&#39;');
  }

  function safeHttpUrl(rawUrl) {
    const str = String(rawUrl ?? '').trim();
    if (!str) return null;
    try {
      const parsed = new URL(str, window.location.origin);
      if (parsed.protocol === 'http:' || parsed.protocol === 'https:') {
        return parsed.href;
      }
      return null;
    } catch {
      return null;
    }
  }

  function formatCurrency(num) {
    const n = Number(num) || 0;
    return '$' + n.toFixed(2);
  }

  // --- API Helper ---
  async function api(url, options = {}) {
    const res = await fetch(url, {
      headers: { 'Content-Type': 'application/json' },
      ...options,
      body: options.body ? JSON.stringify(options.body) : undefined
    });
    const data = await res.json().catch(() => ({}));
    if (!res.ok) {
      const errMsg = data.error || `Request failed (${res.status})`;
      announceStatus(errMsg, true);
      throw new Error(errMsg);
    }
    return data;
  }

  // --- Load All Workshop Data ---
  async function loadOverview(preserveProjectSelection = true) {
    const data = await api('/api/overview');
    state.metrics = data.metrics;
    state.components = data.components;
    state.projects = data.projects;
    state.shopping = data.shopping;

    renderMetrics();
    renderComponentsTable();
    renderProjectsList();
    renderShoppingTable();

    if (preserveProjectSelection && state.selectedProjectId) {
      const stillExists = state.projects.some((p) => p.id === state.selectedProjectId);
      if (stillExists) {
        await selectProject(state.selectedProjectId, false);
      } else {
        state.selectedProjectId = state.projects[0]?.id ?? null;
        if (state.selectedProjectId) await selectProject(state.selectedProjectId, false);
      }
    } else if (!state.selectedProjectId && state.projects.length > 0) {
      state.selectedProjectId = state.projects[0].id;
      await selectProject(state.selectedProjectId, false);
    } else {
      renderProjectWorkspace();
    }
  }

  // --- Render Metrics & Nav Badges ---
  function renderMetrics() {
    const m = state.metrics;
    document.getElementById('nav-badge-components').textContent = m.total_unique_parts ?? 0;
    document.getElementById('nav-badge-projects').textContent = m.active_builds ?? 0;
    document.getElementById('nav-badge-shopping').textContent = m.shopping_shortages_count ?? 0;

    document.getElementById('metric-unique-parts').textContent = m.total_unique_parts ?? 0;
    document.getElementById('metric-inventory-value').textContent = `${formatCurrency(m.total_inventory_value)} est. bench value`;

    document.getElementById('metric-units-available').textContent = m.total_units_available ?? 0;
    document.getElementById('metric-units-onhand').textContent = `${m.total_units_on_hand ?? 0} total`;
    document.getElementById('metric-units-allocated').textContent = `${m.total_units_allocated ?? 0} reserved in build kits`;

    document.getElementById('metric-active-builds').textContent = m.active_builds ?? 0;
    document.getElementById('metric-completed-builds').textContent = `${m.completed_builds ?? 0} completed builds`;

    document.getElementById('metric-low-stock').textContent = m.low_stock_count ?? 0;
    document.getElementById('metric-shopping-count').textContent = `${m.shopping_shortages_count ?? 0} items on order list`;
  }

  // --- View Switching ---
  function switchView(viewName) {
    state.activeView = viewName;

    for (const btn of document.querySelectorAll('.nav-tab')) {
      const isMatch = btn.getAttribute('data-view') === viewName;
      btn.classList.toggle('is-active', isMatch);
      if (isMatch) {
        btn.setAttribute('aria-current', 'page');
      } else {
        btn.removeAttribute('aria-current');
      }
    }

    document.getElementById('view-inventory').hidden = viewName !== 'inventory';
    document.getElementById('view-projects').hidden = viewName !== 'projects';
    document.getElementById('view-shopping').hidden = viewName !== 'shopping';

    const heading = document.getElementById('main-heading');
    const subheading = document.getElementById('main-subheading');
    const actionsContainer = document.getElementById('page-primary-actions');

    if (viewName === 'inventory') {
      document.title = 'Components Inventory | PatchBay & Bench';
      heading.textContent = 'Audio Components Inventory';
      subheading.textContent = 'Track Total On Hand, Allocated project parts, and Available bench stock.';
      actionsContainer.innerHTML = `
        <button type="button" id="btn-open-csv-import" class="btn btn-secondary">Import / Export CSV</button>
        <button type="button" id="btn-add-component" class="btn btn-primary">+ Add Component</button>
      `;
    } else if (viewName === 'projects') {
      document.title = 'DIY Kit Builds | PatchBay & Bench';
      heading.textContent = 'DIY Kit Builds & BOM Tracker';
      subheading.textContent = 'Match BOMs against bench stock, allocate parts into project baggies, and log calibration notes.';
      actionsContainer.innerHTML = `
        <button type="button" id="btn-add-project-top" class="btn btn-primary">+ New DIY Kit Build</button>
      `;
    } else if (viewName === 'shopping') {
      document.title = 'Order List & Shortages | PatchBay & Bench';
      heading.textContent = 'Parts Order List & Shortages';
      subheading.textContent = 'Consolidated BOM shortfalls across active builds and low-stock bench components.';
      actionsContainer.innerHTML = `
        <button type="button" id="btn-add-component" class="btn btn-secondary">+ Add Component</button>
      `;
    }
  }

  // --- VIEW 1: Render Components Inventory Table ---
  function getFilteredComponents() {
    const q = (document.getElementById('inv-search')?.value || '').trim().toLowerCase();
    const cat = document.getElementById('inv-category')?.value || 'ALL';
    const fpFilter = document.getElementById('inv-footprint')?.value || 'ALL';
    const stockFilter = document.getElementById('inv-stock-filter')?.value || 'ALL';

    return state.components.filter((c) => {
      if (cat !== 'ALL' && c.category !== cat) return false;

      if (fpFilter !== 'ALL') {
        const fp = (c.package_footprint || '').toUpperCase();
        const isSmd = fp.includes('SMD') || fp.includes('0603') || fp.includes('0805') || fp.includes('1206') || fp.includes('SOIC') || fp.includes('SSOP') || fp.includes('SOT');
        const isPanel = fp.includes('THONK') || fp.includes('ALPHA') || fp.includes('3PDT') || fp.includes('125B') || fp.includes('1590') || fp.includes('JACK') || fp.includes('LUG');
        if (fpFilter === 'SMD' && !isSmd) return false;
        if (fpFilter === 'PANEL' && !isPanel) return false;
        if (fpFilter === 'THT' && (isSmd || isPanel)) return false;
      }

      if (stockFilter === 'LOW' && !(c.min_qty > 0 && c.qty_available <= c.min_qty)) return false;
      if (stockFilter === 'ALLOCATED' && !(c.qty_allocated > 0)) return false;
      if (stockFilter === 'OUT' && c.qty_available > 0) return false;

      if (q) {
        const haystack = [
          c.name,
          c.value,
          c.category,
          c.package_footprint,
          c.specs,
          c.storage_location,
          c.supplier,
          c.supplier_sku,
          c.notes
        ]
          .join(' ')
          .toLowerCase();
        if (!haystack.includes(q)) return false;
      }

      return true;
    });
  }

  function renderComponentsTable() {
    const tbody = document.getElementById('components-tbody');
    if (!tbody) return;
    const list = getFilteredComponents();

    if (list.length === 0) {
      tbody.innerHTML = `
        <tr>
          <td colspan="9" class="text-muted">No components match the current filters.</td>
        </tr>
      `;
      return;
    }

    tbody.innerHTML = list
      .map((c) => {
        const isOut = c.qty_available === 0;
        const isLow = !isOut && c.min_qty > 0 && c.qty_available <= c.min_qty;

        let availBadge = `<span class="pill pill-ok">${c.qty_available} avail</span>`;
        if (isOut) {
          availBadge = `<span class="pill pill-danger">0 avail</span>`;
        } else if (isLow) {
          availBadge = `<span class="pill pill-warn">${c.qty_available} avail (Low &le; ${c.min_qty})</span>`;
        }

        const allocDisplay =
          c.qty_allocated > 0
            ? `<span class="pill pill-alloc">${c.qty_allocated} reserved</span>`
            : `<span class="text-muted">0</span>`;

        return `
          <tr data-comp-id="${c.id}">
            <th scope="row">
              <span class="part-primary">${esc(c.name)}</span>
              ${c.specs ? `<span class="part-secondary">${esc(c.specs)}</span>` : ''}
              ${c.notes ? `<span class="part-secondary">${esc(c.notes)}</span>` : ''}
            </th>
            <td><span class="val-chip">${esc(c.value || '—')}</span></td>
            <td>
              <span class="part-primary">${esc(c.package_footprint || '—')}</span>
              <span class="part-secondary">${esc(c.category)}</span>
            </td>
            <td>
              <span class="part-primary">${esc(c.storage_location || 'Unassigned')}</span>
            </td>
            <td>
              <span class="part-primary">${esc(c.supplier || '—')}</span>
              <span class="part-secondary">${esc(c.supplier_sku || '')} ${c.unit_cost ? `• ${formatCurrency(c.unit_cost)}` : ''}</span>
            </td>
            <td class="num-col">
              <div class="stock-stepper">
                <button type="button" class="btn btn-xs" data-stock-delta="-1" data-comp-id="${c.id}" aria-label="Decrease On Hand quantity for ${esc(c.name)}">&minus;</button>
                <strong>${c.qty_on_hand}</strong>
                <button type="button" class="btn btn-xs" data-stock-delta="1" data-comp-id="${c.id}" aria-label="Increase On Hand quantity for ${esc(c.name)}">+</button>
              </div>
            </td>
            <td class="num-col">${allocDisplay}</td>
            <td class="num-col">${availBadge}</td>
            <td class="actions-col">
              <div class="action-cluster">
                <button type="button" class="btn btn-sm btn-secondary" data-edit-comp="${c.id}" aria-label="Edit ${esc(c.name)}">Edit</button>
                <button type="button" class="btn btn-sm btn-danger-ghost" data-delete-comp="${c.id}" aria-label="Delete ${esc(c.name)}">Delete</button>
              </div>
            </td>
          </tr>
        `;
      })
      .join('');
  }

  // --- VIEW 2: Render Projects Sidebar & Active Project Workspace ---
  function statusPillClass(status) {
    if (status === 'Completed' || status === 'Ready to Build') return 'pill-ok';
    if (status === 'Soldering' || status === 'Testing & Calibration') return 'pill-alloc';
    if (status === 'Troubleshooting / On Hold') return 'pill-danger';
    return 'pill-warn';
  }

  function renderProjectsList() {
    const listEl = document.getElementById('projects-list');
    if (!listEl) return;

    const filter = document.getElementById('project-status-filter')?.value || 'ALL';
    const filtered = state.projects.filter((p) => {
      if (filter === 'ALL') return true;
      if (filter === 'ACTIVE') return p.status !== 'Completed';
      return p.status === filter;
    });

    if (filtered.length === 0) {
      listEl.innerHTML = `<li class="text-muted">No builds match this stage filter.</li>`;
      return;
    }

    listEl.innerHTML = filtered
      .map((p) => {
        const isSelected = p.id === state.selectedProjectId;
        const s = p.summary || {};
        const hasShortages = s.shortage_lines > 0;
        return `
          <li>
            <button
              type="button"
              class="project-card-btn ${isSelected ? 'is-selected' : ''}"
              data-select-project="${p.id}"
            >
              <div class="project-card-top">
                <span class="project-card-name">${esc(p.name)}</span>
                <span class="pill ${statusPillClass(p.status)}">${esc(p.status)}</span>
              </div>
              <div class="project-card-meta">
                <span>${esc(p.project_type)}</span>
                <span>
                  BOM: ${s.ready_lines || 0}/${s.total_lines || 0} ready
                  ${hasShortages ? `(${s.shortage_lines} short)` : ''}
                </span>
              </div>
              <div class="project-card-meta">
                <span>Allocated: ${s.total_parts_allocated || 0}/${s.total_parts_required || 0} pts</span>
                <span>Soldered: ${s.populated_lines || 0}/${s.total_lines || 0}</span>
              </div>
            </button>
          </li>
        `;
      })
      .join('');
  }

  async function selectProject(projectId, shouldAnnounce = false) {
    state.selectedProjectId = Number(projectId);
    state.selectedProjectDetail = await api(`/api/projects/${state.selectedProjectId}`);
    renderProjectsList();
    renderProjectWorkspace();
    if (shouldAnnounce && state.selectedProjectDetail) {
      announceStatus(`Loaded build: ${state.selectedProjectDetail.name}`);
    }
  }

  function renderProjectWorkspace() {
    const emptyEl = document.getElementById('project-empty-state');
    const paneEl = document.getElementById('project-detail-pane');
    const p = state.selectedProjectDetail;

    if (!p) {
      emptyEl.hidden = false;
      paneEl.hidden = true;
      return;
    }

    emptyEl.hidden = true;
    paneEl.hidden = false;

    document.getElementById('proj-detail-type').textContent = p.project_type;
    document.getElementById('proj-detail-specs').textContent = p.format_specs || '';
    document.getElementById('proj-detail-title').textContent = p.name;

    const vendorLine = document.getElementById('proj-detail-vendor');
    vendorLine.textContent = '';
    const validDocUrl = safeHttpUrl(p.doc_url);
    if (p.source_vendor) {
      const span = document.createElement('span');
      span.append('Source: ');
      const strong = document.createElement('strong');
      strong.textContent = p.source_vendor;
      span.appendChild(strong);
      vendorLine.appendChild(span);
    }
    if (validDocUrl) {
      if (p.source_vendor) {
        vendorLine.append(' • ');
      }
      const link = document.createElement('a');
      link.href = validDocUrl;
      link.target = '_blank';
      link.rel = 'noopener noreferrer';
      link.textContent = 'Build Docs / Schematic ↗';
      vendorLine.appendChild(link);
    }

    document.getElementById('proj-detail-notes').textContent = p.notes || '';
    document.getElementById('proj-status-select').value = p.status;

    // Summary stats
    const bom = p.bom || [];
    let readyLines = 0;
    let totalReq = 0;
    let totalAlloc = 0;
    let populatedLines = 0;
    let estCost = 0;

    for (const line of bom) {
      totalReq += line.qty_required;
      totalAlloc += line.qty_allocated;
      if (line.is_populated) populatedLines++;
      if (line.stock_status === 'allocated' || line.stock_status === 'consumed' || line.stock_status === 'in_stock') {
        readyLines++;
      }
      if (line.component && line.component.unit_cost) {
        estCost += line.component.unit_cost * line.qty_required;
      }
    }

    document.getElementById('proj-stat-readiness').textContent = `${readyLines} / ${bom.length} lines ready`;
    document.getElementById('proj-stat-allocated').textContent = `${totalAlloc} / ${totalReq} parts`;
    document.getElementById('proj-stat-populated').textContent = `${populatedLines} / ${bom.length} lines`;
    document.getElementById('proj-stat-cost').textContent = formatCurrency(estCost);

    renderBomTable(bom);
    renderBuildLogs(p.logs || []);
  }

  function renderBomTable(bom) {
    const tbody = document.getElementById('bom-tbody');
    if (!tbody) return;

    if (bom.length === 0) {
      tbody.innerHTML = `
        <tr>
          <td colspan="10" class="text-muted">No BOM lines yet. Click "+ Add BOM Line" or "Import BOM CSV" to add components.</td>
        </tr>
      `;
      return;
    }

    tbody.innerHTML = bom
      .map((item) => {
        let matchCell = '';
        if (item.component) {
          matchCell = `
            <span class="part-primary">${esc(item.component.name)}</span>
            <span class="part-secondary">Bin: ${esc(item.component.storage_location || 'Unassigned')}</span>
          `;
        } else if (item.suggested_component) {
          matchCell = `
            <span class="part-secondary text-warn">Unlinked (Suggested: ${esc(item.suggested_component.name)})</span>
            <button type="button" class="btn btn-xs btn-secondary" data-link-suggested="${item.id}" data-suggested-comp-id="${item.suggested_component.id}">
              Link Suggested (${item.suggested_component.qty_available} avail)
            </button>
          `;
        } else {
          matchCell = `
            <span class="part-secondary text-danger">Not in Inventory</span>
            <button type="button" class="btn btn-xs btn-secondary" data-create-from-bom="${item.id}">
              + Create in Inventory
            </button>
          `;
        }

        let statusBadge = '';
        if (item.stock_status === 'consumed') {
          statusBadge = `<span class="pill pill-ok">Soldered / Consumed</span>`;
        } else if (item.stock_status === 'allocated') {
          statusBadge = `<span class="pill pill-alloc">Allocated (${item.qty_allocated}/${item.qty_required})</span>`;
        } else if (item.stock_status === 'in_stock') {
          statusBadge = `<span class="pill pill-ok">In Stock (Can Allocate)</span>`;
        } else if (item.stock_status === 'partial') {
          statusBadge = `<span class="pill pill-warn">Short ${item.shortage_qty}</span>`;
        } else if (item.stock_status === 'out_of_stock') {
          statusBadge = `<span class="pill pill-danger">0 Available (Need ${item.still_needed_to_allocate})</span>`;
        } else {
          statusBadge = `<span class="pill pill-danger">Unlinked (Need ${item.qty_required})</span>`;
        }

        const benchAvail = item.component ? item.component.qty_available : '—';

        return `
          <tr class="${item.is_populated ? 'row-populated' : ''}" data-bom-id="${item.id}">
            <td class="check-col">
              <input
                type="checkbox"
                data-toggle-populated="${item.id}"
                aria-label="Mark ${esc(item.designators || item.part_name)} as soldered/populated"
                ${item.is_populated ? 'checked' : ''}
              >
            </td>
            <th scope="row">
              <code>${esc(item.designators || '—')}</code>
            </th>
            <td>
              <span class="val-chip">${esc(item.value || '—')}</span>
              <span class="part-primary">${esc(item.part_name)}</span>
              ${item.notes ? `<span class="part-secondary">${esc(item.notes)}</span>` : ''}
            </td>
            <td>${esc(item.package_footprint || '—')}</td>
            <td>${matchCell}</td>
            <td class="num-col"><strong>${item.qty_required}</strong></td>
            <td class="num-col">
              ${item.qty_allocated > 0 ? `<span class="pill pill-alloc">${item.qty_allocated}</span>` : '0'}
            </td>
            <td class="num-col">${benchAvail}</td>
            <td>${statusBadge}</td>
            <td class="actions-col">
              <div class="action-cluster">
                ${
                  item.component && !item.is_consumed
                    ? `<button type="button" class="btn btn-xs btn-secondary" data-open-allocate="${item.id}" aria-label="Allocate stock for ${esc(item.part_name)}">Allocate</button>`
                    : ''
                }
                <button type="button" class="btn btn-xs btn-ghost" data-edit-bom="${item.id}" aria-label="Edit BOM line ${esc(item.part_name)}">Edit</button>
                <button type="button" class="btn btn-xs btn-danger-ghost" data-delete-bom="${item.id}" aria-label="Delete BOM line ${esc(item.part_name)}">&times;</button>
              </div>
            </td>
          </tr>
        `;
      })
      .join('');
  }

  function renderBuildLogs(logs) {
    const listEl = document.getElementById('build-logs-list');
    if (!listEl) return;

    if (logs.length === 0) {
      listEl.innerHTML = `<li class="text-muted">No bench notes logged yet. Record calibration readings, resistor matching, or modifications above.</li>`;
      return;
    }

    listEl.innerHTML = logs
      .map((l) => {
        return `
          <li class="build-log-item">
            <div>
              <span class="pill pill-type">${esc(l.log_type)}</span>
              <span class="log-meta">${esc(l.created_at)}</span>
              <p style="margin-block-start: 0.35rem; margin-block-end: 0;">${esc(l.entry)}</p>
            </div>
            <button type="button" class="btn btn-xs btn-danger-ghost" data-delete-log="${l.id}" aria-label="Delete build log entry">&times;</button>
          </li>
        `;
      })
      .join('');
  }

  // --- VIEW 3: Render Shopping List / Shortages Table ---
  function renderShoppingTable() {
    const tbody = document.getElementById('shopping-tbody');
    const supSelect = document.getElementById('shopping-supplier-filter');
    if (!tbody || !supSelect) return;

    const allRows = [
      ...(state.shopping.linked_items || []),
      ...(state.shopping.unlinked_items || [])
    ];

    // Populate unique suppliers in filter dropdown
    const currentSup = supSelect.value || 'ALL';
    const suppliers = Array.from(new Set(allRows.map((r) => r.supplier || 'Unspecified'))).sort();
    supSelect.innerHTML =
      `<option value="ALL">All Suppliers (${allRows.length})</option>` +
      suppliers.map((s) => `<option value="${esc(s)}">${esc(s)}</option>`).join('');
    if (suppliers.includes(currentSup)) supSelect.value = currentSup;

    const filtered = allRows.filter((r) => {
      if (supSelect.value !== 'ALL' && (r.supplier || 'Unspecified') !== supSelect.value) {
        return false;
      }
      return true;
    });

    if (filtered.length === 0) {
      tbody.innerHTML = `
        <tr>
          <td colspan="8" class="text-muted">All active project BOMs are covered and no components are below minimum stock!</td>
        </tr>
      `;
      return;
    }

    tbody.innerHTML = filtered
      .map((item) => {
        let reasonHtml = '';
        if (item.type === 'unlinked_bom') {
          reasonHtml = `
            <span class="pill pill-danger">Unlinked BOM Requirement</span>
            <span class="part-secondary">${esc(item.project_name)} (${esc(item.designators || 'BOM')})</span>
          `;
        } else {
          const buildsText = (item.projects || [])
            .map((p) => `${esc(p.project_name)} (${p.needed} needed)`)
            .join(', ');
          reasonHtml = `
            ${item.bom_shortfall > 0 ? `<span class="pill pill-warn">Project Shortage (${item.bom_shortfall})</span>` : `<span class="pill pill-alloc">Low Stock Restock (Min: ${item.min_qty})</span>`}
            ${buildsText ? `<span class="part-secondary">Builds: ${buildsText}</span>` : ''}
          `;
        }

        const receiveBtn = item.component_id
          ? `<button type="button" class="btn btn-sm btn-accent" data-receive-comp="${item.component_id}" data-receive-qty="${item.qty_to_order}">
               + Receive ${item.qty_to_order} into Stock
             </button>`
          : `<button type="button" class="btn btn-sm btn-secondary" data-create-from-bom="${item.bom_item_id}" data-initial-qty="${item.qty_to_order}">
               + Add to Inventory (${item.qty_to_order})
             </button>`;

        return `
          <tr>
            <th scope="row">
              <span class="val-chip">${esc(item.value || '—')}</span>
              <span class="part-primary">${esc(item.part_name)}</span>
            </th>
            <td>${esc(item.package_footprint || '—')}</td>
            <td>${reasonHtml}</td>
            <td>
              <span class="part-primary">${esc(item.supplier || 'Unspecified')}</span>
              <span class="part-secondary">${esc(item.supplier_sku || '')}</span>
            </td>
            <td class="num-col">${item.qty_available ?? '—'}</td>
            <td class="num-col"><strong>${item.qty_to_order}</strong></td>
            <td class="num-col">${item.estimated_line_cost ? formatCurrency(item.estimated_line_cost) : '—'}</td>
            <td class="actions-col">${receiveBtn}</td>
          </tr>
        `;
      })
      .join('');
  }

  // --- Dialog Openers & Form Submissions ---
  async function openComponentDialog(compId = null) {
    const dlg = document.getElementById('dialog-component');
    const form = document.getElementById('form-component');
    const title = document.getElementById('dialog-component-title');
    const allocBox = document.getElementById('comp-allocations-info');
    form.reset();
    allocBox.hidden = true;
    allocBox.innerHTML = '';

    if (compId) {
      const comp = await api(`/api/components/${compId}`);
      title.textContent = `Edit Component: ${comp.name}`;
      document.getElementById('comp-id').value = comp.id;
      document.getElementById('comp-name').value = comp.name;
      document.getElementById('comp-category').value = comp.category;
      document.getElementById('comp-value').value = comp.value;
      document.getElementById('comp-footprint').value = comp.package_footprint;
      document.getElementById('comp-specs').value = comp.specs;
      document.getElementById('comp-qty').value = comp.qty_on_hand;
      document.getElementById('comp-min-qty').value = comp.min_qty;
      document.getElementById('comp-location').value = comp.storage_location;
      document.getElementById('comp-supplier').value = comp.supplier;
      document.getElementById('comp-sku').value = comp.supplier_sku;
      document.getElementById('comp-cost').value = comp.unit_cost;
      document.getElementById('comp-notes').value = comp.notes;

      if (comp.allocations && comp.allocations.length > 0) {
        allocBox.hidden = false;
        allocBox.innerHTML = `
          <strong>Project BOM Links (${comp.qty_allocated} currently reserved):</strong>
          <ul style="margin: 0.35rem 0 0; padding-inline-start: 1.2rem;">
            ${comp.allocations
              .map(
                (a) =>
                  `<li>${esc(a.project_name)} (${esc(a.designators || 'BOM')}): <strong>${a.qty_allocated} allocated</strong> of ${a.qty_required} req ${a.is_consumed ? '(Consumed)' : ''}</li>`
              )
              .join('')}
          </ul>
        `;
      }
    } else {
      title.textContent = 'Add Audio Component to Inventory';
      document.getElementById('comp-id').value = '';
    }

    dlg.showModal();
  }

  function openProjectDialog(project = null) {
    const dlg = document.getElementById('dialog-project');
    const form = document.getElementById('form-project');
    const title = document.getElementById('dialog-project-title');
    form.reset();

    if (project) {
      title.textContent = `Edit Build: ${project.name}`;
      document.getElementById('proj-id').value = project.id;
      document.getElementById('proj-name').value = project.name;
      document.getElementById('proj-type').value = project.project_type;
      document.getElementById('proj-specs').value = project.format_specs;
      document.getElementById('proj-status').value = project.status;
      document.getElementById('proj-vendor').value = project.source_vendor;
      document.getElementById('proj-doc-url').value = project.doc_url;
      document.getElementById('proj-notes').value = project.notes;
    } else {
      title.textContent = 'New DIY Kit Build';
      document.getElementById('proj-id').value = '';
    }

    dlg.showModal();
  }

  function openBomLineDialog(bomItem = null) {
    const dlg = document.getElementById('dialog-bom-line');
    const form = document.getElementById('form-bom-line');
    const title = document.getElementById('dialog-bom-title');
    const selectEl = document.getElementById('bom-component-select');
    const allocRow = document.getElementById('bom-allocate-immediately-row');
    form.reset();

    // Populate inventory dropdown
    selectEl.innerHTML =
      `<option value="">✨ Auto-Match by Value &amp; Footprint (or leave Unlinked)</option>` +
      state.components
        .map(
          (c) =>
            `<option value="${c.id}">[${esc(c.category)}] ${esc(c.value)} — ${esc(c.name)} (${esc(c.package_footprint)}) • ${c.qty_available} avail</option>`
        )
        .join('');

    if (bomItem) {
      title.textContent = `Edit BOM Line: ${bomItem.designators || bomItem.part_name}`;
      document.getElementById('bom-id').value = bomItem.id;
      selectEl.value = bomItem.component_id ? String(bomItem.component_id) : '';
      document.getElementById('bom-designators').value = bomItem.designators;
      document.getElementById('bom-value').value = bomItem.value;
      document.getElementById('bom-qty-required').value = bomItem.qty_required;
      document.getElementById('bom-part-name').value = bomItem.part_name;
      document.getElementById('bom-footprint').value = bomItem.package_footprint;
      document.getElementById('bom-notes').value = bomItem.notes;
      allocRow.hidden = true;
    } else {
      title.textContent = 'Add BOM Line to Project';
      document.getElementById('bom-id').value = '';
      allocRow.hidden = false;
    }

    dlg.showModal();
  }

  function openAllocateDialog(bomItem) {
    if (!bomItem || !bomItem.component) return;
    const dlg = document.getElementById('dialog-allocate');
    document.getElementById('alloc-bom-id').value = bomItem.id;
    document.getElementById('alloc-part-summary').textContent = `${bomItem.designators ? bomItem.designators + ' — ' : ''}${bomItem.component.name} (${bomItem.value})`;
    document.getElementById('alloc-req-count').textContent = bomItem.qty_required;
    document.getElementById('alloc-cur-count').textContent = bomItem.qty_allocated;
    document.getElementById('alloc-avail-count').textContent = bomItem.component.qty_available;

    const maxPossible = bomItem.qty_allocated + bomItem.component.qty_available;
    const input = document.getElementById('alloc-qty-input');
    input.max = String(maxPossible);
    input.value = String(Math.min(bomItem.qty_required, maxPossible));

    dlg.showModal();
  }

  // --- CSV Parser & Exporter ---
  function parseCsvText(text) {
    const raw = String(text || '').trim();
    if (!raw) return [];

    // Tokenize rows and cells while preserving quoted multiline values
    const rows = [];
    let currentRow = [];
    let cur = '';
    let inQuotes = false;

    for (let i = 0; i < raw.length; i++) {
      const ch = raw[i];
      if (ch === '"') {
        if (inQuotes && raw[i + 1] === '"') {
          cur += '"';
          i++;
        } else {
          inQuotes = !inQuotes;
        }
      } else if (ch === ',' && !inQuotes) {
        currentRow.push(cur.trim());
        cur = '';
      } else if ((ch === '\n' || ch === '\r') && !inQuotes) {
        if (ch === '\r' && raw[i + 1] === '\n') {
          i++;
        }
        currentRow.push(cur.trim());
        cur = '';
        if (currentRow.some((cell) => cell !== '')) {
          rows.push(currentRow);
        }
        currentRow = [];
      } else {
        cur += ch;
      }
    }
    currentRow.push(cur.trim());
    if (currentRow.some((cell) => cell !== '')) {
      rows.push(currentRow);
    }

    if (rows.length === 0) return [];

    const firstRow = rows[0];
    const headerKeywords = ['name', 'value', 'category', 'footprint', 'qty', 'designator', 'ref', 'part', 'supplier', 'sku'];
    const hasHeader = firstRow.some((col) => headerKeywords.some((kw) => col.toLowerCase().includes(kw)));

    if (hasHeader && rows.length > 1) {
      const headers = firstRow.map((h) => h.toLowerCase().replace(/[^a-z0-9]/g, '_'));
      return rows.slice(1).map((cols) => {
        const obj = {};
        headers.forEach((h, idx) => {
          obj[h] = cols[idx] ?? '';
        });
        return obj;
      });
    }

    // Positional fallback
    return rows.map((cols) => ({ _cols: cols }));
  }

  function downloadCsvFile(filename, rows) {
    const csvContent = rows
      .map((r) =>
        r
          .map((cell) => {
            const s = String(cell ?? '');
            return s.includes(',') || s.includes('"') || s.includes('\n')
              ? `"${s.replace(/"/g, '""')}"`
              : s;
          })
          .join(',')
      )
      .join('\n');

    const blob = new Blob([csvContent], { type: 'text/csv;charset=utf-8;' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = filename;
    document.body.appendChild(a);
    a.click();
    a.remove();
    URL.revokeObjectURL(url);
  }

  function openCsvDialog(mode = 'inventory') {
    const dlg = document.getElementById('dialog-csv');
    document.getElementById('csv-mode').value = mode;
    document.getElementById('csv-textarea').value = '';
    document.getElementById('csv-file-input').value = '';

    const title = document.getElementById('dialog-csv-title');
    const desc = document.getElementById('csv-mode-description');
    const autoAllocRow = document.getElementById('csv-bom-auto-alloc-row');

    if (mode === 'bom' && state.selectedProjectDetail) {
      title.textContent = `BOM CSV Import / Export — ${state.selectedProjectDetail.name}`;
      desc.textContent = 'Download this project BOM as CSV, or paste/upload BOM lines (Designators,Value,Part,Footprint,Qty,Notes) to auto-match against inventory.';
      autoAllocRow.hidden = false;
      document.getElementById('csv-textarea').placeholder =
        'Designators,Value,Part,Footprint,Qty,Notes\nR1-R4,100k,100k Metal Film Resistor,THT Axial 1/4W,4,1%\nU1,TL072,Dual Op-Amp,DIP-8,1,Use socket';
    } else {
      title.textContent = 'Components Inventory CSV Import / Export';
      desc.textContent = 'Export your full components inventory as CSV, or paste/upload CSV rows to add parts in bulk.';
      autoAllocRow.hidden = true;
      document.getElementById('csv-textarea').placeholder =
        'Name,Category,Value,Footprint,Specs,Qty,MinQty,Location,Supplier,SKU,UnitCost\n22kΩ Metal Film Resistor,Resistors,22k,THT Axial 1/4W,1% 0.25W,100,20,Resistor Binder A - p.4,Tayda,A-2206,0.02';
    }

    dlg.showModal();
  }

  // --- Wire Up Event Listeners ---
  function initEventListeners() {
    // Primary Nav Tabs
    document.querySelector('.primary-nav')?.addEventListener('click', (e) => {
      const tab = e.target.closest('.nav-tab');
      if (!tab) return;
      switchView(tab.getAttribute('data-view'));
    });

    // Top Page Action Buttons (delegated)
    document.getElementById('page-primary-actions')?.addEventListener('click', (e) => {
      if (e.target.closest('#btn-add-component')) openComponentDialog(null);
      if (e.target.closest('#btn-open-csv-import')) openCsvDialog('inventory');
      if (e.target.closest('#btn-add-project-top')) openProjectDialog(null);
    });

    // Inventory Search & Filters
    const invForm = document.getElementById('inventory-filter-form');
    invForm?.addEventListener('input', () => renderComponentsTable());
    invForm?.addEventListener('change', () => renderComponentsTable());
    invForm?.addEventListener('submit', (e) => e.preventDefault());

    // Inventory Table Actions (delegated)
    document.getElementById('components-tbody')?.addEventListener('click', async (e) => {
      const stepBtn = e.target.closest('[data-stock-delta]');
      if (stepBtn) {
        const id = Number(stepBtn.getAttribute('data-comp-id'));
        const delta = Number(stepBtn.getAttribute('data-stock-delta'));
        await api(`/api/components/${id}/stock`, { method: 'PATCH', body: { delta } });
        await loadOverview();
        announceStatus(`Updated stock quantity`);
        return;
      }

      const editBtn = e.target.closest('[data-edit-comp]');
      if (editBtn) {
        await openComponentDialog(Number(editBtn.getAttribute('data-edit-comp')));
        return;
      }

      const delBtn = e.target.closest('[data-delete-comp]');
      if (delBtn) {
        const id = Number(delBtn.getAttribute('data-delete-comp'));
        const comp = state.components.find((c) => c.id === id);
        if (confirm(`Delete component "${comp ? comp.name : id}" from inventory?`)) {
          await api(`/api/components/${id}`, { method: 'DELETE' });
          await loadOverview();
          announceStatus('Component deleted');
        }
      }
    });

    // Component Modal Form Submit
    document.getElementById('form-component')?.addEventListener('submit', async (e) => {
      e.preventDefault();
      const form = e.target;
      if (!form.checkValidity()) {
        form.reportValidity();
        return;
      }
      const id = document.getElementById('comp-id').value;
      const payload = {
        name: document.getElementById('comp-name').value,
        category: document.getElementById('comp-category').value,
        value: document.getElementById('comp-value').value,
        package_footprint: document.getElementById('comp-footprint').value,
        specs: document.getElementById('comp-specs').value,
        qty_on_hand: Number(document.getElementById('comp-qty').value),
        min_qty: Number(document.getElementById('comp-min-qty').value),
        storage_location: document.getElementById('comp-location').value,
        supplier: document.getElementById('comp-supplier').value,
        supplier_sku: document.getElementById('comp-sku').value,
        unit_cost: Number(document.getElementById('comp-cost').value),
        notes: document.getElementById('comp-notes').value
      };

      if (id) {
        await api(`/api/components/${id}`, { method: 'PUT', body: payload });
        announceStatus(`Updated component: ${payload.name}`);
      } else {
        await api('/api/components', { method: 'POST', body: payload });
        announceStatus(`Added component: ${payload.name}`);
      }

      document.getElementById('dialog-component').close('save');
      await loadOverview();
    });

    // Projects Sidebar Events
    document.getElementById('project-status-filter')?.addEventListener('change', () => renderProjectsList());
    document.getElementById('btn-new-project-inline')?.addEventListener('click', () => openProjectDialog(null));

    document.getElementById('projects-list')?.addEventListener('click', async (e) => {
      const card = e.target.closest('[data-select-project]');
      if (card) {
        await selectProject(Number(card.getAttribute('data-select-project')), true);
      }
    });

    // Project Detail Controls
    document.getElementById('proj-status-select')?.addEventListener('change', async (e) => {
      if (!state.selectedProjectId) return;
      const newStatus = e.target.value;
      await api(`/api/projects/${state.selectedProjectId}`, {
        method: 'PUT',
        body: { status: newStatus }
      });
      await loadOverview();
      announceStatus(`Project stage updated to ${newStatus}`);
    });

    document.getElementById('btn-edit-project')?.addEventListener('click', () => {
      if (state.selectedProjectDetail) openProjectDialog(state.selectedProjectDetail);
    });

    document.getElementById('btn-delete-project')?.addEventListener('click', async () => {
      const p = state.selectedProjectDetail;
      if (!p) return;
      if (confirm(`Delete project "${p.name}"? Any unconsumed allocated parts will return to Available stock.`)) {
        await api(`/api/projects/${p.id}`, { method: 'DELETE' });
        state.selectedProjectId = null;
        state.selectedProjectDetail = null;
        await loadOverview(false);
        announceStatus(`Deleted build: ${p.name}`);
      }
    });

    // Project Form Submit
    document.getElementById('form-project')?.addEventListener('submit', async (e) => {
      e.preventDefault();
      const form = e.target;
      if (!form.checkValidity()) {
        form.reportValidity();
        return;
      }

      const id = document.getElementById('proj-id').value;
      const payload = {
        name: document.getElementById('proj-name').value,
        project_type: document.getElementById('proj-type').value,
        format_specs: document.getElementById('proj-specs').value,
        status: document.getElementById('proj-status').value,
        source_vendor: document.getElementById('proj-vendor').value,
        doc_url: document.getElementById('proj-doc-url').value,
        notes: document.getElementById('proj-notes').value
      };

      if (id) {
        await api(`/api/projects/${id}`, { method: 'PUT', body: payload });
        state.selectedProjectId = Number(id);
        announceStatus(`Updated project: ${payload.name}`);
      } else {
        const created = await api('/api/projects', { method: 'POST', body: payload });
        state.selectedProjectId = created.id;
        announceStatus(`Created project: ${payload.name}`);
      }

      document.getElementById('dialog-project').close('save');
      await loadOverview();
    });

    // Project Allocation & BOM Toolbar Buttons
    document.getElementById('btn-allocate-all')?.addEventListener('click', async () => {
      if (!state.selectedProjectId) return;
      const res = await api(`/api/projects/${state.selectedProjectId}/allocate-all`, { method: 'POST' });
      await loadOverview();
      announceStatus(`Allocated ${res.units_newly_allocated} available units to this build`);
    });

    document.getElementById('btn-release-all')?.addEventListener('click', async () => {
      if (!state.selectedProjectId) return;
      await api(`/api/projects/${state.selectedProjectId}/release-all`, { method: 'POST' });
      await loadOverview();
      announceStatus('Released reserved parts back to available inventory');
    });

    document.getElementById('btn-consume-allocated')?.addEventListener('click', async () => {
      if (!state.selectedProjectId) return;
      const res = await api(`/api/projects/${state.selectedProjectId}/consume-allocated`, {
        method: 'POST',
        body: {}
      });
      await loadOverview();
      announceStatus(`Soldered & deducted ${res.consumed_units} allocated units from On Hand inventory`);
    });

    document.getElementById('btn-auto-match-bom')?.addEventListener('click', async () => {
      if (!state.selectedProjectId) return;
      const res = await api(`/api/projects/${state.selectedProjectId}/bom/auto-match`, { method: 'POST' });
      await loadOverview();
      announceStatus(`Smart BOM Matcher linked ${res.matched_count} unlinked lines`);
    });

    document.getElementById('btn-add-bom-line')?.addEventListener('click', () => openBomLineDialog(null));
    document.getElementById('btn-import-bom-csv')?.addEventListener('click', () => openCsvDialog('bom'));
    document.getElementById('btn-export-bom-csv')?.addEventListener('click', () => {
      const p = state.selectedProjectDetail;
      if (!p) return;
      const rows = [
        ['Designators', 'Value', 'Part', 'Footprint', 'QtyRequired', 'QtyAllocated', 'Status', 'InventoryBin', 'Notes'],
        ...(p.bom || []).map((b) => [
          b.designators,
          b.value,
          b.part_name,
          b.package_footprint,
          b.qty_required,
          b.qty_allocated,
          b.stock_status,
          b.component ? b.component.storage_location : '',
          b.notes
        ])
      ];
      downloadCsvFile(`${p.name.toLowerCase().replace(/[^a-z0-9]+/g, '-')}-bom.csv`, rows);
      announceStatus('Exported project BOM CSV');
    });

    // Auto-fill BOM fields when picking a component from dropdown
    document.getElementById('bom-component-select')?.addEventListener('change', (e) => {
      const compId = Number(e.target.value);
      if (!compId) return;
      const comp = state.components.find((c) => c.id === compId);
      if (!comp) return;
      const valInput = document.getElementById('bom-value');
      const nameInput = document.getElementById('bom-part-name');
      const fpInput = document.getElementById('bom-footprint');
      if (!valInput.value) valInput.value = comp.value;
      if (!nameInput.value) nameInput.value = comp.name;
      if (!fpInput.value) fpInput.value = comp.package_footprint;
    });

    // Submit BOM Line Modal
    document.getElementById('form-bom-line')?.addEventListener('submit', async (e) => {
      e.preventDefault();
      const form = e.target;
      if (!form.checkValidity()) {
        form.reportValidity();
        return;
      }
      if (!state.selectedProjectId) return;

      const bomId = document.getElementById('bom-id').value;
      const compSelectVal = document.getElementById('bom-component-select').value;
      const payload = {
        component_id: compSelectVal ? Number(compSelectVal) : null,
        designators: document.getElementById('bom-designators').value,
        value: document.getElementById('bom-value').value,
        qty_required: Number(document.getElementById('bom-qty-required').value),
        part_name: document.getElementById('bom-part-name').value || document.getElementById('bom-value').value,
        package_footprint: document.getElementById('bom-footprint').value,
        notes: document.getElementById('bom-notes').value,
        allocate_immediately: document.getElementById('bom-allocate-immediately').checked
      };

      if (bomId) {
        await api(`/api/bom/${bomId}`, { method: 'PUT', body: payload });
        announceStatus('Updated BOM line');
      } else {
        await api(`/api/projects/${state.selectedProjectId}/bom`, { method: 'POST', body: payload });
        announceStatus('Added BOM line to project');
      }

      document.getElementById('dialog-bom-line').close('save');
      await loadOverview();
    });

    // BOM Table Delegated Actions
    document.getElementById('bom-tbody')?.addEventListener('click', async (e) => {
      const popCheck = e.target.closest('[data-toggle-populated]');
      if (popCheck) {
        const bomId = Number(popCheck.getAttribute('data-toggle-populated'));
        await api(`/api/bom/${bomId}`, {
          method: 'PUT',
          body: { is_populated: popCheck.checked }
        });
        await loadOverview();
        return;
      }

      const linkSugBtn = e.target.closest('[data-link-suggested]');
      if (linkSugBtn) {
        const bomId = Number(linkSugBtn.getAttribute('data-link-suggested'));
        const compId = Number(linkSugBtn.getAttribute('data-suggested-comp-id'));
        await api(`/api/bom/${bomId}`, {
          method: 'PUT',
          body: { component_id: compId }
        });
        await loadOverview();
        announceStatus('Linked BOM line to inventory component');
        return;
      }

      const createBtn = e.target.closest('[data-create-from-bom]');
      if (createBtn) {
        const bomId = Number(createBtn.getAttribute('data-create-from-bom'));
        await api(`/api/bom/${bomId}/create-component`, {
          method: 'POST',
          body: { qty_on_hand: 0 }
        });
        await loadOverview();
        announceStatus('Created component in inventory and linked to BOM');
        return;
      }

      const allocBtn = e.target.closest('[data-open-allocate]');
      if (allocBtn) {
        const bomId = Number(allocBtn.getAttribute('data-open-allocate'));
        const item = state.selectedProjectDetail?.bom?.find((b) => b.id === bomId);
        if (item) openAllocateDialog(item);
        return;
      }

      const editBtn = e.target.closest('[data-edit-bom]');
      if (editBtn) {
        const bomId = Number(editBtn.getAttribute('data-edit-bom'));
        const item = state.selectedProjectDetail?.bom?.find((b) => b.id === bomId);
        if (item) openBomLineDialog(item);
        return;
      }

      const delBtn = e.target.closest('[data-delete-bom]');
      if (delBtn) {
        const bomId = Number(delBtn.getAttribute('data-delete-bom'));
        await api(`/api/bom/${bomId}`, { method: 'DELETE' });
        await loadOverview();
        announceStatus('Removed BOM line');
      }
    });

    // Allocate Single BOM Line Dialog Submit
    document.getElementById('form-allocate')?.addEventListener('submit', async (e) => {
      e.preventDefault();
      const bomId = Number(document.getElementById('alloc-bom-id').value);
      const qty = Number(document.getElementById('alloc-qty-input').value);
      await api(`/api/bom/${bomId}/allocate`, {
        method: 'POST',
        body: { qty_allocated: qty }
      });
      document.getElementById('dialog-allocate').close('save');
      await loadOverview();
      announceStatus(`Reserved ${qty} units for project`);
    });

    // Build Log Submit & Delete
    document.getElementById('build-log-form')?.addEventListener('submit', async (e) => {
      e.preventDefault();
      if (!state.selectedProjectId) return;
      const input = document.getElementById('log-entry-input');
      const entry = input.value.trim();
      if (!entry) return;
      const logType = document.getElementById('log-type-select').value;
      await api(`/api/projects/${state.selectedProjectId}/logs`, {
        method: 'POST',
        body: { log_type: logType, entry }
      });
      input.value = '';
      await selectProject(state.selectedProjectId, false);
      announceStatus('Added build log note');
    });

    document.getElementById('build-logs-list')?.addEventListener('click', async (e) => {
      const delBtn = e.target.closest('[data-delete-log]');
      if (delBtn) {
        const logId = Number(delBtn.getAttribute('data-delete-log'));
        await api(`/api/logs/${logId}`, { method: 'DELETE' });
        await selectProject(state.selectedProjectId, false);
        announceStatus('Deleted build log entry');
      }
    });

    // Shopping List Filter, Receive & Export
    document.getElementById('shopping-supplier-filter')?.addEventListener('change', () => renderShoppingTable());

    document.getElementById('shopping-tbody')?.addEventListener('click', async (e) => {
      const recBtn = e.target.closest('[data-receive-comp]');
      if (recBtn) {
        const compId = Number(recBtn.getAttribute('data-receive-comp'));
        const qty = Number(recBtn.getAttribute('data-receive-qty'));
        await api('/api/shopping-list/receive', {
          method: 'POST',
          body: { component_id: compId, qty_received: qty }
        });
        await loadOverview();
        announceStatus(`Received +${qty} units into On Hand stock`);
        return;
      }

      const createBtn = e.target.closest('[data-create-from-bom]');
      if (createBtn) {
        const bomId = Number(createBtn.getAttribute('data-create-from-bom'));
        const initQty = Number(createBtn.getAttribute('data-initial-qty') || 0);
        await api(`/api/bom/${bomId}/create-component`, {
          method: 'POST',
          body: { qty_on_hand: initQty }
        });
        await loadOverview();
        announceStatus(`Created component in inventory with ${initQty} on hand`);
      }
    });

    document.getElementById('btn-export-shopping-csv')?.addEventListener('click', () => {
      const allRows = [
        ...(state.shopping.linked_items || []),
        ...(state.shopping.unlinked_items || [])
      ];
      const csvRows = [
        ['PartName', 'Value', 'Footprint', 'Supplier', 'SKU', 'BenchAvailable', 'OrderQty', 'UnitCost', 'EstCost'],
        ...allRows.map((r) => [
          r.part_name,
          r.value,
          r.package_footprint,
          r.supplier,
          r.supplier_sku,
          r.qty_available ?? 0,
          r.qty_to_order,
          r.unit_cost ?? 0,
          r.estimated_line_cost ?? 0
        ])
      ];
      downloadCsvFile('audio-diy-shopping-list.csv', csvRows);
      announceStatus('Downloaded shopping list CSV');
    });

    // CSV Modal File Upload & Download & Import Submit
    document.getElementById('csv-file-input')?.addEventListener('change', async (e) => {
      const file = e.target.files?.[0];
      if (!file) return;
      const text = await file.text();
      document.getElementById('csv-textarea').value = text;
    });

    document.getElementById('btn-download-csv')?.addEventListener('click', () => {
      const mode = document.getElementById('csv-mode').value;
      if (mode === 'bom' && state.selectedProjectDetail) {
        document.getElementById('btn-export-bom-csv')?.click();
      } else {
        const rows = [
          ['Name', 'Category', 'Value', 'Footprint', 'Specs', 'QtyOnHand', 'QtyAllocated', 'QtyAvailable', 'MinQty', 'Location', 'Supplier', 'SKU', 'UnitCost'],
          ...state.components.map((c) => [
            c.name,
            c.category,
            c.value,
            c.package_footprint,
            c.specs,
            c.qty_on_hand,
            c.qty_allocated,
            c.qty_available,
            c.min_qty,
            c.storage_location,
            c.supplier,
            c.supplier_sku,
            c.unit_cost
          ])
        ];
        downloadCsvFile('patchbay-components-inventory.csv', rows);
        announceStatus('Downloaded full inventory CSV');
      }
    });

    document.getElementById('form-csv')?.addEventListener('submit', async (e) => {
      e.preventDefault();
      const mode = document.getElementById('csv-mode').value;
      const rawText = document.getElementById('csv-textarea').value;
      const parsed = parseCsvText(rawText);
      if (parsed.length === 0) {
        announceStatus('Please paste or select CSV rows first');
        return;
      }

      if (mode === 'bom' && state.selectedProjectId) {
        const normalizedBom = parsed.map((r) => {
          if (r._cols) {
            return {
              designators: r._cols[0] || '',
              value: r._cols[1] || '',
              part_name: r._cols[2] || r._cols[1] || '',
              package_footprint: r._cols[3] || '',
              qty_required: Number(r._cols[4] || 1),
              notes: r._cols[5] || ''
            };
          }
          return {
            designators: r.designators || r.ref || r.reference || '',
            value: r.value || r.val || '',
            part_name: r.part || r.part_name || r.name || r.description || r.value || '',
            package_footprint: r.footprint || r.package_footprint || r.package || '',
            qty_required: Number(r.qty || r.qty_required || r.quantity || 1),
            notes: r.notes || r.comment || ''
          };
        });

        const res = await api(`/api/projects/${state.selectedProjectId}/bom/import-csv`, {
          method: 'POST',
          body: {
            items: normalizedBom,
            auto_allocate: document.getElementById('csv-bom-auto-alloc').checked
          }
        });
        document.getElementById('dialog-csv').close('save');
        await loadOverview();
        announceStatus(`Imported ${res.imported} BOM lines (${res.matched_count} matched to inventory)`);
      } else {
        const normalizedComp = parsed.map((r) => {
          if (r._cols) {
            return {
              name: r._cols[0] || '',
              category: r._cols[1] || 'Other / Misc',
              value: r._cols[2] || '',
              package_footprint: r._cols[3] || '',
              specs: r._cols[4] || '',
              qty_on_hand: Number(r._cols[5] || 0),
              min_qty: Number(r._cols[6] || 0),
              storage_location: r._cols[7] || '',
              supplier: r._cols[8] || '',
              supplier_sku: r._cols[9] || '',
              unit_cost: Number(r._cols[10] || 0)
            };
          }
          return {
            name: r.name || r.part_name || r.part || r.value || '',
            category: r.category || 'Other / Misc',
            value: r.value || '',
            package_footprint: r.footprint || r.package_footprint || '',
            specs: r.specs || '',
            qty_on_hand: Number(r.qty || r.qty_on_hand || 0),
            min_qty: Number(r.minqty || r.min_qty || 0),
            storage_location: r.location || r.storage_location || '',
            supplier: r.supplier || '',
            supplier_sku: r.sku || r.supplier_sku || '',
            unit_cost: Number(r.unitcost || r.unit_cost || r.cost || 0)
          };
        });

        const res = await api('/api/components/import-csv', {
          method: 'POST',
          body: { items: normalizedComp }
        });
        document.getElementById('dialog-csv').close('save');
        await loadOverview();
        announceStatus(`Imported ${res.imported} components into inventory`);
      }
    });
  }

  // --- Bootstrap Application ---
  window.addEventListener('unhandledrejection', (e) => {
    const msg = e.reason?.message || 'An unexpected error occurred';
    announceStatus(msg, true);
    e.preventDefault();
  });

  document.addEventListener('DOMContentLoaded', async () => {
    initDialogs();
    initThemeToggle();
    initFormValidation();
    initEventListeners();
    await loadOverview();
  });
})();
