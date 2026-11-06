use crate::db::Db;
use crate::spec_parser::{
    Category, ParsedSpec, format_capacitance_canonical, format_inductance_canonical,
    format_resistance_canonical, parse_spec,
};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value as JsonValue};
use std::collections::BTreeMap;
use std::str::FromStr;
use turso::{Value as SqlValue, params_from_iter};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LocationKind {
    Crate,
    Box,
    Binder,
    Drawer,
    Bag,
    Other,
}

impl LocationKind {
    pub const ALL: &'static [LocationKind] = &[
        LocationKind::Crate,
        LocationKind::Box,
        LocationKind::Binder,
        LocationKind::Drawer,
        LocationKind::Bag,
        LocationKind::Other,
    ];

    pub fn as_str(&self) -> &'static str {
        match self {
            LocationKind::Crate => "Crate",
            LocationKind::Box => "Box",
            LocationKind::Binder => "Binder",
            LocationKind::Drawer => "Drawer",
            LocationKind::Bag => "Bag",
            LocationKind::Other => "Other",
        }
    }

    pub fn from_label(s: &str) -> Self {
        match s.trim().to_lowercase().as_str() {
            "crate" => LocationKind::Crate,
            "box" => LocationKind::Box,
            "binder" => LocationKind::Binder,
            "drawer" => LocationKind::Drawer,
            "bag" => LocationKind::Bag,
            _ => LocationKind::Other,
        }
    }
}

impl std::fmt::Display for LocationKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for LocationKind {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(LocationKind::from_label(s))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CreateStorageLocation {
    pub name: String,
    pub kind: String,
    pub parent_id: Option<String>,
    pub notes: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StorageLocation {
    pub id: String,
    pub name: String,
    pub kind: String,
    pub parent_id: Option<String>,
    pub notes: String,
    pub breadcrumb_path: String,
    pub depth: i64,
    pub child_count: i64,
    pub lot_count: i64,
    pub kit_build_count: i64,
}

const STORAGE_LOCATIONS_CTE_SQL: &str = "
WITH RECURSIVE location_tree(id, name, kind, parent_id, notes, breadcrumb_path, depth) AS (
    SELECT
        id,
        name,
        kind,
        parent_id,
        notes,
        name AS breadcrumb_path,
        0 AS depth
    FROM storage_locations
    WHERE parent_id IS NULL
    UNION ALL
    SELECT
        c.id,
        c.name,
        c.kind,
        c.parent_id,
        c.notes,
        p.breadcrumb_path || ' > ' || c.name AS breadcrumb_path,
        p.depth + 1 AS depth
    FROM storage_locations c
    JOIN location_tree p ON c.parent_id = p.id
)
SELECT
    t.id,
    t.name,
    t.kind,
    t.parent_id,
    t.notes,
    t.breadcrumb_path,
    t.depth,
    COALESCE((SELECT COUNT(*) FROM storage_locations ch WHERE ch.parent_id = t.id), 0) AS child_count,
    COALESCE((SELECT COUNT(*) FROM stock_lots sl WHERE sl.location_id = t.id), 0) AS lot_count,
    COALESCE((SELECT COUNT(*) FROM kit_builds kb WHERE kb.location_id = t.id), 0) AS kit_build_count
FROM location_tree t
";

fn map_storage_location_row(row: &turso::Row) -> anyhow::Result<StorageLocation> {
    Ok(StorageLocation {
        id: row.get::<String>(0)?,
        name: row.get::<String>(1)?,
        kind: row.get::<String>(2)?,
        parent_id: row.get::<Option<String>>(3)?,
        notes: row.get::<String>(4)?,
        breadcrumb_path: row.get::<String>(5)?,
        depth: row.get::<i64>(6)?,
        child_count: row.get::<i64>(7)?,
        lot_count: row.get::<i64>(8)?,
        kit_build_count: row.get::<i64>(9)?,
    })
}

pub async fn create_storage_location(
    db: &Db,
    input: CreateStorageLocation,
) -> anyhow::Result<StorageLocation> {
    let name = input.name.trim();
    if name.is_empty() {
        anyhow::bail!("Storage Location name cannot be empty");
    }
    let kind = LocationKind::from_label(&input.kind).as_str().to_string();
    let parent_id = input
        .parent_id
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string);

    let conn = db.connect().await?;
    if let Some(ref pid) = parent_id {
        let mut parent_rows = conn
            .query("SELECT id FROM storage_locations WHERE id = ?1", (pid.as_str(),))
            .await?;
        if parent_rows.next().await?.is_none() {
            anyhow::bail!("Parent Storage Location '{pid}' does not exist");
        }
    }

    let id = format!("loc-{}", Uuid::new_v4());
    conn.execute(
        "INSERT INTO storage_locations (id, name, kind, parent_id, notes) VALUES (?1, ?2, ?3, ?4, ?5)",
        (
            id.as_str(),
            name,
            kind.as_str(),
            parent_id,
            input.notes.trim(),
        ),
    )
    .await?;

    get_storage_location(db, &id)
        .await?
        .ok_or_else(|| anyhow::anyhow!("Failed to load created Storage Location"))
}

pub async fn get_storage_location(
    db: &Db,
    id: &str,
) -> anyhow::Result<Option<StorageLocation>> {
    let conn = db.connect().await?;
    let sql = format!("{STORAGE_LOCATIONS_CTE_SQL} WHERE t.id = ?1");
    let mut rows = conn.query(&sql, (id,)).await?;
    if let Some(row) = rows.next().await? {
        Ok(Some(map_storage_location_row(&row)?))
    } else {
        Ok(None)
    }
}

pub async fn list_storage_locations(db: &Db) -> anyhow::Result<Vec<StorageLocation>> {
    let conn = db.connect().await?;
    let sql = format!("{STORAGE_LOCATIONS_CTE_SQL} ORDER BY t.breadcrumb_path ASC");
    let mut rows = conn.query(&sql, ()).await?;
    let mut out = Vec::new();
    while let Some(row) = rows.next().await? {
        out.push(map_storage_location_row(&row)?);
    }
    Ok(out)
}

pub async fn list_subtree_location_ids(db: &Db, root_id: &str) -> anyhow::Result<Vec<String>> {
    let conn = db.connect().await?;
    let mut rows = conn
        .query(
            "WITH RECURSIVE subtree(id) AS (
                SELECT id FROM storage_locations WHERE id = ?1
                UNION ALL
                SELECT c.id FROM storage_locations c
                JOIN subtree s ON c.parent_id = s.id
            )
            SELECT id FROM subtree",
            (root_id,),
        )
        .await?;
    let mut ids = Vec::new();
    while let Some(row) = rows.next().await? {
        ids.push(row.get::<String>(0)?);
    }
    Ok(ids)
}

pub async fn move_storage_location(
    db: &Db,
    id: &str,
    new_parent_id: Option<&str>,
) -> anyhow::Result<StorageLocation> {
    let normalized_parent = new_parent_id
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string);

    let existing = get_storage_location(db, id)
        .await?
        .ok_or_else(|| anyhow::anyhow!("Storage Location '{id}' not found"))?;

    if let Some(ref target_pid) = normalized_parent {
        if target_pid == &existing.id {
            anyhow::bail!("Cannot move a Storage Location into itself");
        }
        let target_exists = get_storage_location(db, target_pid).await?;
        if target_exists.is_none() {
            anyhow::bail!("Target parent Storage Location '{target_pid}' does not exist");
        }
        let subtree_ids = list_subtree_location_ids(db, id).await?;
        if subtree_ids.iter().any(|sub_id| sub_id == target_pid) {
            anyhow::bail!(
                "Cannot move Storage Location '{}' into its own descendant",
                existing.name
            );
        }
    }

    let conn = db.connect().await?;
    conn.execute(
        "UPDATE storage_locations SET parent_id = ?1, updated_at = datetime('now') WHERE id = ?2",
        (normalized_parent, id),
    )
    .await?;

    get_storage_location(db, id)
        .await?
        .ok_or_else(|| anyhow::anyhow!("Storage Location '{id}' not found after move"))
}

pub async fn update_storage_location(
    db: &Db,
    id: &str,
    name: &str,
    kind: &str,
    parent_id: Option<&str>,
    notes: &str,
) -> anyhow::Result<StorageLocation> {
    let trimmed_name = name.trim();
    if trimmed_name.is_empty() {
        anyhow::bail!("Storage Location name cannot be empty");
    }
    let normalized_kind = LocationKind::from_label(kind).as_str().to_string();
    move_storage_location(db, id, parent_id).await?;

    let conn = db.connect().await?;
    conn.execute(
        "UPDATE storage_locations SET name = ?1, kind = ?2, notes = ?3, updated_at = datetime('now') WHERE id = ?4",
        (trimmed_name, normalized_kind.as_str(), notes.trim(), id),
    )
    .await?;

    get_storage_location(db, id)
        .await?
        .ok_or_else(|| anyhow::anyhow!("Storage Location '{id}' not found after update"))
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StockLot {
    pub id: String,
    pub component_id: String,
    pub location_id: Option<String>,
    pub location_breadcrumb: String,
    pub quantity: i64,
    pub supplier: String,
    pub supplier_part_no: String,
    pub lot_notes: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CreateInitialStockLot {
    pub location_id: Option<String>,
    pub quantity: i64,
    pub supplier: String,
    pub supplier_part_no: String,
    pub lot_notes: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CreateStockLot {
    pub component_id: String,
    pub location_id: Option<String>,
    pub quantity: i64,
    pub supplier: String,
    pub supplier_part_no: String,
    pub lot_notes: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Component {
    pub id: String,
    pub name: String,
    pub category: String,
    pub subtype: String,
    pub package_or_mounting: String,
    pub raw_spec_text: String,
    pub canonical_value_display: String,
    pub resistance_ohms: Option<f64>,
    pub capacitance_pf: Option<f64>,
    pub inductance_uh: Option<f64>,
    pub voltage_rating_volts: Option<f64>,
    pub low_stock_threshold: i64,
    pub datasheet_url: String,
    pub notes: String,
    pub specs_json: JsonValue,
    pub total_on_hand: i64,
    pub reserved_quantity: i64,
    pub available_stock: i64,
    pub is_low_stock: bool,
    pub lots: Vec<StockLot>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CreateComponent {
    pub name: String,
    pub category: String,
    pub subtype: String,
    pub package_or_mounting: String,
    pub raw_spec_text: String,
    pub canonical_value_display: Option<String>,
    pub resistance_ohms: Option<f64>,
    pub capacitance_pf: Option<f64>,
    pub inductance_uh: Option<f64>,
    pub voltage_rating_volts: Option<f64>,
    pub low_stock_threshold: i64,
    pub datasheet_url: String,
    pub notes: String,
    pub specs_json: Option<JsonValue>,
    pub initial_lot: Option<CreateInitialStockLot>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UpdateComponent {
    pub name: String,
    pub category: String,
    pub subtype: String,
    pub package_or_mounting: String,
    pub raw_spec_text: String,
    pub canonical_value_display: Option<String>,
    pub resistance_ohms: Option<f64>,
    pub capacitance_pf: Option<f64>,
    pub inductance_uh: Option<f64>,
    pub voltage_rating_volts: Option<f64>,
    pub low_stock_threshold: i64,
    pub datasheet_url: String,
    pub notes: String,
    pub specs_json: Option<JsonValue>,
}

pub fn category_spec_fields(category: Category) -> &'static [(&'static str, &'static str)] {
    match category {
        Category::Resistors => &[
            ("tolerance", "Tolerance"),
            ("power_rating_watts", "Power (W)"),
            ("composition", "Composition"),
        ],
        Category::Capacitors => &[
            ("dielectric", "Dielectric"),
            ("tolerance", "Tolerance"),
            ("polarized", "Polarized"),
            ("lead_spacing_mm", "Lead Spacing (mm)"),
        ],
        Category::SemiconductorsDiscrete => &[
            ("part_code", "Part Code"),
            ("polarity_or_channel", "Polarity / Channel"),
            ("key_spec", "Key Spec (Vf / hFE / Vgs)"),
        ],
        Category::IntegratedCircuits => &[
            ("part_code", "Part Code"),
            ("function", "Function"),
            ("channel_or_pin_count", "Channels / Pins"),
        ],
        Category::PotentiometersAndEncoders => &[
            ("taper", "Taper"),
            ("body_size", "Body Size"),
            ("shaft_type", "Shaft Type"),
            ("gangs", "Gangs"),
            ("has_switch", "Has Switch"),
            ("center_detent", "Center Detent"),
        ],
        Category::SwitchesJacksAndConnectors => &[
            ("configuration", "Configuration"),
            ("actuation", "Actuation"),
            ("size_or_pitch", "Size / Pitch"),
        ],
        Category::InductorsTransformersAndMagnetics => &[
            ("value_or_ratio", "Value / Ratio"),
            ("current_or_power_rating", "Current / Power"),
        ],
        Category::EnclosuresKnobsAndHardware => &[
            ("size_or_form", "Size / Form"),
            ("finish_or_color", "Finish / Color"),
            ("shaft_fit", "Shaft Fit"),
        ],
        Category::ModulesAndSubAssemblies => &[
            ("model_or_platform", "Model / Platform"),
            ("interface_or_voltage", "Interface / Voltage"),
            ("form_factor_or_pinout", "Form Factor / Pinout"),
        ],
        Category::OtherCustom => &[],
    }
}

fn normalize_specs_json(parsed_specs: JsonValue, override_specs: Option<JsonValue>) -> JsonValue {
    let mut map = match parsed_specs {
        JsonValue::Object(m) => m,
        _ => Map::new(),
    };
    if let Some(JsonValue::Object(overrides)) = override_specs {
        for (k, v) in overrides {
            match &v {
                JsonValue::Null => {}
                JsonValue::String(s) if s.trim().is_empty() => {}
                JsonValue::String(s) => {
                    // If key is power_rating_watts or lead_spacing_mm and string parses as float, store as number or string cleanly
                    if (k == "power_rating_watts" || k == "lead_spacing_mm")
                        && let Ok(num) = s.trim().parse::<f64>()
                    {
                        map.insert(k, JsonValue::from(num));
                    } else {
                        map.insert(k, JsonValue::String(s.trim().to_string()));
                    }
                }
                _ => {
                    map.insert(k, v);
                }
            }
        }
    }
    JsonValue::Object(map)
}

pub async fn list_stock_lots_for_component(
    db: &Db,
    component_id: &str,
) -> anyhow::Result<Vec<StockLot>> {
    let conn = db.connect().await?;
    let mut rows = conn
        .query(
            "WITH RECURSIVE location_tree(id, breadcrumb_path) AS (
                SELECT id, name AS breadcrumb_path FROM storage_locations WHERE parent_id IS NULL
                UNION ALL
                SELECT c.id, p.breadcrumb_path || ' > ' || c.name
                FROM storage_locations c
                JOIN location_tree p ON c.parent_id = p.id
            )
            SELECT
                sl.id,
                sl.component_id,
                sl.location_id,
                COALESCE(lt.breadcrumb_path, 'Unassigned Location') AS location_breadcrumb,
                sl.quantity,
                sl.supplier,
                sl.supplier_part_no,
                sl.lot_notes
            FROM stock_lots sl
            LEFT JOIN location_tree lt ON sl.location_id = lt.id
            WHERE sl.component_id = ?1
            ORDER BY location_breadcrumb ASC, sl.created_at ASC",
            (component_id,),
        )
        .await?;
    let mut lots = Vec::new();
    while let Some(row) = rows.next().await? {
        lots.push(StockLot {
            id: row.get::<String>(0)?,
            component_id: row.get::<String>(1)?,
            location_id: row.get::<Option<String>>(2)?,
            location_breadcrumb: row.get::<String>(3)?,
            quantity: row.get::<i64>(4)?,
            supplier: row.get::<String>(5)?,
            supplier_part_no: row.get::<String>(6)?,
            lot_notes: row.get::<String>(7)?,
        });
    }
    Ok(lots)
}

pub(crate) fn merge_lot_metadata(
    existing_qty: i64,
    incoming_qty: i64,
    existing_sup: &str,
    incoming_sup: &str,
    existing_pno: &str,
    incoming_pno: &str,
    existing_notes: &str,
    incoming_notes: &str,
) -> (i64, String, String, String) {
    let new_qty = (existing_qty + incoming_qty).max(0);
    let merged_sup = if existing_sup.trim().is_empty() {
        incoming_sup.trim().to_string()
    } else {
        existing_sup.to_string()
    };
    let merged_pno = if existing_pno.trim().is_empty() {
        incoming_pno.trim().to_string()
    } else {
        existing_pno.to_string()
    };
    let merged_notes = match (existing_notes.trim(), incoming_notes.trim()) {
        ("", n) => n.to_string(),
        (e, "") => e.to_string(),
        (e, n) if e == n => e.to_string(),
        (e, n) => format!("{e}; {n}"),
    };
    (new_qty, merged_sup, merged_pno, merged_notes)
}

pub(crate) async fn upsert_or_merge_stock_lot_tx(
    tx: &turso::transaction::Transaction<'_>,
    input: CreateStockLot,
) -> anyhow::Result<String> {
    let mut comp_rows = tx
        .query(
            "SELECT id FROM components WHERE id = ?1",
            (input.component_id.as_str(),),
        )
        .await?;
    if comp_rows.next().await?.is_none() {
        anyhow::bail!("Component '{}' does not exist", input.component_id);
    }
    drop(comp_rows);

    let location_id = input
        .location_id
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string);

    if let Some(ref loc_id) = location_id {
        let mut loc_rows = tx
            .query(
                "SELECT id FROM storage_locations WHERE id = ?1",
                (loc_id.as_str(),),
            )
            .await?;
        if loc_rows.next().await?.is_none() {
            anyhow::bail!("Storage Location '{loc_id}' does not exist");
        }
    }

    // Check if a lot already exists for (component_id, location_id)
    let existing_row = if let Some(ref loc_id) = location_id {
        let mut rows = tx
            .query(
                "SELECT id, quantity, supplier, supplier_part_no, lot_notes
                 FROM stock_lots
                 WHERE component_id = ?1 AND location_id = ?2
                 ORDER BY created_at ASC LIMIT 1",
                (input.component_id.as_str(), loc_id.as_str()),
            )
            .await?;
        rows.next().await?
    } else {
        let mut rows = tx
            .query(
                "SELECT id, quantity, supplier, supplier_part_no, lot_notes
                 FROM stock_lots
                 WHERE component_id = ?1 AND location_id IS NULL
                 ORDER BY created_at ASC LIMIT 1",
                (input.component_id.as_str(),),
            )
            .await?;
        rows.next().await?
    };

    let target_lot_id = if let Some(row) = existing_row {
        let existing_id: String = row.get::<String>(0)?;
        let existing_qty: i64 = row.get::<i64>(1)?;
        let existing_sup: String = row.get::<String>(2)?;
        let existing_pno: String = row.get::<String>(3)?;
        let existing_notes: String = row.get::<String>(4)?;

        let (new_qty, merged_sup, merged_pno, merged_notes) = merge_lot_metadata(
            existing_qty,
            input.quantity,
            &existing_sup,
            &input.supplier,
            &existing_pno,
            &input.supplier_part_no,
            &existing_notes,
            &input.lot_notes,
        );

        tx.execute(
            "UPDATE stock_lots
             SET quantity = ?1, supplier = ?2, supplier_part_no = ?3, lot_notes = ?4, updated_at = datetime('now')
             WHERE id = ?5",
            (
                new_qty,
                merged_sup.as_str(),
                merged_pno.as_str(),
                merged_notes.as_str(),
                existing_id.as_str(),
            ),
        )
        .await?;
        existing_id
    } else {
        let lot_id = format!("lot-{}", Uuid::new_v4());
        tx.execute(
            "INSERT INTO stock_lots (id, component_id, location_id, quantity, supplier, supplier_part_no, lot_notes)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            (
                lot_id.as_str(),
                input.component_id.as_str(),
                location_id,
                input.quantity.max(0),
                input.supplier.trim(),
                input.supplier_part_no.trim(),
                input.lot_notes.trim(),
            ),
        )
        .await?;
        lot_id
    };

    Ok(target_lot_id)
}

pub async fn upsert_or_merge_stock_lot(
    db: &Db,
    input: CreateStockLot,
) -> anyhow::Result<StockLot> {
    let comp_id = input.component_id.clone();
    let mut conn = db.connect().await?;
    let tx = conn.transaction().await?;
    let target_lot_id = upsert_or_merge_stock_lot_tx(&tx, input).await?;
    tx.commit().await?;

    let lots = list_stock_lots_for_component(db, &comp_id).await?;
    lots.into_iter()
        .find(|l| l.id == target_lot_id)
        .ok_or_else(|| anyhow::anyhow!("Failed to load Stock Lot '{target_lot_id}'"))
}

pub async fn adjust_stock_lot_delta(
    db: &Db,
    lot_id: &str,
    delta: i64,
) -> anyhow::Result<Component> {
    let conn = db.connect().await?;
    let mut rows = conn
        .query(
            "SELECT component_id, quantity FROM stock_lots WHERE id = ?1",
            (lot_id,),
        )
        .await?;
    let row = rows
        .next()
        .await?
        .ok_or_else(|| anyhow::anyhow!("Stock Lot '{lot_id}' not found"))?;
    let component_id = row.get::<String>(0)?;
    let current_qty = row.get::<i64>(1)?;
    let next_qty = (current_qty + delta).max(0);

    conn.execute(
        "UPDATE stock_lots SET quantity = ?1, updated_at = datetime('now') WHERE id = ?2",
        (next_qty, lot_id),
    )
    .await?;

    get_component(db, &component_id)
        .await?
        .ok_or_else(|| anyhow::anyhow!("Component '{component_id}' not found after lot adjustment"))
}

pub async fn move_stock_lot_to_location(
    db: &Db,
    lot_id: &str,
    target_location_id: Option<&str>,
) -> anyhow::Result<Component> {
    let normalized_target = target_location_id
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string);

    let mut conn = db.connect().await?;
    if let Some(ref loc_id) = normalized_target {
        let mut loc_rows = conn
            .query(
                "SELECT id FROM storage_locations WHERE id = ?1",
                (loc_id.as_str(),),
            )
            .await?;
        if loc_rows.next().await?.is_none() {
            anyhow::bail!("Target Storage Location '{loc_id}' does not exist");
        }
    }

    let mut rows = conn
        .query(
            "SELECT component_id, location_id, quantity, supplier, supplier_part_no, lot_notes
             FROM stock_lots WHERE id = ?1",
            (lot_id,),
        )
        .await?;
    let row = rows
        .next()
        .await?
        .ok_or_else(|| anyhow::anyhow!("Stock Lot '{lot_id}' not found"))?;

    let component_id = row.get::<String>(0)?;
    let current_loc = row.get::<Option<String>>(1)?;
    let qty = row.get::<i64>(2)?;
    let supplier = row.get::<String>(3)?;
    let supplier_part_no = row.get::<String>(4)?;
    let lot_notes = row.get::<String>(5)?;
    drop(rows);

    if current_loc == normalized_target {
        return get_component(db, &component_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Component '{component_id}' not found"));
    }

    // Check if another lot for this component already exists at normalized_target
    let duplicate_row = if let Some(ref loc_id) = normalized_target {
        let mut dup_rows = conn
            .query(
                "SELECT id, quantity, supplier, supplier_part_no, lot_notes
                 FROM stock_lots
                 WHERE component_id = ?1 AND location_id = ?2 AND id != ?3
                 ORDER BY created_at ASC LIMIT 1",
                (component_id.as_str(), loc_id.as_str(), lot_id),
            )
            .await?;
        dup_rows.next().await?
    } else {
        let mut dup_rows = conn
            .query(
                "SELECT id, quantity, supplier, supplier_part_no, lot_notes
                 FROM stock_lots
                 WHERE component_id = ?1 AND location_id IS NULL AND id != ?2
                 ORDER BY created_at ASC LIMIT 1",
                (component_id.as_str(), lot_id),
            )
            .await?;
        dup_rows.next().await?
    };

    let tx = conn.transaction().await?;
    if let Some(dup) = duplicate_row {
        let target_id = dup.get::<String>(0)?;
        let target_qty = dup.get::<i64>(1)?;
        let target_sup = dup.get::<String>(2)?;
        let target_pno = dup.get::<String>(3)?;
        let target_notes = dup.get::<String>(4)?;

        let (merged_qty, merged_sup, merged_pno, merged_notes) = merge_lot_metadata(
            target_qty,
            qty,
            &target_sup,
            &supplier,
            &target_pno,
            &supplier_part_no,
            &target_notes,
            &lot_notes,
        );

        tx.execute(
            "UPDATE stock_lots
             SET quantity = ?1, supplier = ?2, supplier_part_no = ?3, lot_notes = ?4, updated_at = datetime('now')
             WHERE id = ?5",
            (
                merged_qty,
                merged_sup.as_str(),
                merged_pno.as_str(),
                merged_notes.as_str(),
                target_id.as_str(),
            ),
        )
        .await?;
        tx.execute(
            "UPDATE bom_lines SET source_lot_id = ?1 WHERE source_lot_id = ?2",
            (target_id.as_str(), lot_id),
        )
        .await?;
        tx.execute("DELETE FROM stock_lots WHERE id = ?1", (lot_id,))
            .await?;
    } else {
        tx.execute(
            "UPDATE stock_lots SET location_id = ?1, updated_at = datetime('now') WHERE id = ?2",
            (normalized_target, lot_id),
        )
        .await?;
    }
    tx.commit().await?;

    get_component(db, &component_id)
        .await?
        .ok_or_else(|| anyhow::anyhow!("Component '{component_id}' not found after lot move"))
}

pub async fn merge_component_stock_lots(
    db: &Db,
    component_id: &str,
    target_location_id: Option<&str>,
) -> anyhow::Result<Component> {
    let lots = list_stock_lots_for_component(db, component_id).await?;
    if lots.len() <= 1 && target_location_id.is_none() {
        return get_component(db, component_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Component '{component_id}' not found"));
    }
    if lots.is_empty() {
        return get_component(db, component_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Component '{component_id}' not found"));
    }

    let normalized_target = target_location_id
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .or_else(|| lots[0].location_id.clone());

    // Pick primary lot (prefer lot already at normalized_target, otherwise lots[0])
    let primary_idx = lots
        .iter()
        .position(|l| l.location_id == normalized_target)
        .unwrap_or(0);
    let primary_id = lots[primary_idx].id.clone();

    let mut total_qty = 0i64;
    let mut merged_supplier = String::new();
    let mut merged_part_no = String::new();
    let mut notes_list: Vec<String> = Vec::new();

    for lot in &lots {
        total_qty += lot.quantity;
        if merged_supplier.is_empty() && !lot.supplier.trim().is_empty() {
            merged_supplier = lot.supplier.trim().to_string();
        }
        if merged_part_no.is_empty() && !lot.supplier_part_no.trim().is_empty() {
            merged_part_no = lot.supplier_part_no.trim().to_string();
        }
        let n = lot.lot_notes.trim();
        if !n.is_empty() && !notes_list.iter().any(|existing| existing == n) {
            notes_list.push(n.to_string());
        }
    }
    let merged_notes = notes_list.join("; ");

    let mut conn = db.connect().await?;
    let tx = conn.transaction().await?;

    tx.execute(
        "UPDATE stock_lots
         SET location_id = ?1, quantity = ?2, supplier = ?3, supplier_part_no = ?4, lot_notes = ?5, updated_at = datetime('now')
         WHERE id = ?6",
        (
            normalized_target,
            total_qty.max(0),
            merged_supplier.as_str(),
            merged_part_no.as_str(),
            merged_notes.as_str(),
            primary_id.as_str(),
        ),
    )
    .await?;

    for lot in &lots {
        if lot.id != primary_id {
            tx.execute(
                "UPDATE bom_lines SET source_lot_id = ?1 WHERE source_lot_id = ?2",
                (primary_id.as_str(), lot.id.as_str()),
            )
            .await?;
            tx.execute("DELETE FROM stock_lots WHERE id = ?1", (lot.id.as_str(),))
                .await?;
        }
    }

    tx.commit().await?;

    get_component(db, component_id)
        .await?
        .ok_or_else(|| anyhow::anyhow!("Component '{component_id}' not found after merge"))
}

pub(crate) async fn create_component_tx(
    tx: &turso::transaction::Transaction<'_>,
    input: CreateComponent,
) -> anyhow::Result<String> {
    let raw_spec_text = input.raw_spec_text.trim();
    if raw_spec_text.is_empty() && input.name.trim().is_empty() {
        anyhow::bail!("Component raw specification or name cannot be empty");
    }

    let category_enum = Category::from_label(&input.category).unwrap_or(Category::OtherCustom);
    let parsed = parse_spec(raw_spec_text, Some(category_enum));

    let resistance_ohms = input.resistance_ohms.or(parsed.resistance_ohms);
    let capacitance_pf = input.capacitance_pf.or(parsed.capacitance_pf);
    let inductance_uh = input.inductance_uh.or(parsed.inductance_uh);
    let voltage_rating_volts = input.voltage_rating_volts.or(parsed.voltage_rating_volts);

    let canonical_value_display = input
        .canonical_value_display
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .unwrap_or_else(|| {
            if !parsed.canonical_value_display.is_empty() {
                parsed.canonical_value_display.clone()
            } else if let Some(ohms) = resistance_ohms {
                format_resistance_canonical(ohms)
            } else if let Some(pf) = capacitance_pf {
                format_capacitance_canonical(pf)
            } else if let Some(uh) = inductance_uh {
                format_inductance_canonical(uh)
            } else {
                raw_spec_text.to_string()
            }
        });

    let name = if input.name.trim().is_empty() {
        if !canonical_value_display.is_empty() {
            format!("{} {}", canonical_value_display, category_enum.as_str())
        } else {
            raw_spec_text.to_string()
        }
    } else {
        input.name.trim().to_string()
    };

    let package_or_mounting = if input.package_or_mounting.trim().is_empty() {
        parsed.package_or_mounting.unwrap_or_default()
    } else {
        input.package_or_mounting.trim().to_string()
    };

    let specs_json = normalize_specs_json(parsed.specs_json, input.specs_json);
    let specs_json_str = serde_json::to_string(&specs_json)?;

    let id = format!("comp-{}", Uuid::new_v4());
    tx.execute(
        "INSERT INTO components (
            id, name, category, subtype, package_or_mounting, raw_spec_text,
            canonical_value_display, resistance_ohms, capacitance_pf, inductance_uh,
            voltage_rating_volts, low_stock_threshold, datasheet_url, notes, specs_json
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)",
        (
            id.as_str(),
            name.as_str(),
            category_enum.as_str(),
            input.subtype.trim(),
            package_or_mounting.as_str(),
            raw_spec_text,
            canonical_value_display.as_str(),
            resistance_ohms,
            capacitance_pf,
            inductance_uh,
            voltage_rating_volts,
            input.low_stock_threshold.max(0),
            input.datasheet_url.trim(),
            input.notes.trim(),
            specs_json_str.as_str(),
        ),
    )
    .await?;

    if let Some(lot) = input.initial_lot {
        upsert_or_merge_stock_lot_tx(
            tx,
            CreateStockLot {
                component_id: id.clone(),
                location_id: lot.location_id,
                quantity: lot.quantity,
                supplier: lot.supplier,
                supplier_part_no: lot.supplier_part_no,
                lot_notes: lot.lot_notes,
            },
        )
        .await?;
    }

    Ok(id)
}

pub async fn create_component(db: &Db, input: CreateComponent) -> anyhow::Result<Component> {
    let mut conn = db.connect().await?;
    let tx = conn.transaction().await?;
    let id = create_component_tx(&tx, input).await?;
    tx.commit().await?;

    get_component(db, &id)
        .await?
        .ok_or_else(|| anyhow::anyhow!("Failed to load created Component '{id}'"))
}

pub async fn update_component(
    db: &Db,
    id: &str,
    input: UpdateComponent,
) -> anyhow::Result<Component> {
    let existing = get_component(db, id)
        .await?
        .ok_or_else(|| anyhow::anyhow!("Component '{id}' not found"))?;

    let raw_spec_text = if input.raw_spec_text.trim().is_empty() {
        existing.raw_spec_text.as_str()
    } else {
        input.raw_spec_text.trim()
    };

    let category_enum = Category::from_label(&input.category)
        .or_else(|| Category::from_label(&existing.category))
        .unwrap_or(Category::OtherCustom);
    let parsed = parse_spec(raw_spec_text, Some(category_enum));

    let resistance_ohms = input.resistance_ohms.or(parsed.resistance_ohms);
    let capacitance_pf = input.capacitance_pf.or(parsed.capacitance_pf);
    let inductance_uh = input.inductance_uh.or(parsed.inductance_uh);
    let voltage_rating_volts = input.voltage_rating_volts.or(parsed.voltage_rating_volts);

    let canonical_value_display = input
        .canonical_value_display
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .unwrap_or_else(|| {
            if !parsed.canonical_value_display.is_empty() {
                parsed.canonical_value_display.clone()
            } else {
                existing.canonical_value_display.clone()
            }
        });

    let name = if input.name.trim().is_empty() {
        existing.name.clone()
    } else {
        input.name.trim().to_string()
    };

    let package_or_mounting = if input.package_or_mounting.trim().is_empty() {
        parsed
            .package_or_mounting
            .unwrap_or(existing.package_or_mounting)
    } else {
        input.package_or_mounting.trim().to_string()
    };

    let specs_json = normalize_specs_json(parsed.specs_json, input.specs_json);
    let specs_json_str = serde_json::to_string(&specs_json)?;

    let conn = db.connect().await?;
    conn.execute(
        "UPDATE components SET
            name = ?1,
            category = ?2,
            subtype = ?3,
            package_or_mounting = ?4,
            raw_spec_text = ?5,
            canonical_value_display = ?6,
            resistance_ohms = ?7,
            capacitance_pf = ?8,
            inductance_uh = ?9,
            voltage_rating_volts = ?10,
            low_stock_threshold = ?11,
            datasheet_url = ?12,
            notes = ?13,
            specs_json = ?14,
            updated_at = datetime('now')
         WHERE id = ?15",
        (
            name.as_str(),
            category_enum.as_str(),
            input.subtype.trim(),
            package_or_mounting.as_str(),
            raw_spec_text,
            canonical_value_display.as_str(),
            resistance_ohms,
            capacitance_pf,
            inductance_uh,
            voltage_rating_volts,
            input.low_stock_threshold.max(0),
            input.datasheet_url.trim(),
            input.notes.trim(),
            specs_json_str.as_str(),
            id,
        ),
    )
    .await?;

    get_component(db, id)
        .await?
        .ok_or_else(|| anyhow::anyhow!("Component '{id}' not found after update"))
}

const SELECT_COMPONENT_BASE_SQL: &str = "
SELECT
    c.id,
    c.name,
    c.category,
    c.subtype,
    c.package_or_mounting,
    c.raw_spec_text,
    c.canonical_value_display,
    c.resistance_ohms,
    c.capacitance_pf,
    c.inductance_uh,
    c.voltage_rating_volts,
    c.low_stock_threshold,
    c.datasheet_url,
    c.notes,
    c.specs_json,
    COALESCE((SELECT SUM(sl.quantity) FROM stock_lots sl WHERE sl.component_id = c.id), 0) AS total_on_hand,
    COALESCE((SELECT SUM(bl.quantity_allocated) FROM bom_lines bl WHERE bl.component_id = c.id AND bl.fulfillment_status = 'Reserved'), 0) AS reserved_quantity
FROM components c
";

async fn map_component_row(db: &Db, row: &turso::Row) -> anyhow::Result<Component> {
    let id = row.get::<String>(0)?;
    let name = row.get::<String>(1)?;
    let category = row.get::<String>(2)?;
    let subtype = row.get::<String>(3)?;
    let package_or_mounting = row.get::<String>(4)?;
    let raw_spec_text = row.get::<String>(5)?;
    let canonical_value_display = row.get::<String>(6)?;
    let resistance_ohms = row.get::<Option<f64>>(7)?;
    let capacitance_pf = row.get::<Option<f64>>(8)?;
    let inductance_uh = row.get::<Option<f64>>(9)?;
    let voltage_rating_volts = row.get::<Option<f64>>(10)?;
    let low_stock_threshold = row.get::<i64>(11)?;
    let datasheet_url = row.get::<String>(12)?;
    let notes = row.get::<String>(13)?;
    let specs_json_str = row.get::<String>(14)?;
    let total_on_hand = row.get::<i64>(15)?;
    let reserved_quantity = row.get::<i64>(16)?;

    let available_stock = (total_on_hand - reserved_quantity).max(0);
    let is_low_stock = low_stock_threshold > 0 && available_stock <= low_stock_threshold;
    let specs_json: JsonValue = serde_json::from_str(&specs_json_str).unwrap_or_else(|_| JsonValue::Object(Map::new()));
    let lots = list_stock_lots_for_component(db, &id).await?;

    Ok(Component {
        id,
        name,
        category,
        subtype,
        package_or_mounting,
        raw_spec_text,
        canonical_value_display,
        resistance_ohms,
        capacitance_pf,
        inductance_uh,
        voltage_rating_volts,
        low_stock_threshold,
        datasheet_url,
        notes,
        specs_json,
        total_on_hand,
        reserved_quantity,
        available_stock,
        is_low_stock,
        lots,
    })
}

pub async fn get_component(db: &Db, id: &str) -> anyhow::Result<Option<Component>> {
    let conn = db.connect().await?;
    let sql = format!("{SELECT_COMPONENT_BASE_SQL} WHERE c.id = ?1");
    let mut rows = conn.query(&sql, (id,)).await?;
    if let Some(row) = rows.next().await? {
        Ok(Some(map_component_row(db, &row).await?))
    } else {
        Ok(None)
    }
}

pub async fn list_components(db: &Db) -> anyhow::Result<Vec<Component>> {
    let conn = db.connect().await?;
    let sql = format!("{SELECT_COMPONENT_BASE_SQL} ORDER BY c.category ASC, COALESCE(c.resistance_ohms, c.capacitance_pf, c.inductance_uh, 0) ASC, c.name ASC");
    let mut rows = conn.query(&sql, ()).await?;
    let mut out = Vec::new();
    while let Some(row) = rows.next().await? {
        out.push(map_component_row(db, &row).await?);
    }
    Ok(out)
}

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct ComponentQueryFilter {
    pub q: Option<String>,
    pub category: Option<String>,
    pub location_id: Option<String>,
    pub package_or_mounting: Option<String>,
    pub voltage_rating_volts: Option<f64>,
    pub spec_filters: BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ComponentQueryResult {
    pub components: Vec<Component>,
    pub parsed_query: Option<ParsedSpec>,
}

fn is_valid_spec_json_key(key: &str) -> bool {
    !key.is_empty()
        && key
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_')
}

pub async fn query_components(
    db: &Db,
    filter: &ComponentQueryFilter,
) -> anyhow::Result<ComponentQueryResult> {
    let mut clauses: Vec<String> = Vec::new();
    let mut bind_values: Vec<SqlValue> = Vec::new();

    let cat_hint = filter
        .category
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty() && *s != "All")
        .and_then(Category::from_label);

    if let Some(cat) = cat_hint {
        clauses.push("c.category = ?".to_string());
        bind_values.push(SqlValue::from(cat.as_str()));
    }

    if let Some(ref pkg) = filter.package_or_mounting {
        let p = pkg.trim();
        if !p.is_empty() {
            clauses.push("LOWER(c.package_or_mounting) LIKE LOWER(?)".to_string());
            bind_values.push(SqlValue::from(format!("%{p}%")));
        }
    }

    if let Some(volts) = filter.voltage_rating_volts {
        clauses.push("ABS(c.voltage_rating_volts - ?) < 1e-4".to_string());
        bind_values.push(SqlValue::from(volts));
    }

    if let Some(ref loc_id) = filter.location_id {
        let lid = loc_id.trim();
        if !lid.is_empty() && lid != "All" {
            let subtree_ids = list_subtree_location_ids(db, lid).await?;
            if subtree_ids.is_empty() {
                return Ok(ComponentQueryResult {
                    components: Vec::new(),
                    parsed_query: None,
                });
            }
            let placeholders = std::iter::repeat_n("?", subtree_ids.len())
                .collect::<Vec<_>>()
                .join(", ");
            clauses.push(format!(
                "EXISTS (SELECT 1 FROM stock_lots sl WHERE sl.component_id = c.id AND sl.location_id IN ({placeholders}))"
            ));
            for sub_id in subtree_ids {
                bind_values.push(SqlValue::from(sub_id));
            }
        }
    }

    for (key, val) in &filter.spec_filters {
        let k = key.trim();
        let v = val.trim();
        if !is_valid_spec_json_key(k) || v.is_empty() {
            continue;
        }
        if let Ok(num_val) = v.parse::<f64>() {
            clauses.push(format!(
                "(LOWER(CAST(json_extract(c.specs_json, '$.{k}') AS TEXT)) = LOWER(?) OR ABS(CAST(json_extract(c.specs_json, '$.{k}') AS REAL) - ?) < 1e-5)"
            ));
            bind_values.push(SqlValue::from(v.to_string()));
            bind_values.push(SqlValue::from(num_val));
        } else {
            clauses.push(format!(
                "LOWER(CAST(json_extract(c.specs_json, '$.{k}') AS TEXT)) LIKE LOWER(?)"
            ));
            bind_values.push(SqlValue::from(format!("%{v}%")));
        }
    }

    let q_str = filter.q.as_deref().unwrap_or("").trim();
    let parsed_query = if !q_str.is_empty() {
        let parsed = parse_spec(q_str, cat_hint);

        let has_numeric_electrical = parsed.resistance_ohms.is_some()
            || parsed.capacitance_pf.is_some()
            || parsed.inductance_uh.is_some();

        if has_numeric_electrical {
            if let Some(ohms) = parsed.resistance_ohms {
                clauses.push("ABS(c.resistance_ohms - ?) < 1e-3".to_string());
                bind_values.push(SqlValue::from(ohms));
            }
            if let Some(pf) = parsed.capacitance_pf {
                clauses.push("ABS(c.capacitance_pf - ?) < 1e-3".to_string());
                bind_values.push(SqlValue::from(pf));
            }
            if let Some(uh) = parsed.inductance_uh {
                clauses.push("ABS(c.inductance_uh - ?) < 1e-3".to_string());
                bind_values.push(SqlValue::from(uh));
            }
            if let Some(ref taper) = parsed.taper {
                clauses.push(
                    "UPPER(CAST(json_extract(c.specs_json, '$.taper') AS TEXT)) = UPPER(?)"
                        .to_string(),
                );
                bind_values.push(SqlValue::from(taper.clone()));
            }
            if let Some(ref tol) = parsed.tolerance {
                clauses.push(
                    "LOWER(CAST(json_extract(c.specs_json, '$.tolerance') AS TEXT)) = LOWER(?)"
                        .to_string(),
                );
                bind_values.push(SqlValue::from(tol.clone()));
            }
            if let Some(w) = parsed.power_rating_watts {
                clauses.push(
                    "ABS(CAST(json_extract(c.specs_json, '$.power_rating_watts') AS REAL) - ?) < 1e-4"
                        .to_string(),
                );
                bind_values.push(SqlValue::from(w));
            }
            if let Some(v) = parsed.voltage_rating_volts {
                clauses.push("ABS(c.voltage_rating_volts - ?) < 1e-3".to_string());
                bind_values.push(SqlValue::from(v));
            }
            if let Some(ref pkg) = parsed.package_or_mounting {
                clauses.push("LOWER(c.package_or_mounting) LIKE LOWER(?)".to_string());
                bind_values.push(SqlValue::from(format!("%{pkg}%")));
            }
        } else {
            let pattern = format!("%{q_str}%");
            clauses.push(
                "(LOWER(c.name) LIKE LOWER(?)
                  OR LOWER(c.raw_spec_text) LIKE LOWER(?)
                  OR LOWER(c.canonical_value_display) LIKE LOWER(?)
                  OR LOWER(c.subtype) LIKE LOWER(?)
                  OR LOWER(c.package_or_mounting) LIKE LOWER(?)
                  OR LOWER(c.notes) LIKE LOWER(?)
                  OR LOWER(c.specs_json) LIKE LOWER(?))"
                    .to_string(),
            );
            for _ in 0..7 {
                bind_values.push(SqlValue::from(pattern.clone()));
            }
        }

        Some(parsed)
    } else {
        None
    };

    let where_sql = if clauses.is_empty() {
        String::new()
    } else {
        format!(" WHERE {}", clauses.join(" AND "))
    };

    let sql = format!(
        "{SELECT_COMPONENT_BASE_SQL}{where_sql} ORDER BY c.category ASC, COALESCE(c.resistance_ohms, c.capacitance_pf, c.inductance_uh, 0) ASC, c.name ASC"
    );

    let conn = db.connect().await?;
    let mut rows = conn.query(&sql, params_from_iter(bind_values)).await?;
    let mut components = Vec::new();
    while let Some(row) = rows.next().await? {
        components.push(map_component_row(db, &row).await?);
    }

    Ok(ComponentQueryResult {
        components,
        parsed_query,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::init_memory_db;

    #[tokio::test]
    async fn creates_nested_storage_locations_updates_breadcrumbs_on_move_and_rejects_cycles() {
        let db = init_memory_db().await.expect("memory db");

        let crate_1 = create_storage_location(
            &db,
            CreateStorageLocation {
                name: "Crate 1".into(),
                kind: "Crate".into(),
                parent_id: None,
                notes: "Top shelf crate".into(),
            },
        )
        .await
        .expect("create crate 1");
        assert_eq!(crate_1.breadcrumb_path, "Crate 1");
        assert_eq!(crate_1.depth, 0);

        let crate_2 = create_storage_location(
            &db,
            CreateStorageLocation {
                name: "Crate 2".into(),
                kind: "Crate".into(),
                parent_id: None,
                notes: "Bottom shelf crate".into(),
            },
        )
        .await
        .expect("create crate 2");

        let passives_bag = create_storage_location(
            &db,
            CreateStorageLocation {
                name: "Passives Bag".into(),
                kind: "Bag".into(),
                parent_id: Some(crate_1.id.clone()),
                notes: "Large zip bag".into(),
            },
        )
        .await
        .expect("create passives bag");
        assert_eq!(passives_bag.breadcrumb_path, "Crate 1 > Passives Bag");
        assert_eq!(passives_bag.depth, 1);

        let film_caps_bag = create_storage_location(
            &db,
            CreateStorageLocation {
                name: "Film Caps Sub-bag".into(),
                kind: "Bag".into(),
                parent_id: Some(passives_bag.id.clone()),
                notes: "Box film caps".into(),
            },
        )
        .await
        .expect("create film caps sub-bag");
        assert_eq!(
            film_caps_bag.breadcrumb_path,
            "Crate 1 > Passives Bag > Film Caps Sub-bag"
        );
        assert_eq!(film_caps_bag.depth, 2);

        // Verify subtree query on Crate 1 returns Crate 1, Passives Bag, Film Caps Sub-bag
        let crate_1_subtree = list_subtree_location_ids(&db, &crate_1.id)
            .await
            .expect("subtree");
        assert_eq!(crate_1_subtree.len(), 3);
        assert!(crate_1_subtree.contains(&crate_1.id));
        assert!(crate_1_subtree.contains(&passives_bag.id));
        assert!(crate_1_subtree.contains(&film_caps_bag.id));

        // Move Passives Bag from Crate 1 to Crate 2
        let moved_bag = move_storage_location(&db, &passives_bag.id, Some(&crate_2.id))
            .await
            .expect("move passives bag to crate 2");
        assert_eq!(moved_bag.breadcrumb_path, "Crate 2 > Passives Bag");

        // Verify nested child Film Caps Sub-bag automatically updated its breadcrumb path!
        let updated_leaf = get_storage_location(&db, &film_caps_bag.id)
            .await
            .expect("get leaf")
            .expect("leaf exists");
        assert_eq!(
            updated_leaf.breadcrumb_path,
            "Crate 2 > Passives Bag > Film Caps Sub-bag"
        );

        // Verify list_storage_locations returns all 4 locations with updated paths
        let all_locations = list_storage_locations(&db).await.expect("list all");
        assert_eq!(all_locations.len(), 4);
        assert!(
            all_locations
                .iter()
                .any(|l| l.breadcrumb_path == "Crate 2 > Passives Bag > Film Caps Sub-bag")
        );

        // Verify circular move is rejected: cannot move Crate 2 into its own descendant Film Caps Sub-bag, nor into itself
        let self_move = move_storage_location(&db, &crate_2.id, Some(&crate_2.id)).await;
        assert!(self_move.is_err(), "moving location into itself must fail");

        let cycle_move = move_storage_location(&db, &crate_2.id, Some(&film_caps_bag.id)).await;
        assert!(
            cycle_move.is_err(),
            "moving location into its descendant must fail"
        );
    }

    #[tokio::test]
    async fn creates_and_updates_components_across_categories_with_parsed_specs_and_initial_stock_lot() {
        let db = init_memory_db().await.expect("memory db");

        let binder = create_storage_location(
            &db,
            CreateStorageLocation {
                name: "Binder A".into(),
                kind: "Binder".into(),
                parent_id: None,
                notes: "".into(),
            },
        )
        .await
        .unwrap();
        let page_2 = create_storage_location(
            &db,
            CreateStorageLocation {
                name: "Page 2 (1k-10k)".into(),
                kind: "Bag".into(),
                parent_id: Some(binder.id.clone()),
                notes: "".into(),
            },
        )
        .await
        .unwrap();

        // Create a Resistor with shorthand raw_spec_text and initial stock lot
        let mut custom_specs = serde_json::Map::new();
        custom_specs.insert(
            "composition".into(),
            serde_json::Value::String("Metal Film".into()),
        );

        let resistor = create_component(
            &db,
            CreateComponent {
                name: "4.7kΩ 1% 1/4W Metal Film Resistor".into(),
                category: "Resistors".into(),
                subtype: "Fixed".into(),
                package_or_mounting: "Through-Hole".into(),
                raw_spec_text: "4k7 1% 1/4W Metal Film".into(),
                canonical_value_display: None,
                resistance_ohms: None,
                capacitance_pf: None,
                inductance_uh: None,
                voltage_rating_volts: None,
                low_stock_threshold: 20,
                datasheet_url: "https://example.com/resistor.pdf".into(),
                notes: "Royal Ohm 1%".into(),
                specs_json: Some(serde_json::Value::Object(custom_specs)),
                initial_lot: Some(CreateInitialStockLot {
                    location_id: Some(page_2.id.clone()),
                    quantity: 42,
                    supplier: "Tayda".into(),
                    supplier_part_no: "A-2214".into(),
                    lot_notes: "Blue body".into(),
                }),
            },
        )
        .await
        .expect("create resistor");

        assert_eq!(resistor.category, "Resistors");
        assert_eq!(resistor.raw_spec_text, "4k7 1% 1/4W Metal Film");
        assert_eq!(resistor.canonical_value_display, "4.7kΩ");
        assert_eq!(resistor.resistance_ohms, Some(4700.0));
        assert_eq!(resistor.specs_json["tolerance"], "1%");
        assert_eq!(resistor.specs_json["power_rating_watts"], 0.25);
        assert_eq!(resistor.specs_json["composition"], "Metal Film");
        assert_eq!(resistor.total_on_hand, 42);
        assert_eq!(resistor.available_stock, 42);
        assert!(!resistor.is_low_stock);
        assert_eq!(resistor.lots.len(), 1);
        assert_eq!(
            resistor.lots[0].location_breadcrumb,
            "Binder A > Page 2 (1k-10k)"
        );
        assert_eq!(resistor.lots[0].supplier, "Tayda");

        // Create components across the other built-in categories to verify all 9 + Other / Custom work
        let categories_and_specs = [
            ("Capacitors", "Film", "0.047uF 63V 5% Box Film 5mm", "47nF"),
            ("Semiconductors — Discrete", "Diodes", "1N34A Germanium Diode", "1N34A"),
            ("Integrated Circuits (ICs)", "Op-Amp", "TL072IP Dual JFET Op-Amp DIP-8", "TL072IP"),
            ("Potentiometers & Encoders", "Rotary Pots", "100kB Dual-Gang 16mm PCB", "100kΩ"),
            ("Switches, Jacks & Connectors", "Footswitch", "3PDT Latching Footswitch Solder Lug", "3PDT Latching Footswitch Solder Lug"),
            ("Inductors, Transformers & Magnetics", "Wah Inductor", "500mH Halo Wah Inductor", "500mH"),
            ("Enclosures, Knobs & Mechanical Hardware", "Enclosures", "125B Diecast Aluminum Enclosure", "125B Diecast Aluminum Enclosure"),
            ("Modules & Sub-Assemblies", "Reverb Brick", "BTDR-2H Belton Digital Reverb Brick", "BTDR-2H"),
            ("Other / Custom", "Misc", "Silicone Hookup Wire 24AWG", "Silicone Hookup Wire 24AWG"),
        ];

        for (cat, sub, raw, expected_canonical) in categories_and_specs {
            let comp = create_component(
                &db,
                CreateComponent {
                    name: format!("{cat} test part"),
                    category: cat.into(),
                    subtype: sub.into(),
                    package_or_mounting: "".into(),
                    raw_spec_text: raw.into(),
                    canonical_value_display: None,
                    resistance_ohms: None,
                    capacitance_pf: None,
                    inductance_uh: None,
                    voltage_rating_volts: None,
                    low_stock_threshold: 5,
                    datasheet_url: "".into(),
                    notes: "".into(),
                    specs_json: None,
                    initial_lot: None,
                },
            )
            .await
            .unwrap_or_else(|e| panic!("failed to create {cat}: {e}"));
            assert_eq!(comp.category, cat);
            assert_eq!(comp.raw_spec_text, raw);
            assert_eq!(comp.canonical_value_display, expected_canonical);
        }

        // Update the resistor's notes and threshold
        let updated = update_component(
            &db,
            &resistor.id,
            UpdateComponent {
                name: "4.7kΩ 1% 1/4W Metal Film (Updated)".into(),
                category: "Resistors".into(),
                subtype: "Fixed".into(),
                package_or_mounting: "Through-Hole".into(),
                raw_spec_text: "4k7 1% 1/4W Metal Film".into(),
                canonical_value_display: Some("4.7kΩ".into()),
                resistance_ohms: Some(4700.0),
                capacitance_pf: None,
                inductance_uh: None,
                voltage_rating_volts: None,
                low_stock_threshold: 50,
                datasheet_url: "https://example.com/resistor.pdf".into(),
                notes: "Updated notes".into(),
                specs_json: Some(resistor.specs_json.clone()),
            },
        )
        .await
        .expect("update resistor");

        assert_eq!(updated.name, "4.7kΩ 1% 1/4W Metal Film (Updated)");
        assert_eq!(updated.notes, "Updated notes");
        assert_eq!(updated.low_stock_threshold, 50);
        assert!(
            updated.is_low_stock,
            "42 available <= 50 threshold should mark low stock"
        );
    }

    #[tokio::test]
    async fn adjusts_stock_lot_deltas_computes_available_stock_and_merges_duplicate_lots_automatically_and_manually() {
        let db = init_memory_db().await.expect("memory db");

        let binder = create_storage_location(
            &db,
            CreateStorageLocation {
                name: "Binder A".into(),
                kind: "Binder".into(),
                parent_id: None,
                notes: "".into(),
            },
        )
        .await
        .unwrap();
        let crate_b = create_storage_location(
            &db,
            CreateStorageLocation {
                name: "Crate B".into(),
                kind: "Crate".into(),
                parent_id: None,
                notes: "".into(),
            },
        )
        .await
        .unwrap();
        let crate_c = create_storage_location(
            &db,
            CreateStorageLocation {
                name: "Crate C".into(),
                kind: "Crate".into(),
                parent_id: None,
                notes: "".into(),
            },
        )
        .await
        .unwrap();

        let resistor = create_component(
            &db,
            CreateComponent {
                name: "4.7kΩ 1% Metal Film".into(),
                category: "Resistors".into(),
                subtype: "Fixed".into(),
                package_or_mounting: "Through-Hole".into(),
                raw_spec_text: "4k7 1% 1/4W Metal Film".into(),
                canonical_value_display: None,
                resistance_ohms: None,
                capacitance_pf: None,
                inductance_uh: None,
                voltage_rating_volts: None,
                low_stock_threshold: 20,
                datasheet_url: "".into(),
                notes: "".into(),
                specs_json: None,
                initial_lot: Some(CreateInitialStockLot {
                    location_id: Some(binder.id.clone()),
                    quantity: 42,
                    supplier: "Tayda".into(),
                    supplier_part_no: "A-2214".into(),
                    lot_notes: "Primary sleeve".into(),
                }),
            },
        )
        .await
        .unwrap();

        let lot_a_id = resistor.lots[0].id.clone();

        // 1. Adjust single lot delta: -2 -> 40, +5 -> 45, -100 -> floored at 0, +42 -> 42
        let after_minus_2 = adjust_stock_lot_delta(&db, &lot_a_id, -2).await.unwrap();
        assert_eq!(after_minus_2.total_on_hand, 40);
        let after_plus_5 = adjust_stock_lot_delta(&db, &lot_a_id, 5).await.unwrap();
        assert_eq!(after_plus_5.total_on_hand, 45);
        let after_floor = adjust_stock_lot_delta(&db, &lot_a_id, -100).await.unwrap();
        assert_eq!(after_floor.total_on_hand, 0);
        let restored = adjust_stock_lot_delta(&db, &lot_a_id, 42).await.unwrap();
        assert_eq!(restored.total_on_hand, 42);

        // 2. Add a second lot in Crate B (15 pcs)
        upsert_or_merge_stock_lot(
            &db,
            CreateStockLot {
                component_id: resistor.id.clone(),
                location_id: Some(crate_b.id.clone()),
                quantity: 15,
                supplier: "Mouser".into(),
                supplier_part_no: "271-4.7K-RC".into(),
                lot_notes: "Surplus bag".into(),
            },
        )
        .await
        .unwrap();

        let with_two_lots = get_component(&db, &resistor.id).await.unwrap().unwrap();
        assert_eq!(with_two_lots.lots.len(), 2);
        assert_eq!(with_two_lots.total_on_hand, 57);

        // 3. Automatic lot merge on upsert: adding 8 pcs into Crate B again merges into existing Crate B lot (15 + 8 = 23)
        upsert_or_merge_stock_lot(
            &db,
            CreateStockLot {
                component_id: resistor.id.clone(),
                location_id: Some(crate_b.id.clone()),
                quantity: 8,
                supplier: "Mouser".into(),
                supplier_part_no: "271-4.7K-RC".into(),
                lot_notes: "Restock".into(),
            },
        )
        .await
        .unwrap();

        let after_auto_upsert = get_component(&db, &resistor.id).await.unwrap().unwrap();
        assert_eq!(after_auto_upsert.lots.len(), 2);
        assert_eq!(after_auto_upsert.total_on_hand, 65);
        let lot_b = after_auto_upsert
            .lots
            .iter()
            .find(|l| l.location_id.as_deref() == Some(crate_b.id.as_str()))
            .unwrap();
        assert_eq!(lot_b.quantity, 23);
        assert!(lot_b.lot_notes.contains("Surplus bag"));
        assert!(lot_b.lot_notes.contains("Restock"));

        // 4. Add a third lot in Crate C (10 pcs) and move it into Crate B -> auto-merges into Crate B (23 + 10 = 33)
        let lot_c = upsert_or_merge_stock_lot(
            &db,
            CreateStockLot {
                component_id: resistor.id.clone(),
                location_id: Some(crate_c.id.clone()),
                quantity: 10,
                supplier: "DigiKey".into(),
                supplier_part_no: "4.7KQBK-ND".into(),
                lot_notes: "Loose strip".into(),
            },
        )
        .await
        .unwrap();

        let after_move_merge = move_stock_lot_to_location(&db, &lot_c.id, Some(&crate_b.id))
            .await
            .unwrap();
        assert_eq!(after_move_merge.lots.len(), 2);
        assert_eq!(after_move_merge.total_on_hand, 75);

        // 5. Verify Available Stock accounts for Reserved BOM lines
        let conn = db.connect().await.unwrap();
        conn.execute(
            "INSERT INTO kit_builds (id, name, status) VALUES ('kit-1', 'Test Pedal', 'Sourcing / Waiting on Parts')",
            (),
        )
        .await
        .unwrap();
        conn.execute(
            "INSERT INTO bom_lines (id, kit_build_id, component_id, raw_spec_text, quantity_required, quantity_allocated, fulfillment_status)
             VALUES ('bom-1', 'kit-1', ?1, '4k7', 12, 12, 'Reserved')",
            (resistor.id.as_str(),),
        )
        .await
        .unwrap();

        let with_reserved = get_component(&db, &resistor.id).await.unwrap().unwrap();
        assert_eq!(with_reserved.total_on_hand, 75);
        assert_eq!(with_reserved.reserved_quantity, 12);
        assert_eq!(with_reserved.available_stock, 63);

        // 6. Manual 1-click "Merge Duplicate Stock Lots" consolidates all lots into Binder A
        let merged = merge_component_stock_lots(&db, &resistor.id, Some(&binder.id))
            .await
            .unwrap();
        assert_eq!(merged.lots.len(), 1);
        assert_eq!(merged.lots[0].location_id.as_deref(), Some(binder.id.as_str()));
        assert_eq!(merged.lots[0].quantity, 75);
        assert_eq!(merged.total_on_hand, 75);
        assert_eq!(merged.available_stock, 63);
    }

    #[tokio::test]
    async fn queries_components_by_shorthand_value_json_specs_and_recursive_storage_location_subtree() {
        let db = init_memory_db().await.expect("memory db");

        // Hierarchy: Crate 1 -> Passives Bag -> Film Caps Sub-bag; Crate 2 -> Pots Bag
        let crate_1 = create_storage_location(
            &db,
            CreateStorageLocation {
                name: "Crate 1".into(),
                kind: "Crate".into(),
                parent_id: None,
                notes: "".into(),
            },
        )
        .await
        .unwrap();
        let passives_bag = create_storage_location(
            &db,
            CreateStorageLocation {
                name: "Passives Bag".into(),
                kind: "Bag".into(),
                parent_id: Some(crate_1.id.clone()),
                notes: "".into(),
            },
        )
        .await
        .unwrap();
        let film_caps_bag = create_storage_location(
            &db,
            CreateStorageLocation {
                name: "Film Caps Sub-bag".into(),
                kind: "Bag".into(),
                parent_id: Some(passives_bag.id.clone()),
                notes: "".into(),
            },
        )
        .await
        .unwrap();
        let crate_2 = create_storage_location(
            &db,
            CreateStorageLocation {
                name: "Crate 2".into(),
                kind: "Crate".into(),
                parent_id: None,
                notes: "".into(),
            },
        )
        .await
        .unwrap();

        // 1) 4k7 1% Metal Film in Passives Bag
        let r_4k7_1pct = create_component(
            &db,
            CreateComponent {
                name: "4.7kΩ 1% Metal Film".into(),
                category: "Resistors".into(),
                subtype: "Fixed".into(),
                package_or_mounting: "Through-Hole".into(),
                raw_spec_text: "4k7 1% 1/4W Metal Film".into(),
                canonical_value_display: None,
                resistance_ohms: None,
                capacitance_pf: None,
                inductance_uh: None,
                voltage_rating_volts: None,
                low_stock_threshold: 10,
                datasheet_url: "".into(),
                notes: "".into(),
                specs_json: None,
                initial_lot: Some(CreateInitialStockLot {
                    location_id: Some(passives_bag.id.clone()),
                    quantity: 50,
                    supplier: "Tayda".into(),
                    supplier_part_no: "A-2214".into(),
                    lot_notes: "".into(),
                }),
            },
        )
        .await
        .unwrap();

        // 2) 4.7k 5% Carbon Film in Crate 2
        let r_4k7_5pct = create_component(
            &db,
            CreateComponent {
                name: "4.7kΩ 5% Carbon Film".into(),
                category: "Resistors".into(),
                subtype: "Fixed".into(),
                package_or_mounting: "Through-Hole".into(),
                raw_spec_text: "4.7k 5% 1/4W Carbon Film".into(),
                canonical_value_display: None,
                resistance_ohms: None,
                capacitance_pf: None,
                inductance_uh: None,
                voltage_rating_volts: None,
                low_stock_threshold: 10,
                datasheet_url: "".into(),
                notes: "".into(),
                specs_json: None,
                initial_lot: Some(CreateInitialStockLot {
                    location_id: Some(crate_2.id.clone()),
                    quantity: 20,
                    supplier: "Mouser".into(),
                    supplier_part_no: "CF-4K7".into(),
                    lot_notes: "".into(),
                }),
            },
        )
        .await
        .unwrap();

        // 3) 47nF Film Cap in Film Caps Sub-bag (nested inside Passives Bag inside Crate 1)
        let c_47nf = create_component(
            &db,
            CreateComponent {
                name: "47nF 63V WIMA Box Film".into(),
                category: "Capacitors".into(),
                subtype: "Film".into(),
                package_or_mounting: "Through-Hole 5mm".into(),
                raw_spec_text: "0.047uF 63V 5% Box Film 5mm".into(),
                canonical_value_display: None,
                resistance_ohms: None,
                capacitance_pf: None,
                inductance_uh: None,
                voltage_rating_volts: None,
                low_stock_threshold: 10,
                datasheet_url: "".into(),
                notes: "".into(),
                specs_json: None,
                initial_lot: Some(CreateInitialStockLot {
                    location_id: Some(film_caps_bag.id.clone()),
                    quantity: 12,
                    supplier: "Mouser".into(),
                    supplier_part_no: "MKS2-47N".into(),
                    lot_notes: "".into(),
                }),
            },
        )
        .await
        .unwrap();

        // 4) 100kA Potentiometer in Crate 2
        let pot_100ka = create_component(
            &db,
            CreateComponent {
                name: "100kΩ Audio Taper Pot".into(),
                category: "Potentiometers & Encoders".into(),
                subtype: "Rotary Pots".into(),
                package_or_mounting: "16mm PCB".into(),
                raw_spec_text: "100kA 16mm PCB".into(),
                canonical_value_display: None,
                resistance_ohms: None,
                capacitance_pf: None,
                inductance_uh: None,
                voltage_rating_volts: None,
                low_stock_threshold: 2,
                datasheet_url: "".into(),
                notes: "".into(),
                specs_json: None,
                initial_lot: Some(CreateInitialStockLot {
                    location_id: Some(crate_2.id.clone()),
                    quantity: 4,
                    supplier: "Tayda".into(),
                    supplier_part_no: "P-100KA".into(),
                    lot_notes: "".into(),
                }),
            },
        )
        .await
        .unwrap();

        // 5) 100kB Potentiometer in Crate 2
        let pot_100kb = create_component(
            &db,
            CreateComponent {
                name: "100kΩ Linear Taper Pot".into(),
                category: "Potentiometers & Encoders".into(),
                subtype: "Rotary Pots".into(),
                package_or_mounting: "16mm PCB".into(),
                raw_spec_text: "100kB 16mm PCB".into(),
                canonical_value_display: None,
                resistance_ohms: None,
                capacitance_pf: None,
                inductance_uh: None,
                voltage_rating_volts: None,
                low_stock_threshold: 2,
                datasheet_url: "".into(),
                notes: "".into(),
                specs_json: None,
                initial_lot: Some(CreateInitialStockLot {
                    location_id: Some(crate_2.id.clone()),
                    quantity: 6,
                    supplier: "Tayda".into(),
                    supplier_part_no: "P-100KB".into(),
                    lot_notes: "".into(),
                }),
            },
        )
        .await
        .unwrap();

        // Shorthand resistance query: "4700R" matches both 4.7k resistors and returns parsed_query
        let res_4700r = query_components(
            &db,
            &ComponentQueryFilter {
                q: Some("4700R".into()),
                ..Default::default()
            },
        )
        .await
        .unwrap();
        assert_eq!(res_4700r.components.len(), 2);
        assert!(
            res_4700r
                .parsed_query
                .as_ref()
                .and_then(|p| p.summary_badge())
                .is_some_and(|b| b.contains("4.7kΩ"))
        );

        // Shorthand capacitance query: "47n" matches the 0.047uF capacitor
        let res_47n = query_components(
            &db,
            &ComponentQueryFilter {
                q: Some("47n".into()),
                ..Default::default()
            },
        )
        .await
        .unwrap();
        assert_eq!(res_47n.components.len(), 1);
        assert_eq!(res_47n.components[0].id, c_47nf.id);

        // Shorthand pot query with taper: "100kB" matches only the B-taper 100k pot, not the A-taper 100k pot
        let res_100kb = query_components(
            &db,
            &ComponentQueryFilter {
                q: Some("100kB".into()),
                ..Default::default()
            },
        )
        .await
        .unwrap();
        assert_eq!(res_100kb.components.len(), 1);
        assert_eq!(res_100kb.components[0].id, pot_100kb.id);
        assert_ne!(res_100kb.components[0].id, pot_100ka.id);

        // Category + json_extract spec_filters: Resistors with tolerance = "1%" and composition = "Metal Film"
        let mut spec_filters = BTreeMap::new();
        spec_filters.insert("tolerance".into(), "1%".into());
        spec_filters.insert("composition".into(), "Metal Film".into());
        let res_filtered = query_components(
            &db,
            &ComponentQueryFilter {
                category: Some("Resistors".into()),
                spec_filters,
                ..Default::default()
            },
        )
        .await
        .unwrap();
        assert_eq!(res_filtered.components.len(), 1);
        assert_eq!(res_filtered.components[0].id, r_4k7_1pct.id);
        assert_ne!(res_filtered.components[0].id, r_4k7_5pct.id);

        // Recursive Storage Location subtree filter: selecting Crate 1 returns components in Passives Bag AND Film Caps Sub-bag
        let res_crate_1 = query_components(
            &db,
            &ComponentQueryFilter {
                location_id: Some(crate_1.id.clone()),
                ..Default::default()
            },
        )
        .await
        .unwrap();
        assert_eq!(res_crate_1.components.len(), 2);
        let crate_1_comp_ids: Vec<&str> = res_crate_1
            .components
            .iter()
            .map(|c| c.id.as_str())
            .collect();
        assert!(crate_1_comp_ids.contains(&r_4k7_1pct.id.as_str()));
        assert!(crate_1_comp_ids.contains(&c_47nf.id.as_str()));
    }
}
