CREATE TABLE IF NOT EXISTS _schema_migrations (
    version INTEGER PRIMARY KEY,
    name TEXT NOT NULL,
    applied_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- 1. Hierarchical Storage Locations (Crate, Box, Binder, Drawer, Bag, Other)
CREATE TABLE IF NOT EXISTS storage_locations (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    kind TEXT NOT NULL DEFAULT 'Other',
    parent_id TEXT REFERENCES storage_locations(id) ON DELETE SET NULL,
    notes TEXT NOT NULL DEFAULT '',
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- 2. Catalog Components (9 core categories + Other / Custom, hybrid parametric columns + specs_json)
CREATE TABLE IF NOT EXISTS components (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    category TEXT NOT NULL,
    subtype TEXT NOT NULL DEFAULT '',
    package_or_mounting TEXT NOT NULL DEFAULT '',
    raw_spec_text TEXT NOT NULL,
    canonical_value_display TEXT NOT NULL DEFAULT '',
    resistance_ohms REAL,
    capacitance_pf REAL,
    inductance_uh REAL,
    voltage_rating_volts REAL,
    low_stock_threshold INTEGER NOT NULL DEFAULT 0,
    datasheet_url TEXT NOT NULL DEFAULT '',
    notes TEXT NOT NULL DEFAULT '',
    specs_json TEXT NOT NULL DEFAULT '{}',
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_components_category ON components(category);
CREATE INDEX IF NOT EXISTS idx_components_resistance ON components(resistance_ohms);
CREATE INDEX IF NOT EXISTS idx_components_capacitance ON components(capacitance_pf);
CREATE INDEX IF NOT EXISTS idx_components_inductance ON components(inductance_uh);

-- 3. Multi-Location Stock Lots
CREATE TABLE IF NOT EXISTS stock_lots (
    id TEXT PRIMARY KEY,
    component_id TEXT NOT NULL REFERENCES components(id) ON DELETE CASCADE,
    location_id TEXT REFERENCES storage_locations(id) ON DELETE SET NULL,
    quantity INTEGER NOT NULL DEFAULT 0,
    supplier TEXT NOT NULL DEFAULT '',
    supplier_part_no TEXT NOT NULL DEFAULT '',
    lot_notes TEXT NOT NULL DEFAULT '',
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_stock_lots_component ON stock_lots(component_id);
CREATE INDEX IF NOT EXISTS idx_stock_lots_location ON stock_lots(location_id);

-- 4. Kit Builds (6 lifecycle statuses + Shelved / Abandoned)
CREATE TABLE IF NOT EXISTS kit_builds (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    project_type TEXT NOT NULL DEFAULT '',
    status TEXT NOT NULL DEFAULT 'Wishlist / Planned',
    location_id TEXT REFERENCES storage_locations(id) ON DELETE SET NULL,
    description TEXT NOT NULL DEFAULT '',
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- 5. Kit Build BOM Lines (5 fulfillment statuses: Unallocated, Reserved, Picked, Installed, Included in Kit)
CREATE TABLE IF NOT EXISTS bom_lines (
    id TEXT PRIMARY KEY,
    kit_build_id TEXT NOT NULL REFERENCES kit_builds(id) ON DELETE CASCADE,
    component_id TEXT REFERENCES components(id) ON DELETE SET NULL,
    source_lot_id TEXT REFERENCES stock_lots(id) ON DELETE SET NULL,
    raw_spec_text TEXT NOT NULL,
    canonical_value_display TEXT NOT NULL DEFAULT '',
    category TEXT NOT NULL DEFAULT 'Other / Custom',
    package_or_mounting TEXT NOT NULL DEFAULT '',
    resistance_ohms REAL,
    capacitance_pf REAL,
    inductance_uh REAL,
    voltage_rating_volts REAL,
    specs_json TEXT NOT NULL DEFAULT '{}',
    quantity_required INTEGER NOT NULL DEFAULT 1,
    quantity_allocated INTEGER NOT NULL DEFAULT 0,
    fulfillment_status TEXT NOT NULL DEFAULT 'Unallocated',
    included_in_kit INTEGER NOT NULL DEFAULT 0,
    reference_designators TEXT NOT NULL DEFAULT '',
    installed_designators_json TEXT NOT NULL DEFAULT '[]',
    substitution_note TEXT NOT NULL DEFAULT '',
    notes TEXT NOT NULL DEFAULT '',
    sort_order INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_bom_lines_kit ON bom_lines(kit_build_id);
CREATE INDEX IF NOT EXISTS idx_bom_lines_component ON bom_lines(component_id);

-- 6. Timestamped Bench & Troubleshooting Log Entries
CREATE TABLE IF NOT EXISTS bench_log_entries (
    id TEXT PRIMARY KEY,
    kit_build_id TEXT NOT NULL REFERENCES kit_builds(id) ON DELETE CASCADE,
    entry_type TEXT NOT NULL DEFAULT 'Note',
    body TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- 7. Kit Build Attachments & External Links
CREATE TABLE IF NOT EXISTS attachments (
    id TEXT PRIMARY KEY,
    kit_build_id TEXT NOT NULL REFERENCES kit_builds(id) ON DELETE CASCADE,
    bench_log_entry_id TEXT REFERENCES bench_log_entries(id) ON DELETE SET NULL,
    kind TEXT NOT NULL DEFAULT 'Schematic PDF',
    original_filename TEXT NOT NULL,
    mime_type TEXT NOT NULL DEFAULT 'application/octet-stream',
    relative_path TEXT NOT NULL,
    size_bytes INTEGER NOT NULL DEFAULT 0,
    external_url TEXT NOT NULL DEFAULT '',
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- 8. CSV Import Mapping Presets, Staged Batches & Staged Rows
CREATE TABLE IF NOT EXISTS import_presets (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    target_kind TEXT NOT NULL DEFAULT 'components',
    mapping_json TEXT NOT NULL DEFAULT '{}',
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS import_batches (
    id TEXT PRIMARY KEY,
    target_kind TEXT NOT NULL,
    kit_build_id TEXT REFERENCES kit_builds(id) ON DELETE CASCADE,
    import_mode TEXT NOT NULL DEFAULT 'append',
    default_location_id TEXT REFERENCES storage_locations(id) ON DELETE SET NULL,
    default_supplier TEXT NOT NULL DEFAULT '',
    group_designators INTEGER NOT NULL DEFAULT 1,
    status TEXT NOT NULL DEFAULT 'staged',
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS import_rows (
    id TEXT PRIMARY KEY,
    batch_id TEXT NOT NULL REFERENCES import_batches(id) ON DELETE CASCADE,
    row_index INTEGER NOT NULL,
    raw_spec_text TEXT NOT NULL DEFAULT '',
    category TEXT NOT NULL DEFAULT '',
    subtype TEXT NOT NULL DEFAULT '',
    package_or_mounting TEXT NOT NULL DEFAULT '',
    quantity INTEGER NOT NULL DEFAULT 1,
    location_path TEXT NOT NULL DEFAULT '',
    supplier TEXT NOT NULL DEFAULT '',
    supplier_part_no TEXT NOT NULL DEFAULT '',
    datasheet_url TEXT NOT NULL DEFAULT '',
    reference_designators TEXT NOT NULL DEFAULT '',
    included_in_kit INTEGER NOT NULL DEFAULT 0,
    notes TEXT NOT NULL DEFAULT '',
    parsed_json TEXT NOT NULL DEFAULT '{}',
    match_status TEXT NOT NULL DEFAULT 'no_match',
    matched_component_id TEXT REFERENCES components(id) ON DELETE SET NULL,
    candidates_json TEXT NOT NULL DEFAULT '[]',
    chosen_action TEXT NOT NULL DEFAULT 'create_new'
);

-- 9. Inline Shopping List Overrides (keyed by component_id or unlinked spec group key)
CREATE TABLE IF NOT EXISTS shopping_list_overrides (
    group_key TEXT PRIMARY KEY,
    order_quantity INTEGER,
    supplier TEXT,
    supplier_part_no TEXT,
    notes TEXT,
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);
