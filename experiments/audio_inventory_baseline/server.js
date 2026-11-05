const path = require('node:path');
const express = require('express');
const {
  db,
  findBestComponentMatch,
  getAllComponents,
  getComponentById,
  getAllProjects,
  getProjectDetail,
  getShoppingList
} = require('./db');

const app = express();
const PORT = process.env.PORT || 3000;

// Baseline preventative security headers
app.use((req, res, next) => {
  res.setHeader('X-Content-Type-Options', 'nosniff');
  res.setHeader('Referrer-Policy', 'strict-origin-when-cross-origin');
  res.setHeader('X-Frame-Options', 'DENY');
  res.setHeader('Cross-Origin-Opener-Policy', 'same-origin-allow-popups');
  res.setHeader('Cross-Origin-Resource-Policy', 'same-origin');
  res.setHeader('Permissions-Policy', 'camera=(), geolocation=(), microphone=(), focus-without-user-activation=()');
  res.setHeader(
    'Content-Security-Policy',
    "object-src 'none'; base-uri 'none'; frame-ancestors 'none'; form-action 'self'"
  );
  next();
});

app.use(express.json({ limit: '5mb' }));
app.use(express.static(path.join(__dirname, 'public')));

function sanitizeHttpUrl(rawUrl) {
  const str = String(rawUrl ?? '').trim();
  if (!str) return '';
  try {
    const parsed = new URL(str);
    if (parsed.protocol === 'http:' || parsed.protocol === 'https:') {
      return parsed.href;
    }
    return null;
  } catch {
    return null;
  }
}

/**
 * Ensure unconsumed BOM allocations for a component never exceed its new qty_on_hand.
 */
function clampComponentAllocations(componentId, newQtyOnHand) {
  const rows = db.prepare(`
    SELECT id, qty_allocated FROM bom_items
    WHERE component_id = ? AND is_consumed = 0 AND qty_allocated > 0
    ORDER BY id DESC
  `).all(componentId);

  const totalAllocated = rows.reduce((sum, r) => sum + r.qty_allocated, 0);
  let excess = totalAllocated - newQtyOnHand;
  if (excess <= 0) return;

  const updateBom = db.prepare('UPDATE bom_items SET qty_allocated = ? WHERE id = ?');
  for (const row of rows) {
    if (excess <= 0) break;
    const deduct = Math.min(row.qty_allocated, excess);
    updateBom.run(row.qty_allocated - deduct, row.id);
    excess -= deduct;
  }
}

// --- Overview / Dashboard Summary ---
app.get('/api/overview', (req, res) => {
  const components = getAllComponents();
  const projects = getAllProjects();
  const shopping = getShoppingList();

  let totalUniqueParts = components.length;
  let totalUnitsOnHand = 0;
  let totalUnitsAllocated = 0;
  let totalInventoryValue = 0;
  let lowStockCount = 0;

  for (const c of components) {
    totalUnitsOnHand += c.qty_on_hand;
    totalUnitsAllocated += c.qty_allocated;
    totalInventoryValue += (c.qty_on_hand || 0) * (c.unit_cost || 0);
    if (c.min_qty > 0 && c.qty_available <= c.min_qty) {
      lowStockCount++;
    }
  }

  const activeBuilds = projects.filter((p) => p.status !== 'Completed').length;
  const completedBuilds = projects.filter((p) => p.status === 'Completed').length;

  res.json({
    metrics: {
      total_unique_parts: totalUniqueParts,
      total_units_on_hand: totalUnitsOnHand,
      total_units_allocated: totalUnitsAllocated,
      total_units_available: Math.max(0, totalUnitsOnHand - totalUnitsAllocated),
      total_inventory_value: Math.round(totalInventoryValue * 100) / 100,
      low_stock_count: lowStockCount,
      active_builds: activeBuilds,
      completed_builds: completedBuilds,
      shopping_shortages_count: shopping.linked_items.length + shopping.unlinked_items.length
    },
    components,
    projects,
    shopping
  });
});

// --- Components Routes ---
app.get('/api/components', (req, res) => {
  res.json(getAllComponents());
});

app.get('/api/components/:id', (req, res) => {
  const comp = getComponentById(Number(req.params.id));
  if (!comp) return res.status(404).json({ error: 'Component not found' });
  res.json(comp);
});

app.post('/api/components', (req, res) => {
  const {
    name,
    category = 'Other / Misc',
    value = '',
    package_footprint = '',
    specs = '',
    qty_on_hand = 0,
    min_qty = 0,
    storage_location = '',
    supplier = '',
    supplier_sku = '',
    unit_cost = 0,
    notes = ''
  } = req.body;

  if (!name || !String(name).trim()) {
    return res.status(400).json({ error: 'Component name is required' });
  }

  const stmt = db.prepare(`
    INSERT INTO components (
      name, category, value, package_footprint, specs,
      qty_on_hand, min_qty, storage_location, supplier, supplier_sku, unit_cost, notes
    ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
  `);

  const result = stmt.run(
    String(name).trim(),
    String(category).trim() || 'Other / Misc',
    String(value).trim(),
    String(package_footprint).trim(),
    String(specs).trim(),
    Math.max(0, parseInt(qty_on_hand, 10) || 0),
    Math.max(0, parseInt(min_qty, 10) || 0),
    String(storage_location).trim(),
    String(supplier).trim(),
    String(supplier_sku).trim(),
    Math.max(0, parseFloat(unit_cost) || 0),
    String(notes).trim()
  );

  res.status(201).json(getComponentById(Number(result.lastInsertRowid)));
});

app.put('/api/components/:id', (req, res) => {
  const id = Number(req.params.id);
  const existing = db.prepare('SELECT * FROM components WHERE id = ?').get(id);
  if (!existing) return res.status(404).json({ error: 'Component not found' });

  const {
    name = existing.name,
    category = existing.category,
    value = existing.value,
    package_footprint = existing.package_footprint,
    specs = existing.specs,
    qty_on_hand = existing.qty_on_hand,
    min_qty = existing.min_qty,
    storage_location = existing.storage_location,
    supplier = existing.supplier,
    supplier_sku = existing.supplier_sku,
    unit_cost = existing.unit_cost,
    notes = existing.notes
  } = req.body;

  if (!name || !String(name).trim()) {
    return res.status(400).json({ error: 'Component name is required' });
  }

  const newQtyOnHand = Math.max(0, parseInt(qty_on_hand, 10) || 0);

  db.exec('BEGIN');
  try {
    db.prepare(`
      UPDATE components
      SET name = ?, category = ?, value = ?, package_footprint = ?, specs = ?,
          qty_on_hand = ?, min_qty = ?, storage_location = ?, supplier = ?,
          supplier_sku = ?, unit_cost = ?, notes = ?, updated_at = datetime('now')
      WHERE id = ?
    `).run(
      String(name).trim(),
      String(category).trim(),
      String(value).trim(),
      String(package_footprint).trim(),
      String(specs).trim(),
      newQtyOnHand,
      Math.max(0, parseInt(min_qty, 10) || 0),
      String(storage_location).trim(),
      String(supplier).trim(),
      String(supplier_sku).trim(),
      Math.max(0, parseFloat(unit_cost) || 0),
      String(notes).trim(),
      id
    );
    clampComponentAllocations(id, newQtyOnHand);
    db.exec('COMMIT');
  } catch (err) {
    db.exec('ROLLBACK');
    return res.status(500).json({ error: err.message });
  }

  res.json(getComponentById(id));
});

app.patch('/api/components/:id/stock', (req, res) => {
  const id = Number(req.params.id);
  const existing = db.prepare('SELECT * FROM components WHERE id = ?').get(id);
  if (!existing) return res.status(404).json({ error: 'Component not found' });

  const { delta, set_qty } = req.body;
  let newQty = existing.qty_on_hand;
  if (typeof set_qty === 'number') {
    newQty = Math.max(0, Math.round(set_qty));
  } else if (typeof delta === 'number') {
    newQty = Math.max(0, existing.qty_on_hand + Math.round(delta));
  }

  db.exec('BEGIN');
  try {
    db.prepare(`UPDATE components SET qty_on_hand = ?, updated_at = datetime('now') WHERE id = ?`).run(newQty, id);
    clampComponentAllocations(id, newQty);
    db.exec('COMMIT');
  } catch (err) {
    db.exec('ROLLBACK');
    return res.status(500).json({ error: err.message });
  }
  res.json(getComponentById(id));
});

app.delete('/api/components/:id', (req, res) => {
  const id = Number(req.params.id);
  db.prepare('DELETE FROM components WHERE id = ?').run(id);
  res.json({ deleted: true, id });
});

app.post('/api/components/import-csv', (req, res) => {
  const { items } = req.body;
  if (!Array.isArray(items) || items.length === 0) {
    return res.status(400).json({ error: 'No items provided for import' });
  }

  const insertStmt = db.prepare(`
    INSERT INTO components (
      name, category, value, package_footprint, specs,
      qty_on_hand, min_qty, storage_location, supplier, supplier_sku, unit_cost, notes
    ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
  `);

  let imported = 0;
  db.exec('BEGIN');
  try {
    for (const row of items) {
      const name = String(row.name || row.part_name || row.value || '').trim();
      if (!name) continue;
      insertStmt.run(
        name,
        String(row.category || 'Other / Misc').trim(),
        String(row.value || '').trim(),
        String(row.package_footprint || row.footprint || '').trim(),
        String(row.specs || '').trim(),
        Math.max(0, parseInt(row.qty_on_hand ?? row.qty ?? 0, 10) || 0),
        Math.max(0, parseInt(row.min_qty ?? 0, 10) || 0),
        String(row.storage_location || row.location || '').trim(),
        String(row.supplier || '').trim(),
        String(row.supplier_sku || row.sku || '').trim(),
        Math.max(0, parseFloat(row.unit_cost ?? row.cost ?? 0) || 0),
        String(row.notes || '').trim()
      );
      imported++;
    }
    db.exec('COMMIT');
  } catch (err) {
    db.exec('ROLLBACK');
    return res.status(500).json({ error: err.message });
  }

  res.json({ imported, components: getAllComponents() });
});

// --- Projects / DIY Kit Builds Routes ---
app.get('/api/projects', (req, res) => {
  res.json(getAllProjects());
});

app.get('/api/projects/:id', (req, res) => {
  const detail = getProjectDetail(Number(req.params.id));
  if (!detail) return res.status(404).json({ error: 'Project not found' });
  res.json(detail);
});

app.post('/api/projects', (req, res) => {
  const {
    name,
    project_type = 'Eurorack Module',
    format_specs = '',
    status = 'Planning',
    source_vendor = '',
    doc_url = '',
    notes = ''
  } = req.body;

  if (!name || !String(name).trim()) {
    return res.status(400).json({ error: 'Project name is required' });
  }

  const safeDocUrl = sanitizeHttpUrl(doc_url);
  if (safeDocUrl === null) {
    return res.status(400).json({ error: 'Invalid documentation URL: must use http:// or https://' });
  }

  const result = db.prepare(`
    INSERT INTO projects (name, project_type, format_specs, status, source_vendor, doc_url, notes)
    VALUES (?, ?, ?, ?, ?, ?, ?)
  `).run(
    String(name).trim(),
    String(project_type).trim() || 'Eurorack Module',
    String(format_specs).trim(),
    String(status).trim() || 'Planning',
    String(source_vendor).trim(),
    safeDocUrl,
    String(notes).trim()
  );

  res.status(201).json(getProjectDetail(Number(result.lastInsertRowid)));
});

app.put('/api/projects/:id', (req, res) => {
  const id = Number(req.params.id);
  const existing = db.prepare('SELECT * FROM projects WHERE id = ?').get(id);
  if (!existing) return res.status(404).json({ error: 'Project not found' });

  const {
    name = existing.name,
    project_type = existing.project_type,
    format_specs = existing.format_specs,
    status = existing.status,
    source_vendor = existing.source_vendor,
    doc_url = existing.doc_url,
    notes = existing.notes
  } = req.body;

  if (!name || !String(name).trim()) {
    return res.status(400).json({ error: 'Project name is required' });
  }

  const safeDocUrl = sanitizeHttpUrl(doc_url);
  if (safeDocUrl === null) {
    return res.status(400).json({ error: 'Invalid documentation URL: must use http:// or https://' });
  }

  db.prepare(`
    UPDATE projects
    SET name = ?, project_type = ?, format_specs = ?, status = ?,
        source_vendor = ?, doc_url = ?, notes = ?, updated_at = datetime('now')
    WHERE id = ?
  `).run(
    String(name).trim(),
    String(project_type).trim(),
    String(format_specs).trim(),
    String(status).trim(),
    String(source_vendor).trim(),
    safeDocUrl,
    String(notes).trim(),
    id
  );

  res.json(getProjectDetail(id));
});

app.delete('/api/projects/:id', (req, res) => {
  const id = Number(req.params.id);
  db.prepare('DELETE FROM projects WHERE id = ?').run(id);
  res.json({ deleted: true, id });
});

// --- BOM & Allocation Routes ---
app.post('/api/projects/:id/bom', (req, res) => {
  const projectId = Number(req.params.id);
  const proj = db.prepare('SELECT * FROM projects WHERE id = ?').get(projectId);
  if (!proj) return res.status(404).json({ error: 'Project not found' });

  const {
    component_id = null,
    designators = '',
    part_name = '',
    category = '',
    value = '',
    package_footprint = '',
    qty_required = 1,
    allocate_immediately = false,
    notes = ''
  } = req.body;

  let resolvedCompId = component_id ? Number(component_id) : null;
  const allComponents = getAllComponents();

  if (!resolvedCompId && req.body.auto_match !== false) {
    const matched = findBestComponentMatch({ part_name, category, value, package_footprint }, allComponents);
    if (matched) resolvedCompId = matched.id;
  }

  const linkedComp = resolvedCompId ? allComponents.find((c) => c.id === resolvedCompId) : null;
  const finalPartName = String(part_name || (linkedComp ? linkedComp.name : value) || 'Component').trim();
  const finalCategory = String(category || (linkedComp ? linkedComp.category : '')).trim();
  const finalValue = String(value || (linkedComp ? linkedComp.value : '')).trim();
  const finalFootprint = String(package_footprint || (linkedComp ? linkedComp.package_footprint : '')).trim();
  const reqQty = Math.max(1, parseInt(qty_required, 10) || 1);

  let allocQty = 0;
  if (allocate_immediately && linkedComp) {
    allocQty = Math.min(reqQty, linkedComp.qty_available);
  }

  db.prepare(`
    INSERT INTO bom_items (
      project_id, component_id, designators, part_name, category, value,
      package_footprint, qty_required, qty_allocated, is_consumed, is_populated, notes
    ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, 0, 0, ?)
  `).run(
    projectId,
    resolvedCompId,
    String(designators).trim(),
    finalPartName,
    finalCategory,
    finalValue,
    finalFootprint,
    reqQty,
    allocQty,
    String(notes).trim()
  );

  db.prepare(`UPDATE projects SET updated_at = datetime('now') WHERE id = ?`).run(projectId);
  res.status(201).json(getProjectDetail(projectId));
});

app.post('/api/projects/:id/bom/import-csv', (req, res) => {
  const projectId = Number(req.params.id);
  const proj = db.prepare('SELECT * FROM projects WHERE id = ?').get(projectId);
  if (!proj) return res.status(404).json({ error: 'Project not found' });

  const { items, auto_allocate = false } = req.body;
  if (!Array.isArray(items) || items.length === 0) {
    return res.status(400).json({ error: 'No BOM lines provided' });
  }

  const allComponents = getAllComponents();
  const availTracker = new Map(allComponents.map((c) => [c.id, c.qty_available]));

  const insertStmt = db.prepare(`
    INSERT INTO bom_items (
      project_id, component_id, designators, part_name, category, value,
      package_footprint, qty_required, qty_allocated, is_consumed, is_populated, notes
    ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, 0, 0, ?)
  `);

  let imported = 0;
  let matchedCount = 0;

  db.exec('BEGIN');
  try {
    for (const row of items) {
      const value = String(row.value || '').trim();
      const partName = String(row.part_name || row.part || row.name || value || '').trim();
      if (!partName && !value) continue;

      const designators = String(row.designators || row.ref || '').trim();
      const category = String(row.category || '').trim();
      const footprint = String(row.package_footprint || row.footprint || '').trim();
      const qtyReq = Math.max(1, parseInt(row.qty_required ?? row.qty ?? 1, 10) || 1);
      const notes = String(row.notes || '').trim();

      const matched = findBestComponentMatch(
        { part_name: partName, category, value, package_footprint: footprint },
        allComponents
      );

      let compId = null;
      let allocQty = 0;
      if (matched) {
        compId = matched.id;
        matchedCount++;
        if (auto_allocate) {
          const curAvail = availTracker.get(compId) || 0;
          allocQty = Math.min(qtyReq, curAvail);
          availTracker.set(compId, Math.max(0, curAvail - allocQty));
        }
      }

      insertStmt.run(
        projectId,
        compId,
        designators,
        partName || (matched ? matched.name : value),
        category || (matched ? matched.category : ''),
        value || (matched ? matched.value : ''),
        footprint || (matched ? matched.package_footprint : ''),
        qtyReq,
        allocQty,
        notes
      );
      imported++;
    }
    db.prepare(`UPDATE projects SET updated_at = datetime('now') WHERE id = ?`).run(projectId);
    db.exec('COMMIT');
  } catch (err) {
    db.exec('ROLLBACK');
    return res.status(500).json({ error: err.message });
  }

  res.json({
    imported,
    matched_count: matchedCount,
    project: getProjectDetail(projectId)
  });
});

app.post('/api/projects/:id/bom/auto-match', (req, res) => {
  const projectId = Number(req.params.id);
  const allComponents = getAllComponents();
  const unlinked = db.prepare('SELECT * FROM bom_items WHERE project_id = ? AND component_id IS NULL').all(projectId);

  let matchedCount = 0;
  const updateStmt = db.prepare('UPDATE bom_items SET component_id = ? WHERE id = ?');

  db.exec('BEGIN');
  try {
    for (const item of unlinked) {
      const match = findBestComponentMatch(item, allComponents);
      if (match) {
        updateStmt.run(match.id, item.id);
        matchedCount++;
      }
    }
    db.prepare(`UPDATE projects SET updated_at = datetime('now') WHERE id = ?`).run(projectId);
    db.exec('COMMIT');
  } catch (err) {
    db.exec('ROLLBACK');
    return res.status(500).json({ error: err.message });
  }

  res.json({ matched_count: matchedCount, project: getProjectDetail(projectId) });
});

app.post('/api/projects/:id/allocate-all', (req, res) => {
  const projectId = Number(req.params.id);
  const allComponents = getAllComponents();
  const availMap = new Map(allComponents.map((c) => [c.id, c.qty_available]));

  const bomLines = db.prepare(`
    SELECT * FROM bom_items
    WHERE project_id = ? AND component_id IS NOT NULL AND is_consumed = 0
    ORDER BY id ASC
  `).all(projectId);

  let unitsNewlyAllocated = 0;
  const updateStmt = db.prepare('UPDATE bom_items SET qty_allocated = ? WHERE id = ?');

  db.exec('BEGIN');
  try {
    for (const line of bomLines) {
      const stillNeeded = Math.max(0, line.qty_required - line.qty_allocated);
      if (stillNeeded === 0) continue;
      const avail = availMap.get(line.component_id) || 0;
      if (avail <= 0) continue;

      const addAlloc = Math.min(stillNeeded, avail);
      updateStmt.run(line.qty_allocated + addAlloc, line.id);
      availMap.set(line.component_id, avail - addAlloc);
      unitsNewlyAllocated += addAlloc;
    }
    db.prepare(`UPDATE projects SET updated_at = datetime('now') WHERE id = ?`).run(projectId);
    db.exec('COMMIT');
  } catch (err) {
    db.exec('ROLLBACK');
    return res.status(500).json({ error: err.message });
  }

  res.json({
    units_newly_allocated: unitsNewlyAllocated,
    project: getProjectDetail(projectId)
  });
});

app.post('/api/projects/:id/release-all', (req, res) => {
  const projectId = Number(req.params.id);
  db.prepare(`
    UPDATE bom_items SET qty_allocated = 0
    WHERE project_id = ? AND is_consumed = 0
  `).run(projectId);
  db.prepare(`UPDATE projects SET updated_at = datetime('now') WHERE id = ?`).run(projectId);
  res.json({ project: getProjectDetail(projectId) });
});

app.post('/api/projects/:id/consume-allocated', (req, res) => {
  const projectId = Number(req.params.id);
  const lines = db.prepare(`
    SELECT * FROM bom_items
    WHERE project_id = ? AND is_consumed = 0 AND qty_allocated > 0 AND component_id IS NOT NULL
  `).all(projectId);

  let consumedUnits = 0;
  const deductComp = db.prepare(`
    UPDATE components
    SET qty_on_hand = MAX(0, qty_on_hand - ?), updated_at = datetime('now')
    WHERE id = ?
  `);
  const markConsumed = db.prepare(`
    UPDATE bom_items
    SET is_consumed = 1, is_populated = 1
    WHERE id = ?
  `);

  db.exec('BEGIN');
  try {
    for (const line of lines) {
      deductComp.run(line.qty_allocated, line.component_id);
      markConsumed.run(line.id);
      consumedUnits += line.qty_allocated;
    }
    if (req.body.mark_completed) {
      db.prepare(`UPDATE projects SET status = 'Completed', updated_at = datetime('now') WHERE id = ?`).run(projectId);
    } else {
      db.prepare(`UPDATE projects SET updated_at = datetime('now') WHERE id = ?`).run(projectId);
    }
    db.exec('COMMIT');
  } catch (err) {
    db.exec('ROLLBACK');
    return res.status(500).json({ error: err.message });
  }

  res.json({
    consumed_units: consumedUnits,
    project: getProjectDetail(projectId)
  });
});

app.put('/api/bom/:bomId', (req, res) => {
  const bomId = Number(req.params.bomId);
  const existing = db.prepare('SELECT * FROM bom_items WHERE id = ?').get(bomId);
  if (!existing) return res.status(404).json({ error: 'BOM item not found' });

  const {
    component_id = existing.component_id,
    designators = existing.designators,
    part_name = existing.part_name,
    category = existing.category,
    value = existing.value,
    package_footprint = existing.package_footprint,
    qty_required = existing.qty_required,
    is_populated = existing.is_populated,
    notes = existing.notes
  } = req.body;

  // If component_id changed and was not consumed, reset qty_allocated if unlinked
  const newCompId = component_id === null || component_id === '' ? null : Number(component_id);
  const reqQty = Math.max(1, parseInt(qty_required, 10) || 1);
  let newAllocated = existing.qty_allocated;
  if (newCompId !== existing.component_id && !existing.is_consumed) {
    newAllocated = 0;
  } else if (!existing.is_consumed) {
    newAllocated = Math.min(newAllocated, reqQty);
  }

  db.prepare(`
    UPDATE bom_items
    SET component_id = ?, designators = ?, part_name = ?, category = ?,
        value = ?, package_footprint = ?, qty_required = ?, qty_allocated = ?,
        is_populated = ?, notes = ?
    WHERE id = ?
  `).run(
    newCompId,
    String(designators).trim(),
    String(part_name).trim(),
    String(category).trim(),
    String(value).trim(),
    String(package_footprint).trim(),
    reqQty,
    newAllocated,
    is_populated ? 1 : 0,
    String(notes).trim(),
    bomId
  );

  db.prepare(`UPDATE projects SET updated_at = datetime('now') WHERE id = ?`).run(existing.project_id);
  res.json(getProjectDetail(existing.project_id));
});

app.post('/api/bom/:bomId/allocate', (req, res) => {
  const bomId = Number(req.params.bomId);
  const existing = db.prepare('SELECT * FROM bom_items WHERE id = ?').get(bomId);
  if (!existing) return res.status(404).json({ error: 'BOM item not found' });
  if (!existing.component_id) {
    return res.status(400).json({ error: 'Link a component from inventory before allocating parts.' });
  }
  if (existing.is_consumed) {
    return res.status(400).json({ error: 'Parts on this line have already been consumed/soldered.' });
  }

  const comp = getComponentById(existing.component_id);
  if (!comp) return res.status(404).json({ error: 'Linked component not found' });

  // Maximum we can allocate for THIS line is its existing allocation + currently unallocated available stock
  const maxPossibleForLine = existing.qty_allocated + comp.qty_available;
  let desiredQty;

  if (typeof req.body.qty_allocated === 'number') {
    desiredQty = Math.max(0, Math.min(Math.round(req.body.qty_allocated), maxPossibleForLine));
  } else {
    // Default: allocate up to qty_required
    desiredQty = Math.min(existing.qty_required, maxPossibleForLine);
  }

  db.prepare('UPDATE bom_items SET qty_allocated = ? WHERE id = ?').run(desiredQty, bomId);
  db.prepare(`UPDATE projects SET updated_at = datetime('now') WHERE id = ?`).run(existing.project_id);

  res.json(getProjectDetail(existing.project_id));
});

app.post('/api/bom/:bomId/consume', (req, res) => {
  const bomId = Number(req.params.bomId);
  const existing = db.prepare('SELECT * FROM bom_items WHERE id = ?').get(bomId);
  if (!existing) return res.status(404).json({ error: 'BOM item not found' });
  if (!existing.component_id) {
    return res.status(400).json({ error: 'Cannot consume an unlinked BOM item' });
  }

  db.exec('BEGIN');
  try {
    if (!existing.is_consumed) {
      const qtyToConsume = existing.qty_allocated > 0 ? existing.qty_allocated : existing.qty_required;
      db.prepare(`
        UPDATE components
        SET qty_on_hand = MAX(0, qty_on_hand - ?), updated_at = datetime('now')
        WHERE id = ?
      `).run(qtyToConsume, existing.component_id);
      db.prepare(`
        UPDATE bom_items
        SET qty_allocated = ?, is_consumed = 1, is_populated = 1
        WHERE id = ?
      `).run(qtyToConsume, bomId);
    } else if (req.body.unconsume) {
      db.prepare(`
        UPDATE components
        SET qty_on_hand = qty_on_hand + ?, updated_at = datetime('now')
        WHERE id = ?
      `).run(existing.qty_allocated, existing.component_id);
      db.prepare(`
        UPDATE bom_items
        SET is_consumed = 0
        WHERE id = ?
      `).run(bomId);
    }
    db.prepare(`UPDATE projects SET updated_at = datetime('now') WHERE id = ?`).run(existing.project_id);
    db.exec('COMMIT');
  } catch (err) {
    db.exec('ROLLBACK');
    return res.status(500).json({ error: err.message });
  }

  res.json(getProjectDetail(existing.project_id));
});

app.post('/api/bom/:bomId/create-component', (req, res) => {
  const bomId = Number(req.params.bomId);
  const existing = db.prepare('SELECT * FROM bom_items WHERE id = ?').get(bomId);
  if (!existing) return res.status(404).json({ error: 'BOM item not found' });

  const initialStock = Math.max(0, parseInt(req.body.qty_on_hand ?? 0, 10) || 0);
  const result = db.prepare(`
    INSERT INTO components (
      name, category, value, package_footprint, specs,
      qty_on_hand, min_qty, storage_location, supplier, supplier_sku, unit_cost, notes
    ) VALUES (?, ?, ?, ?, '', ?, 0, ?, ?, '', 0, ?)
  `).run(
    existing.part_name || existing.value || 'New Component',
    existing.category || 'Semiconductors & ICs',
    existing.value || '',
    existing.package_footprint || '',
    initialStock,
    String(req.body.storage_location || '').trim(),
    String(req.body.supplier || '').trim(),
    existing.notes || ''
  );

  const newCompId = Number(result.lastInsertRowid);
  const alloc = Math.min(existing.qty_required, initialStock);
  db.prepare('UPDATE bom_items SET component_id = ?, qty_allocated = ? WHERE id = ?').run(newCompId, alloc, bomId);
  db.prepare(`UPDATE projects SET updated_at = datetime('now') WHERE id = ?`).run(existing.project_id);

  res.json(getProjectDetail(existing.project_id));
});

app.delete('/api/bom/:bomId', (req, res) => {
  const bomId = Number(req.params.bomId);
  const existing = db.prepare('SELECT * FROM bom_items WHERE id = ?').get(bomId);
  if (!existing) return res.status(404).json({ error: 'BOM item not found' });

  db.prepare('DELETE FROM bom_items WHERE id = ?').run(bomId);
  db.prepare(`UPDATE projects SET updated_at = datetime('now') WHERE id = ?`).run(existing.project_id);
  res.json(getProjectDetail(existing.project_id));
});

// --- Build Logs Routes ---
app.post('/api/projects/:id/logs', (req, res) => {
  const projectId = Number(req.params.id);
  const { log_type = 'Progress', entry = '' } = req.body;
  if (!entry || !String(entry).trim()) {
    return res.status(400).json({ error: 'Log entry text is required' });
  }

  db.prepare(`
    INSERT INTO build_logs (project_id, log_type, entry)
    VALUES (?, ?, ?)
  `).run(projectId, String(log_type).trim(), String(entry).trim());
  db.prepare(`UPDATE projects SET updated_at = datetime('now') WHERE id = ?`).run(projectId);

  res.status(201).json(getProjectDetail(projectId));
});

app.delete('/api/logs/:logId', (req, res) => {
  const logId = Number(req.params.logId);
  const existing = db.prepare('SELECT * FROM build_logs WHERE id = ?').get(logId);
  if (!existing) return res.status(404).json({ error: 'Log not found' });

  db.prepare('DELETE FROM build_logs WHERE id = ?').run(logId);
  res.json(getProjectDetail(existing.project_id));
});

// --- Shopping List Routes ---
app.get('/api/shopping-list', (req, res) => {
  res.json(getShoppingList());
});

app.post('/api/shopping-list/receive', (req, res) => {
  const { component_id, qty_received } = req.body;
  const id = Number(component_id);
  const qty = Math.max(1, parseInt(qty_received, 10) || 1);

  const existing = db.prepare('SELECT * FROM components WHERE id = ?').get(id);
  if (!existing) return res.status(404).json({ error: 'Component not found' });

  db.prepare(`
    UPDATE components
    SET qty_on_hand = qty_on_hand + ?, updated_at = datetime('now')
    WHERE id = ?
  `).run(qty, id);

  res.json({
    component: getComponentById(id),
    shopping: getShoppingList()
  });
});

app.listen(PORT, () => {
  console.log(`PatchBay & Bench Audio DIY Inventory server running on http://localhost:${PORT}`);
});
