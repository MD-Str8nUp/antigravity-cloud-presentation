use crate::builds::{get_kit_build, link_bom_line_component};
use crate::catalog::{
    Component, CreateComponent, CreateInitialStockLot, CreateStockLot, StockLot, create_component,
    get_component, list_components, upsert_or_merge_stock_lot,
};
use crate::db::Db;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShoppingListQuery {
    pub kit_build_ids: Vec<String>,
    pub include_low_stock_restock: bool,
    pub supplier_filter: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShoppingListBuildBreakdown {
    pub kit_build_id: String,
    pub kit_build_name: String,
    pub bom_line_id: String,
    pub quantity_needed: i64,
    pub reference_designators: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShoppingListLine {
    pub group_key: String,
    pub component_id: Option<String>,
    pub category: String,
    pub spec_or_component_name: String,
    pub raw_spec_text: String,
    pub canonical_value_display: String,
    pub package_or_mounting: String,
    pub total_needed: i64,
    pub available_stock: i64,
    pub low_stock_threshold: i64,
    pub net_shortfall: i64,
    pub restock_top_up: i64,
    pub recommended_order_quantity: i64,
    pub order_quantity: i64,
    pub supplier: String,
    pub supplier_part_no: String,
    pub notes: String,
    pub has_override: bool,
    pub build_breakdown: Vec<ShoppingListBuildBreakdown>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShoppingListReport {
    pub selected_kit_build_ids: Vec<String>,
    pub include_low_stock_restock: bool,
    pub supplier_filter: Option<String>,
    pub available_suppliers: Vec<String>,
    pub total_shortfall_lines: usize,
    pub total_restock_only_lines: usize,
    pub total_order_quantity: i64,
    pub lines: Vec<ShoppingListLine>,
}

pub fn unlinked_group_key(
    category: &str,
    canonical_value_display: &str,
    raw_spec_text: &str,
    package_or_mounting: &str,
) -> String {
    let canonical = if !canonical_value_display.trim().is_empty() {
        canonical_value_display.trim()
    } else {
        raw_spec_text.trim()
    };
    format!(
        "unlinked:{}:{}:{}",
        category.trim(),
        canonical,
        package_or_mounting.trim()
    )
}

fn default_supplier_from_component(comp: &Component) -> (String, String) {
    let mut supplier = String::new();
    let mut part_no = String::new();
    for lot in &comp.lots {
        if supplier.is_empty() && !lot.supplier.trim().is_empty() {
            supplier = lot.supplier.trim().to_string();
        }
        if part_no.is_empty() && !lot.supplier_part_no.trim().is_empty() {
            part_no = lot.supplier_part_no.trim().to_string();
        }
        if !supplier.is_empty() && !part_no.is_empty() {
            break;
        }
    }
    (supplier, part_no)
}

struct GroupAccumulator {
    group_key: String,
    component_id: Option<String>,
    component: Option<Component>,
    category: String,
    spec_or_component_name: String,
    raw_spec_text: String,
    canonical_value_display: String,
    package_or_mounting: String,
    total_needed: i64,
    notes: String,
    build_breakdown: Vec<ShoppingListBuildBreakdown>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShoppingListOverride {
    pub group_key: String,
    pub order_quantity: Option<i64>,
    pub supplier: Option<String>,
    pub supplier_part_no: Option<String>,
    pub notes: Option<String>,
}

pub async fn get_shopping_list_override(
    db: &Db,
    group_key: &str,
) -> anyhow::Result<Option<ShoppingListOverride>> {
    let conn = db.connect().await?;
    let mut rows = conn
        .query(
            "SELECT group_key, order_quantity, supplier, supplier_part_no, notes
             FROM shopping_list_overrides
             WHERE group_key = ?1",
            (group_key,),
        )
        .await?;
    if let Some(row) = rows.next().await? {
        Ok(Some(ShoppingListOverride {
            group_key: row.get::<String>(0)?,
            order_quantity: row.get::<Option<i64>>(1)?,
            supplier: row.get::<Option<String>>(2)?,
            supplier_part_no: row.get::<Option<String>>(3)?,
            notes: row.get::<Option<String>>(4)?,
        }))
    } else {
        Ok(None)
    }
}

pub async fn list_shopping_list_overrides(
    db: &Db,
) -> anyhow::Result<BTreeMap<String, ShoppingListOverride>> {
    let conn = db.connect().await?;
    let mut rows = conn
        .query(
            "SELECT group_key, order_quantity, supplier, supplier_part_no, notes
             FROM shopping_list_overrides",
            (),
        )
        .await?;
    let mut map = BTreeMap::new();
    while let Some(row) = rows.next().await? {
        let ov = ShoppingListOverride {
            group_key: row.get::<String>(0)?,
            order_quantity: row.get::<Option<i64>>(1)?,
            supplier: row.get::<Option<String>>(2)?,
            supplier_part_no: row.get::<Option<String>>(3)?,
            notes: row.get::<Option<String>>(4)?,
        };
        map.insert(ov.group_key.clone(), ov);
    }
    Ok(map)
}

pub async fn upsert_shopping_list_override(
    db: &Db,
    group_key: &str,
    order_quantity: Option<i64>,
    supplier: Option<&str>,
    supplier_part_no: Option<&str>,
    notes: Option<&str>,
) -> anyhow::Result<ShoppingListOverride> {
    let gkey = group_key.trim();
    if gkey.is_empty() {
        anyhow::bail!("Shopping List group_key cannot be empty");
    }

    let existing = get_shopping_list_override(db, gkey).await?;
    let next_qty = order_quantity
        .map(|q| q.max(0))
        .or_else(|| existing.as_ref().and_then(|e| e.order_quantity));
    let next_sup = match supplier {
        Some(s) => Some(s.trim().to_string()),
        None => existing.as_ref().and_then(|e| e.supplier.clone()),
    };
    let next_pno = match supplier_part_no {
        Some(s) => Some(s.trim().to_string()),
        None => existing.as_ref().and_then(|e| e.supplier_part_no.clone()),
    };
    let next_notes = match notes {
        Some(s) => Some(s.trim().to_string()),
        None => existing.as_ref().and_then(|e| e.notes.clone()),
    };

    let conn = db.connect().await?;
    conn.execute(
        "INSERT INTO shopping_list_overrides (group_key, order_quantity, supplier, supplier_part_no, notes, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, datetime('now'))
         ON CONFLICT(group_key) DO UPDATE SET
            order_quantity = excluded.order_quantity,
            supplier = excluded.supplier,
            supplier_part_no = excluded.supplier_part_no,
            notes = excluded.notes,
            updated_at = datetime('now')",
        (
            gkey,
            next_qty,
            next_sup.as_deref(),
            next_pno.as_deref(),
            next_notes.as_deref(),
        ),
    )
    .await?;

    Ok(ShoppingListOverride {
        group_key: gkey.to_string(),
        order_quantity: next_qty,
        supplier: next_sup,
        supplier_part_no: next_pno,
        notes: next_notes,
    })
}

pub async fn clear_shopping_list_override(db: &Db, group_key: &str) -> anyhow::Result<()> {
    let conn = db.connect().await?;
    conn.execute(
        "DELETE FROM shopping_list_overrides WHERE group_key = ?1",
        (group_key.trim(),),
    )
    .await?;
    Ok(())
}

pub fn format_line_builds_summary(line: &ShoppingListLine) -> String {
    if line.build_breakdown.is_empty() {
        "Low-Stock Restock".to_string()
    } else {
        line.build_breakdown
            .iter()
            .map(|b| format!("{} ({})", b.kit_build_name, b.quantity_needed))
            .collect::<Vec<_>>()
            .join("; ")
    }
}

pub fn format_line_designators_summary(line: &ShoppingListLine) -> String {
    line.build_breakdown
        .iter()
        .map(|b| b.reference_designators.trim())
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("; ")
}

pub fn format_line_customer_reference(line: &ShoppingListLine) -> String {
    if line.build_breakdown.is_empty() {
        format!("{} [Restock]", line.spec_or_component_name)
    } else {
        line.build_breakdown
            .iter()
            .map(|b| {
                let des = b.reference_designators.trim();
                if des.is_empty() {
                    b.kit_build_name.clone()
                } else {
                    format!("{} [{}]", b.kit_build_name, des)
                }
            })
            .collect::<Vec<_>>()
            .join("; ")
    }
}

pub fn export_full_shopping_list_csv(report: &ShoppingListReport) -> String {
    let mut wtr = csv::Writer::from_writer(Vec::new());
    let _ = wtr.write_record([
        "Category",
        "Spec / Component",
        "Package",
        "Total Needed",
        "Available Stock",
        "Net Shortfall",
        "Restock Top-Up",
        "Order Quantity",
        "Supplier",
        "Supplier Part #",
        "Kit Builds",
        "Reference Designators",
    ]);

    for line in &report.lines {
        let _ = wtr.write_record([
            line.category.as_str(),
            line.spec_or_component_name.as_str(),
            line.package_or_mounting.as_str(),
            &line.total_needed.to_string(),
            &line.available_stock.to_string(),
            &line.net_shortfall.to_string(),
            &line.restock_top_up.to_string(),
            &line.order_quantity.to_string(),
            line.supplier.as_str(),
            line.supplier_part_no.as_str(),
            &format_line_builds_summary(line),
            &format_line_designators_summary(line),
        ]);
    }

    let bytes = wtr.into_inner().unwrap_or_default();
    String::from_utf8(bytes).unwrap_or_default()
}

pub fn export_supplier_quick_order_csv(
    report: &ShoppingListReport,
    supplier_filter: Option<&str>,
) -> String {
    let active_filter = supplier_filter
        .or(report.supplier_filter.as_deref())
        .map(str::trim)
        .filter(|s| !s.is_empty() && !s.eq_ignore_ascii_case("All"));

    let mut wtr = csv::Writer::from_writer(Vec::new());
    let _ = wtr.write_record(["Supplier Part #", "Order Quantity", "Customer Reference"]);

    for line in &report.lines {
        if line.order_quantity <= 0 {
            continue;
        }
        if let Some(sup_filter) = active_filter
            && !line.supplier.eq_ignore_ascii_case(sup_filter)
        {
            continue;
        }
        let part_no = if !line.supplier_part_no.trim().is_empty() {
            line.supplier_part_no.trim()
        } else {
            line.spec_or_component_name.trim()
        };
        let customer_ref = format_line_customer_reference(line);
        let _ = wtr.write_record([part_no, &line.order_quantity.to_string(), &customer_ref]);
    }

    let bytes = wtr.into_inner().unwrap_or_default();
    String::from_utf8(bytes).unwrap_or_default()
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReceiveShoppingListLine {
    pub group_key: String,
    pub component_id: Option<String>,
    pub category: String,
    pub raw_spec_text: String,
    pub canonical_value_display: String,
    pub package_or_mounting: String,
    pub quantity_received: i64,
    pub supplier: String,
    pub supplier_part_no: String,
    pub bom_line_ids: Vec<String>,
}

impl ReceiveShoppingListLine {
    pub fn from_shopping_list_line(line: &ShoppingListLine) -> Self {
        Self {
            group_key: line.group_key.clone(),
            component_id: line.component_id.clone(),
            category: line.category.clone(),
            raw_spec_text: line.raw_spec_text.clone(),
            canonical_value_display: line.canonical_value_display.clone(),
            package_or_mounting: line.package_or_mounting.clone(),
            quantity_received: line.order_quantity,
            supplier: line.supplier.clone(),
            supplier_part_no: line.supplier_part_no.clone(),
            bom_line_ids: line
                .build_breakdown
                .iter()
                .map(|b| b.bom_line_id.clone())
                .collect(),
        }
    }
}

impl From<&ShoppingListLine> for ReceiveShoppingListLine {
    fn from(line: &ShoppingListLine) -> Self {
        Self::from_shopping_list_line(line)
    }
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ReceiveShoppingListResult {
    pub updated_lots: Vec<StockLot>,
    pub created_components: Vec<Component>,
    pub linked_bom_line_ids: Vec<String>,
}

pub async fn receive_shopping_list_lines(
    db: &Db,
    target_location_id: Option<&str>,
    lines_to_receive: &[ReceiveShoppingListLine],
) -> anyhow::Result<ReceiveShoppingListResult> {
    let normalized_location_id = target_location_id
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string);

    let mut result = ReceiveShoppingListResult::default();

    for item in lines_to_receive {
        if item.quantity_received <= 0 {
            continue;
        }

        let resolved_cid = item
            .component_id
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .or_else(|| {
                item.group_key
                    .strip_prefix("comp:")
                    .map(str::trim)
                    .filter(|s| !s.is_empty())
            });

        if let Some(cid) = resolved_cid {
            let lot = upsert_or_merge_stock_lot(
                db,
                CreateStockLot {
                    component_id: cid.to_string(),
                    location_id: normalized_location_id.clone(),
                    quantity: item.quantity_received,
                    supplier: item.supplier.clone(),
                    supplier_part_no: item.supplier_part_no.clone(),
                    lot_notes: "Received from Shopping List".to_string(),
                },
            )
            .await?;
            result.updated_lots.push(lot);
            let _ = clear_shopping_list_override(db, &item.group_key).await;
        } else {
            // Find all unlinked BOM lines matching item.bom_line_ids or item.group_key
            let mut matching_bom_ids: BTreeSet<String> =
                item.bom_line_ids.iter().cloned().collect();

            let conn = db.connect().await?;
            let mut rows = conn
                .query(
                    "SELECT id, category, canonical_value_display, raw_spec_text, package_or_mounting
                     FROM bom_lines
                     WHERE component_id IS NULL AND included_in_kit = 0",
                    (),
                )
                .await?;

            let mut fallback_cat = item.category.trim().to_string();
            let mut fallback_canonical = item.canonical_value_display.trim().to_string();
            let mut fallback_raw = item.raw_spec_text.trim().to_string();
            let mut fallback_pkg = item.package_or_mounting.trim().to_string();

            while let Some(row) = rows.next().await? {
                let bid = row.get::<String>(0)?;
                let bcat = row.get::<String>(1)?;
                let bcan = row.get::<String>(2)?;
                let braw = row.get::<String>(3)?;
                let bpkg = row.get::<String>(4)?;

                let b_key = unlinked_group_key(&bcat, &bcan, &braw, &bpkg);
                if b_key == item.group_key || matching_bom_ids.contains(&bid) {
                    matching_bom_ids.insert(bid);
                    if fallback_cat.is_empty() {
                        fallback_cat = bcat;
                    }
                    if fallback_canonical.is_empty() {
                        fallback_canonical = bcan;
                    }
                    if fallback_raw.is_empty() {
                        fallback_raw = braw;
                    }
                    if fallback_pkg.is_empty() {
                        fallback_pkg = bpkg;
                    }
                }
            }
            drop(rows);

            if let Some(rest) = item.group_key.strip_prefix("unlinked:") {
                let parts: Vec<&str> = rest.splitn(3, ':').collect();
                if parts.len() == 3 {
                    if fallback_cat.is_empty() {
                        fallback_cat = parts[0].to_string();
                    }
                    if fallback_canonical.is_empty() {
                        fallback_canonical = parts[1].to_string();
                    }
                    if fallback_pkg.is_empty() {
                        fallback_pkg = parts[2].to_string();
                    }
                }
            }
            if fallback_raw.is_empty() {
                fallback_raw = fallback_canonical.clone();
            }

            let created_comp = create_component(
                db,
                CreateComponent {
                    name: String::new(),
                    category: if fallback_cat.is_empty() {
                        "Other / Custom".to_string()
                    } else {
                        fallback_cat
                    },
                    subtype: String::new(),
                    package_or_mounting: fallback_pkg,
                    raw_spec_text: fallback_raw,
                    canonical_value_display: if fallback_canonical.is_empty() {
                        None
                    } else {
                        Some(fallback_canonical)
                    },
                    resistance_ohms: None,
                    capacitance_pf: None,
                    inductance_uh: None,
                    voltage_rating_volts: None,
                    low_stock_threshold: 0,
                    datasheet_url: String::new(),
                    notes: String::new(),
                    specs_json: None,
                    initial_lot: Some(CreateInitialStockLot {
                        location_id: normalized_location_id.clone(),
                        quantity: item.quantity_received,
                        supplier: item.supplier.clone(),
                        supplier_part_no: item.supplier_part_no.clone(),
                        lot_notes: "Received from Shopping List".to_string(),
                    }),
                },
            )
            .await?;

            if let Some(first_lot) = created_comp.lots.first() {
                result.updated_lots.push(first_lot.clone());
            }

            for bid in matching_bom_ids {
                link_bom_line_component(db, &bid, Some(&created_comp.id), None).await?;
                result.linked_bom_line_ids.push(bid);
            }

            result.created_components.push(created_comp);
            let _ = clear_shopping_list_override(db, &item.group_key).await;
        }
    }

    Ok(result)
}

pub async fn compute_shopping_list(
    db: &Db,
    query: &ShoppingListQuery,
) -> anyhow::Result<ShoppingListReport> {
    let mut groups: BTreeMap<String, GroupAccumulator> = BTreeMap::new();
    let mut group_order: Vec<String> = Vec::new();

    for build_id in &query.kit_build_ids {
        let Some(build) = get_kit_build(db, build_id).await? else {
            continue;
        };

        for line in &build.bom_lines {
            if line.included_in_kit
                || matches!(
                    line.fulfillment_status.as_str(),
                    "Included in Kit" | "Installed" | "Picked"
                )
            {
                continue;
            }
            let unallocated_needed = (line.quantity_required - line.quantity_allocated).max(0);
            if unallocated_needed <= 0 {
                continue;
            }

            let group_key = if let Some(ref cid) = line.component_id {
                format!("comp:{cid}")
            } else {
                unlinked_group_key(
                    &line.category,
                    &line.canonical_value_display,
                    &line.raw_spec_text,
                    &line.package_or_mounting,
                )
            };

            if !groups.contains_key(&group_key) {
                group_order.push(group_key.clone());
                let comp = if let Some(ref cid) = line.component_id {
                    get_component(db, cid).await?
                } else {
                    None
                };

                let (category, spec_or_component_name, canonical_value_display, package_or_mounting, notes) =
                    if let Some(ref c) = comp {
                        (
                            c.category.clone(),
                            c.name.clone(),
                            c.canonical_value_display.clone(),
                            c.package_or_mounting.clone(),
                            c.notes.clone(),
                        )
                    } else {
                        let canonical = if !line.canonical_value_display.trim().is_empty() {
                            line.canonical_value_display.trim().to_string()
                        } else {
                            line.raw_spec_text.trim().to_string()
                        };
                        (
                            line.category.clone(),
                            canonical.clone(),
                            canonical,
                            line.package_or_mounting.clone(),
                            line.notes.clone(),
                        )
                    };

                groups.insert(
                    group_key.clone(),
                    GroupAccumulator {
                        group_key: group_key.clone(),
                        component_id: line.component_id.clone(),
                        component: comp,
                        category,
                        spec_or_component_name,
                        raw_spec_text: line.raw_spec_text.clone(),
                        canonical_value_display,
                        package_or_mounting,
                        total_needed: 0,
                        notes,
                        build_breakdown: Vec::new(),
                    },
                );
            }

            if let Some(acc) = groups.get_mut(&group_key) {
                acc.total_needed += unallocated_needed;
                acc.build_breakdown.push(ShoppingListBuildBreakdown {
                    kit_build_id: build.id.clone(),
                    kit_build_name: build.name.clone(),
                    bom_line_id: line.id.clone(),
                    quantity_needed: unallocated_needed,
                    reference_designators: line.reference_designators.clone(),
                });
            }
        }
    }

    if query.include_low_stock_restock {
        let all_components = list_components(db).await?;
        for comp in all_components {
            if comp.low_stock_threshold <= 0 {
                continue;
            }
            let group_key = format!("comp:{}", comp.id);
            if !groups.contains_key(&group_key) && comp.available_stock < comp.low_stock_threshold {
                group_order.push(group_key.clone());
                groups.insert(
                    group_key.clone(),
                    GroupAccumulator {
                        group_key,
                        component_id: Some(comp.id.clone()),
                        category: comp.category.clone(),
                        spec_or_component_name: comp.name.clone(),
                        raw_spec_text: comp.raw_spec_text.clone(),
                        canonical_value_display: comp.canonical_value_display.clone(),
                        package_or_mounting: comp.package_or_mounting.clone(),
                        total_needed: 0,
                        notes: comp.notes.clone(),
                        build_breakdown: Vec::new(),
                        component: Some(comp),
                    },
                );
            }
        }
    }

    let overrides_map = list_shopping_list_overrides(db).await?;
    let mut available_suppliers_set = BTreeSet::new();
    let mut all_lines = Vec::new();

    for gkey in group_order {
        let Some(acc) = groups.remove(&gkey) else {
            continue;
        };

        let (available_stock, low_stock_threshold, default_supplier, default_part_no) =
            if let Some(ref comp) = acc.component {
                let (sup, pno) = default_supplier_from_component(comp);
                (
                    comp.available_stock,
                    comp.low_stock_threshold,
                    sup,
                    pno,
                )
            } else {
                (0, 0, String::new(), String::new())
            };

        let net_shortfall = (acc.total_needed - available_stock).max(0);
        let restock_top_up = if query.include_low_stock_restock
            && low_stock_threshold > 0
            && (available_stock - acc.total_needed) < low_stock_threshold
        {
            (low_stock_threshold - (available_stock - acc.total_needed).max(0)).max(0)
        } else {
            0
        };
        let recommended_order_quantity = net_shortfall + restock_top_up;

        let ov = overrides_map.get(&acc.group_key);
        let order_quantity = ov
            .and_then(|o| o.order_quantity)
            .unwrap_or(recommended_order_quantity);

        if recommended_order_quantity <= 0 && order_quantity <= 0 {
            continue;
        }

        let supplier = ov
            .and_then(|o| o.supplier.as_deref())
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(str::to_string)
            .unwrap_or(default_supplier);

        let supplier_part_no = ov
            .and_then(|o| o.supplier_part_no.as_deref())
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(str::to_string)
            .unwrap_or(default_part_no);

        let notes = ov
            .and_then(|o| o.notes.as_deref())
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(str::to_string)
            .unwrap_or(acc.notes);

        if !supplier.is_empty() {
            available_suppliers_set.insert(supplier.clone());
        }

        all_lines.push(ShoppingListLine {
            group_key: acc.group_key,
            component_id: acc.component_id,
            category: acc.category,
            spec_or_component_name: acc.spec_or_component_name,
            raw_spec_text: acc.raw_spec_text,
            canonical_value_display: acc.canonical_value_display,
            package_or_mounting: acc.package_or_mounting,
            total_needed: acc.total_needed,
            available_stock,
            low_stock_threshold,
            net_shortfall,
            restock_top_up,
            recommended_order_quantity,
            order_quantity,
            supplier,
            supplier_part_no,
            notes,
            has_override: ov.is_some(),
            build_breakdown: acc.build_breakdown,
        });
    }

    let active_supplier_filter = query
        .supplier_filter
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty() && !s.eq_ignore_ascii_case("All"));

    let lines: Vec<ShoppingListLine> = if let Some(sup_filter) = active_supplier_filter {
        all_lines
            .into_iter()
            .filter(|l| l.supplier.eq_ignore_ascii_case(sup_filter))
            .collect()
    } else {
        all_lines
    };

    let total_shortfall_lines = lines.iter().filter(|l| l.net_shortfall > 0).count();
    let total_restock_only_lines = lines
        .iter()
        .filter(|l| l.net_shortfall == 0 && l.restock_top_up > 0)
        .count();
    let total_order_quantity = lines.iter().map(|l| l.order_quantity).sum();

    Ok(ShoppingListReport {
        selected_kit_build_ids: query.kit_build_ids.clone(),
        include_low_stock_restock: query.include_low_stock_restock,
        supplier_filter: query.supplier_filter.clone(),
        available_suppliers: available_suppliers_set.into_iter().collect(),
        total_shortfall_lines,
        total_restock_only_lines,
        total_order_quantity,
        lines,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::builds::{
        CreateBomLine, CreateKitBuild, add_bom_line, create_kit_build, transition_bom_line_status,
    };
    use crate::catalog::{
        CreateComponent, CreateInitialStockLot, CreateStorageLocation, create_component,
        create_storage_location,
    };
    use crate::db::init_memory_db;

    #[tokio::test]
    async fn aggregates_multi_build_demand_before_subtracting_available_stock_and_groups_unlinked_specs()
     {
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

        // Component 1: 4.7kΩ resistor with 5 on hand (Available Stock = 5)
        let r_4k7 = create_component(
            &db,
            CreateComponent {
                name: "4.7kΩ 1% Metal Film Resistor".into(),
                category: "Resistors".into(),
                subtype: "Metal Film".into(),
                package_or_mounting: "Through-Hole".into(),
                raw_spec_text: "4k7 1% 1/4W Metal Film".into(),
                canonical_value_display: None,
                resistance_ohms: None,
                capacitance_pf: None,
                inductance_uh: None,
                voltage_rating_volts: None,
                low_stock_threshold: 0,
                datasheet_url: "".into(),
                notes: "".into(),
                specs_json: None,
                initial_lot: Some(CreateInitialStockLot {
                    location_id: Some(binder.id.clone()),
                    quantity: 5,
                    supplier: "Tayda".into(),
                    supplier_part_no: "A-2214".into(),
                    lot_notes: "".into(),
                }),
            },
        )
        .await
        .unwrap();

        // Component 2: TL072 op-amp with 3 on hand
        let tl072 = create_component(
            &db,
            CreateComponent {
                name: "TL072IP Dual Op-Amp".into(),
                category: "Integrated Circuits (ICs)".into(),
                subtype: "Op-Amp".into(),
                package_or_mounting: "DIP-8".into(),
                raw_spec_text: "TL072IP DIP-8".into(),
                canonical_value_display: None,
                resistance_ohms: None,
                capacitance_pf: None,
                inductance_uh: None,
                voltage_rating_volts: None,
                low_stock_threshold: 0,
                datasheet_url: "".into(),
                notes: "".into(),
                specs_json: None,
                initial_lot: Some(CreateInitialStockLot {
                    location_id: Some(binder.id.clone()),
                    quantity: 3,
                    supplier: "Mouser".into(),
                    supplier_part_no: "595-TL072IP".into(),
                    lot_notes: "".into(),
                }),
            },
        )
        .await
        .unwrap();

        let build1 = create_kit_build(
            &db,
            CreateKitBuild {
                name: "Klon Centaur (#1)".into(),
                project_type: "Overdrive".into(),
                status: Some("Sourcing / Waiting on Parts".into()),
                location_id: None,
                description: "".into(),
            },
        )
        .await
        .unwrap();

        let build2 = create_kit_build(
            &db,
            CreateKitBuild {
                name: "Tube Screamer TS808".into(),
                project_type: "Overdrive".into(),
                status: Some("Wishlist / Planned".into()),
                location_id: None,
                description: "".into(),
            },
        )
        .await
        .unwrap();

        // Build 1 needs 4x 4.7kΩ (Unallocated) -> alone 4 <= 5 Available Stock (0 shortfall)
        let _b1_r = add_bom_line(
            &db,
            CreateBomLine {
                kit_build_id: build1.id.clone(),
                raw_spec_text: "4k7 1% 1/4W Metal Film".into(),
                category: Some("Resistors".into()),
                package_or_mounting: Some("Through-Hole".into()),
                quantity_required: 4,
                reference_designators: "R1, R2, R3, R4".into(),
                included_in_kit: false,
                component_id: Some(r_4k7.id.clone()),
                substitution_note: "".into(),
                notes: "".into(),
            },
        )
        .await
        .unwrap();

        // Build 2 ALSO needs 4x 4.7kΩ (Unallocated) -> combined 4 + 4 = 8 vs 5 Available Stock => Net Shortfall = 3!
        let _b2_r = add_bom_line(
            &db,
            CreateBomLine {
                kit_build_id: build2.id.clone(),
                raw_spec_text: "4.7k 1%".into(),
                category: Some("Resistors".into()),
                package_or_mounting: Some("Through-Hole".into()),
                quantity_required: 4,
                reference_designators: "R10, R11, R12, R13".into(),
                included_in_kit: false,
                component_id: Some(r_4k7.id.clone()),
                substitution_note: "".into(),
                notes: "".into(),
            },
        )
        .await
        .unwrap();

        // Build 1 has a partially-reserved TL072 line: requires 5, reserved 2 (Available Stock becomes 3 - 2 = 1; unallocated_needed = 3; Net Shortfall = 3 - 1 = 2)
        let b1_ic = add_bom_line(
            &db,
            CreateBomLine {
                kit_build_id: build1.id.clone(),
                raw_spec_text: "TL072IP".into(),
                category: Some("Integrated Circuits (ICs)".into()),
                package_or_mounting: Some("DIP-8".into()),
                quantity_required: 5,
                reference_designators: "U1, U2, U3, U4, U5".into(),
                included_in_kit: false,
                component_id: Some(tl072.id.clone()),
                substitution_note: "".into(),
                notes: "".into(),
            },
        )
        .await
        .unwrap();
        transition_bom_line_status(&db, &b1_ic.id, "Reserved", None, Some(2))
            .await
            .unwrap();

        // Build 1 also has an Included in Kit line (must be ignored)
        let _b1_kit_line = add_bom_line(
            &db,
            CreateBomLine {
                kit_build_id: build1.id.clone(),
                raw_spec_text: "Custom Klon Enclosure".into(),
                category: Some("Enclosures, Knobs & Mechanical Hardware".into()),
                package_or_mounting: None,
                quantity_required: 1,
                reference_designators: "ENC1".into(),
                included_in_kit: true,
                component_id: None,
                substitution_note: "".into(),
                notes: "".into(),
            },
        )
        .await
        .unwrap();

        // Unlinked BOM lines across Build 1 ("0.047uF Film", 2x) and Build 2 ("47n Film", 3x) with same normalized spec
        let _b1_unlinked = add_bom_line(
            &db,
            CreateBomLine {
                kit_build_id: build1.id.clone(),
                raw_spec_text: "0.047uF Film".into(),
                category: Some("Capacitors".into()),
                package_or_mounting: Some("Through-Hole 5mm".into()),
                quantity_required: 2,
                reference_designators: "C1, C2".into(),
                included_in_kit: false,
                component_id: None,
                substitution_note: "".into(),
                notes: "".into(),
            },
        )
        .await
        .unwrap();

        let _b2_unlinked = add_bom_line(
            &db,
            CreateBomLine {
                kit_build_id: build2.id.clone(),
                raw_spec_text: "47n Film".into(),
                category: Some("Capacitors".into()),
                package_or_mounting: Some("Through-Hole 5mm".into()),
                quantity_required: 3,
                reference_designators: "C5, C6, C7".into(),
                included_in_kit: false,
                component_id: None,
                substitution_note: "".into(),
                notes: "".into(),
            },
        )
        .await
        .unwrap();

        // 1. Single build (Build 1 only): 4.7kΩ has 4 needed vs 5 available -> 0 shortfall (not in shortfall list)
        let single_report = compute_shopping_list(
            &db,
            &ShoppingListQuery {
                kit_build_ids: vec![build1.id.clone()],
                include_low_stock_restock: false,
                supplier_filter: None,
            },
        )
        .await
        .unwrap();
        assert!(
            single_report
                .lines
                .iter()
                .all(|l| l.component_id.as_deref() != Some(r_4k7.id.as_str())),
            "4.7kΩ resistor should have 0 shortfall when only Build 1 is selected"
        );

        // 2. Both builds selected: 4.7kΩ has 4 + 4 = 8 needed vs 5 Available Stock -> Net Shortfall = 3!
        let multi_report = compute_shopping_list(
            &db,
            &ShoppingListQuery {
                kit_build_ids: vec![build1.id.clone(), build2.id.clone()],
                include_low_stock_restock: false,
                supplier_filter: None,
            },
        )
        .await
        .unwrap();

        assert_eq!(multi_report.lines.len(), 3);

        let r_line = multi_report
            .lines
            .iter()
            .find(|l| l.component_id.as_deref() == Some(r_4k7.id.as_str()))
            .expect("4.7kΩ line in multi-build report");
        assert_eq!(r_line.group_key, format!("comp:{}", r_4k7.id));
        assert_eq!(r_line.total_needed, 8);
        assert_eq!(r_line.available_stock, 5);
        assert_eq!(r_line.net_shortfall, 3);
        assert_eq!(r_line.restock_top_up, 0);
        assert_eq!(r_line.order_quantity, 3);
        assert_eq!(r_line.supplier, "Tayda");
        assert_eq!(r_line.supplier_part_no, "A-2214");
        assert_eq!(r_line.build_breakdown.len(), 2);
        assert_eq!(r_line.build_breakdown[0].kit_build_name, "Klon Centaur (#1)");
        assert_eq!(r_line.build_breakdown[0].quantity_needed, 4);
        assert_eq!(r_line.build_breakdown[0].reference_designators, "R1, R2, R3, R4");
        assert_eq!(r_line.build_breakdown[1].kit_build_name, "Tube Screamer TS808");
        assert_eq!(r_line.build_breakdown[1].quantity_needed, 4);
        assert_eq!(
            r_line.build_breakdown[1].reference_designators,
            "R10, R11, R12, R13"
        );

        // Partially reserved TL072: 5 required - 2 reserved = 3 unallocated_needed; Available Stock = 3 - 2 = 1; Net Shortfall = 2
        let ic_line = multi_report
            .lines
            .iter()
            .find(|l| l.component_id.as_deref() == Some(tl072.id.as_str()))
            .expect("TL072 line in report");
        assert_eq!(ic_line.total_needed, 3);
        assert_eq!(ic_line.available_stock, 1);
        assert_eq!(ic_line.net_shortfall, 2);
        assert_eq!(ic_line.order_quantity, 2);
        assert_eq!(ic_line.supplier, "Mouser");
        assert_eq!(ic_line.supplier_part_no, "595-TL072IP");

        // Unlinked 47nF capacitor grouped across both builds: 2 + 3 = 5 needed, 0 available -> Net Shortfall = 5
        let cap_line = multi_report
            .lines
            .iter()
            .find(|l| l.component_id.is_none())
            .expect("unlinked 47nF line in report");
        assert_eq!(
            cap_line.group_key,
            "unlinked:Capacitors:47nF:Through-Hole 5mm"
        );
        assert_eq!(cap_line.canonical_value_display, "47nF");
        assert_eq!(cap_line.total_needed, 5);
        assert_eq!(cap_line.available_stock, 0);
        assert_eq!(cap_line.net_shortfall, 5);
        assert_eq!(cap_line.order_quantity, 5);
        assert_eq!(cap_line.build_breakdown.len(), 2);
    }

    #[tokio::test]
    async fn includes_low_stock_catalog_restock_top_ups_for_both_build_and_standalone_components() {
        let db = init_memory_db().await.expect("memory db");

        let loc = create_storage_location(
            &db,
            CreateStorageLocation {
                name: "Drawer 1".into(),
                kind: "Drawer".into(),
                parent_id: None,
                notes: "".into(),
            },
        )
        .await
        .unwrap();

        // Case 1: In build demand AND has shortfall (Available = 5, Needed = 8, Threshold = 10)
        // Net Shortfall = 3, Restock Top-Up = 10, Recommended Order = 13 (= 10 - (5 - 8))
        let comp_shortfall_and_restock = create_component(
            &db,
            CreateComponent {
                name: "10kΩ Resistor".into(),
                category: "Resistors".into(),
                subtype: "Metal Film".into(),
                package_or_mounting: "Through-Hole".into(),
                raw_spec_text: "10k 1% 1/4W".into(),
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
                    location_id: Some(loc.id.clone()),
                    quantity: 5,
                    supplier: "Tayda".into(),
                    supplier_part_no: "A-2200".into(),
                    lot_notes: "".into(),
                }),
            },
        )
        .await
        .unwrap();

        // Case 2: In build demand with enough stock for build (Available = 8, Needed = 5 -> Shortfall = 0),
        // but post-build remaining stock (8 - 5 = 3) falls below low_stock_threshold (6) -> Restock Top-Up = 3
        let comp_post_build_low = create_component(
            &db,
            CreateComponent {
                name: "100nF Film Cap".into(),
                category: "Capacitors".into(),
                subtype: "Film".into(),
                package_or_mounting: "Through-Hole 5mm".into(),
                raw_spec_text: "100nF 63V Box Film".into(),
                canonical_value_display: None,
                resistance_ohms: None,
                capacitance_pf: None,
                inductance_uh: None,
                voltage_rating_volts: None,
                low_stock_threshold: 6,
                datasheet_url: "".into(),
                notes: "".into(),
                specs_json: None,
                initial_lot: Some(CreateInitialStockLot {
                    location_id: Some(loc.id.clone()),
                    quantity: 8,
                    supplier: "Mouser".into(),
                    supplier_part_no: "WIMA-100N".into(),
                    lot_notes: "".into(),
                }),
            },
        )
        .await
        .unwrap();

        // Case 3: Standalone catalog staple (Needed = 0, Available = 2, Threshold = 10) -> Restock Top-Up = 8
        let comp_standalone_low = create_component(
            &db,
            CreateComponent {
                name: "1N4148 Switching Diode".into(),
                category: "Semiconductors — Discrete".into(),
                subtype: "Diode".into(),
                package_or_mounting: "DO-35".into(),
                raw_spec_text: "1N4148 Silicon Diode".into(),
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
                    location_id: Some(loc.id.clone()),
                    quantity: 2,
                    supplier: "Tayda".into(),
                    supplier_part_no: "A-157".into(),
                    lot_notes: "".into(),
                }),
            },
        )
        .await
        .unwrap();

        // Case 4: Standalone catalog component with plenty of stock (Available = 15, Threshold = 10) -> Excluded
        let _comp_healthy = create_component(
            &db,
            CreateComponent {
                name: "2N3904 NPN Transistor".into(),
                category: "Semiconductors — Discrete".into(),
                subtype: "BJT".into(),
                package_or_mounting: "TO-92".into(),
                raw_spec_text: "2N3904 NPN".into(),
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
                    location_id: Some(loc.id.clone()),
                    quantity: 15,
                    supplier: "Tayda".into(),
                    supplier_part_no: "A-111".into(),
                    lot_notes: "".into(),
                }),
            },
        )
        .await
        .unwrap();

        let build = create_kit_build(
            &db,
            CreateKitBuild {
                name: "Fuzz Face".into(),
                project_type: "Fuzz".into(),
                status: Some("Sourcing / Waiting on Parts".into()),
                location_id: None,
                description: "".into(),
            },
        )
        .await
        .unwrap();

        add_bom_line(
            &db,
            CreateBomLine {
                kit_build_id: build.id.clone(),
                raw_spec_text: "10k 1%".into(),
                category: Some("Resistors".into()),
                package_or_mounting: Some("Through-Hole".into()),
                quantity_required: 8,
                reference_designators: "R1, R2, R3, R4, R5, R6, R7, R8".into(),
                included_in_kit: false,
                component_id: Some(comp_shortfall_and_restock.id.clone()),
                substitution_note: "".into(),
                notes: "".into(),
            },
        )
        .await
        .unwrap();

        add_bom_line(
            &db,
            CreateBomLine {
                kit_build_id: build.id.clone(),
                raw_spec_text: "100nF".into(),
                category: Some("Capacitors".into()),
                package_or_mounting: Some("Through-Hole 5mm".into()),
                quantity_required: 5,
                reference_designators: "C1, C2, C3, C4, C5".into(),
                included_in_kit: false,
                component_id: Some(comp_post_build_low.id.clone()),
                substitution_note: "".into(),
                notes: "".into(),
            },
        )
        .await
        .unwrap();

        // With include_low_stock_restock = false: only 10kΩ resistor appears with order_quantity = 3
        let report_no_restock = compute_shopping_list(
            &db,
            &ShoppingListQuery {
                kit_build_ids: vec![build.id.clone()],
                include_low_stock_restock: false,
                supplier_filter: None,
            },
        )
        .await
        .unwrap();
        assert_eq!(report_no_restock.lines.len(), 1);
        assert_eq!(report_no_restock.lines[0].net_shortfall, 3);
        assert_eq!(report_no_restock.lines[0].restock_top_up, 0);
        assert_eq!(report_no_restock.lines[0].order_quantity, 3);

        // With include_low_stock_restock = true: all 3 low-stock components appear with exact top-ups
        let report_with_restock = compute_shopping_list(
            &db,
            &ShoppingListQuery {
                kit_build_ids: vec![build.id.clone()],
                include_low_stock_restock: true,
                supplier_filter: None,
            },
        )
        .await
        .unwrap();
        assert_eq!(report_with_restock.lines.len(), 3);

        let r10k = report_with_restock
            .lines
            .iter()
            .find(|l| l.component_id.as_deref() == Some(comp_shortfall_and_restock.id.as_str()))
            .unwrap();
        assert_eq!(r10k.net_shortfall, 3);
        assert_eq!(r10k.restock_top_up, 10);
        assert_eq!(r10k.recommended_order_quantity, 13);
        assert_eq!(r10k.order_quantity, 13);

        let c100n = report_with_restock
            .lines
            .iter()
            .find(|l| l.component_id.as_deref() == Some(comp_post_build_low.id.as_str()))
            .unwrap();
        assert_eq!(c100n.total_needed, 5);
        assert_eq!(c100n.available_stock, 8);
        assert_eq!(c100n.net_shortfall, 0);
        assert_eq!(c100n.restock_top_up, 3);
        assert_eq!(c100n.recommended_order_quantity, 3);
        assert_eq!(c100n.order_quantity, 3);
        assert_eq!(c100n.build_breakdown.len(), 1);

        let d4148 = report_with_restock
            .lines
            .iter()
            .find(|l| l.component_id.as_deref() == Some(comp_standalone_low.id.as_str()))
            .unwrap();
        assert_eq!(d4148.total_needed, 0);
        assert_eq!(d4148.available_stock, 2);
        assert_eq!(d4148.net_shortfall, 0);
        assert_eq!(d4148.restock_top_up, 8);
        assert_eq!(d4148.recommended_order_quantity, 8);
        assert_eq!(d4148.order_quantity, 8);
        assert!(d4148.build_breakdown.is_empty());
    }

    #[tokio::test]
    async fn applies_inline_overrides_and_exports_full_and_supplier_quick_order_csvs() {
        let db = init_memory_db().await.expect("memory db");

        let loc = create_storage_location(
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

        let r_4k7 = create_component(
            &db,
            CreateComponent {
                name: "4.7kΩ 1% Metal Film Resistor".into(),
                category: "Resistors".into(),
                subtype: "Metal Film".into(),
                package_or_mounting: "Through-Hole".into(),
                raw_spec_text: "4k7 1% 1/4W Metal Film".into(),
                canonical_value_display: None,
                resistance_ohms: None,
                capacitance_pf: None,
                inductance_uh: None,
                voltage_rating_volts: None,
                low_stock_threshold: 0,
                datasheet_url: "".into(),
                notes: "".into(),
                specs_json: None,
                initial_lot: Some(CreateInitialStockLot {
                    location_id: Some(loc.id.clone()),
                    quantity: 2,
                    supplier: "Tayda".into(),
                    supplier_part_no: "A-2214".into(),
                    lot_notes: "".into(),
                }),
            },
        )
        .await
        .unwrap();

        let build = create_kit_build(
            &db,
            CreateKitBuild {
                name: "Klon Centaur".into(),
                project_type: "Overdrive".into(),
                status: Some("Sourcing / Waiting on Parts".into()),
                location_id: None,
                description: "".into(),
            },
        )
        .await
        .unwrap();

        // Linked resistor: needs 5, available 2 -> Net Shortfall = 3
        add_bom_line(
            &db,
            CreateBomLine {
                kit_build_id: build.id.clone(),
                raw_spec_text: "4k7 1%".into(),
                category: Some("Resistors".into()),
                package_or_mounting: Some("Through-Hole".into()),
                quantity_required: 5,
                reference_designators: "R1, R2, R3, R4, R5".into(),
                included_in_kit: false,
                component_id: Some(r_4k7.id.clone()),
                substitution_note: "".into(),
                notes: "".into(),
            },
        )
        .await
        .unwrap();

        // Unlinked pot: needs 1 -> Net Shortfall = 1
        add_bom_line(
            &db,
            CreateBomLine {
                kit_build_id: build.id.clone(),
                raw_spec_text: "100kB Dual-Gang 16mm".into(),
                category: Some("Potentiometers & Encoders".into()),
                package_or_mounting: Some("16mm".into()),
                quantity_required: 1,
                reference_designators: "VR2".into(),
                included_in_kit: false,
                component_id: None,
                substitution_note: "".into(),
                notes: "".into(),
            },
        )
        .await
        .unwrap();

        let r_group_key = format!("comp:{}", r_4k7.id);
        let pot_group_key = "unlinked:Potentiometers & Encoders:100kΩ:16mm";

        // Override resistor order quantity from 3 up to 10 (pack minimum) while keeping Tayda A-2214
        upsert_shopping_list_override(
            &db,
            &r_group_key,
            Some(10),
            None,
            None,
            Some("Round up to 10-pack"),
        )
        .await
        .unwrap();

        // Override unlinked pot with Mouser supplier & part number and order quantity 2
        upsert_shopping_list_override(
            &db,
            pot_group_key,
            Some(2),
            Some("Mouser"),
            Some("313-2420F-100K"),
            Some("Buy 1 spare dual-gang pot"),
        )
        .await
        .unwrap();

        let report = compute_shopping_list(
            &db,
            &ShoppingListQuery {
                kit_build_ids: vec![build.id.clone()],
                include_low_stock_restock: false,
                supplier_filter: None,
            },
        )
        .await
        .unwrap();

        assert_eq!(report.lines.len(), 2);
        assert_eq!(
            report.available_suppliers,
            vec!["Mouser".to_string(), "Tayda".to_string()]
        );

        let r_line = report
            .lines
            .iter()
            .find(|l| l.group_key == r_group_key)
            .unwrap();
        assert_eq!(r_line.net_shortfall, 3);
        assert_eq!(r_line.recommended_order_quantity, 3);
        assert_eq!(r_line.order_quantity, 10);
        assert_eq!(r_line.supplier, "Tayda");
        assert_eq!(r_line.supplier_part_no, "A-2214");
        assert_eq!(r_line.notes, "Round up to 10-pack");
        assert!(r_line.has_override);

        let pot_line = report
            .lines
            .iter()
            .find(|l| l.group_key == pot_group_key)
            .unwrap();
        assert_eq!(pot_line.net_shortfall, 1);
        assert_eq!(pot_line.recommended_order_quantity, 1);
        assert_eq!(pot_line.order_quantity, 2);
        assert_eq!(pot_line.supplier, "Mouser");
        assert_eq!(pot_line.supplier_part_no, "313-2420F-100K");
        assert!(pot_line.has_override);

        // Full CSV export
        let full_csv = export_full_shopping_list_csv(&report);
        assert!(full_csv.starts_with("Category,Spec / Component,Package,Total Needed,Available Stock,Net Shortfall,Restock Top-Up,Order Quantity,Supplier,Supplier Part #,Kit Builds,Reference Designators\n"));
        assert!(full_csv.contains("Resistors,4.7kΩ 1% Metal Film Resistor,Through-Hole,5,2,3,0,10,Tayda,A-2214,Klon Centaur (5),\"R1, R2, R3, R4, R5\""));
        assert!(full_csv.contains("Potentiometers & Encoders,100kΩ,16mm,1,0,1,0,2,Mouser,313-2420F-100K,Klon Centaur (1),VR2"));

        // Supplier Quick-Order CSV filtered to Mouser
        let mouser_quick = export_supplier_quick_order_csv(&report, Some("Mouser"));
        assert!(mouser_quick.starts_with("Supplier Part #,Order Quantity,Customer Reference\n"));
        assert!(mouser_quick.contains("313-2420F-100K,2,Klon Centaur [VR2]"));
        assert!(!mouser_quick.contains("A-2214"));

        // Supplier Quick-Order CSV filtered to Tayda
        let tayda_quick = export_supplier_quick_order_csv(&report, Some("Tayda"));
        assert!(tayda_quick.contains("A-2214,10,\"Klon Centaur [R1, R2, R3, R4, R5]\""));
        assert!(!tayda_quick.contains("313-2420F-100K"));
    }

    #[tokio::test]
    async fn receiving_selected_shopping_list_lines_merges_existing_stock_lots_and_auto_creates_and_links_components_for_unlinked_bom_lines()
     {
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

        let r_4k7 = create_component(
            &db,
            CreateComponent {
                name: "4.7kΩ 1% Metal Film Resistor".into(),
                category: "Resistors".into(),
                subtype: "Metal Film".into(),
                package_or_mounting: "Through-Hole".into(),
                raw_spec_text: "4k7 1% 1/4W Metal Film".into(),
                canonical_value_display: None,
                resistance_ohms: None,
                capacitance_pf: None,
                inductance_uh: None,
                voltage_rating_volts: None,
                low_stock_threshold: 0,
                datasheet_url: "".into(),
                notes: "".into(),
                specs_json: None,
                initial_lot: Some(CreateInitialStockLot {
                    location_id: Some(binder.id.clone()),
                    quantity: 2,
                    supplier: "Tayda".into(),
                    supplier_part_no: "A-2214".into(),
                    lot_notes: "".into(),
                }),
            },
        )
        .await
        .unwrap();

        let build1 = create_kit_build(
            &db,
            CreateKitBuild {
                name: "Klon Centaur".into(),
                project_type: "Overdrive".into(),
                status: Some("Sourcing / Waiting on Parts".into()),
                location_id: None,
                description: "".into(),
            },
        )
        .await
        .unwrap();

        let build2 = create_kit_build(
            &db,
            CreateKitBuild {
                name: "Tube Screamer".into(),
                project_type: "Overdrive".into(),
                status: Some("Wishlist / Planned".into()),
                location_id: None,
                description: "".into(),
            },
        )
        .await
        .unwrap();

        // Linked resistor line (needs 5, available 2 -> shortfall 3)
        add_bom_line(
            &db,
            CreateBomLine {
                kit_build_id: build1.id.clone(),
                raw_spec_text: "4k7 1%".into(),
                category: Some("Resistors".into()),
                package_or_mounting: Some("Through-Hole".into()),
                quantity_required: 5,
                reference_designators: "R1, R2, R3, R4, R5".into(),
                included_in_kit: false,
                component_id: Some(r_4k7.id.clone()),
                substitution_note: "".into(),
                notes: "".into(),
            },
        )
        .await
        .unwrap();

        // Two unlinked 47nF film cap BOM lines across Build 1 and Build 2
        let unlinked_b1 = add_bom_line(
            &db,
            CreateBomLine {
                kit_build_id: build1.id.clone(),
                raw_spec_text: "0.047uF 63V Film".into(),
                category: Some("Capacitors".into()),
                package_or_mounting: Some("Through-Hole 5mm".into()),
                quantity_required: 2,
                reference_designators: "C1, C2".into(),
                included_in_kit: false,
                component_id: None,
                substitution_note: "".into(),
                notes: "".into(),
            },
        )
        .await
        .unwrap();

        let unlinked_b2 = add_bom_line(
            &db,
            CreateBomLine {
                kit_build_id: build2.id.clone(),
                raw_spec_text: "47n 63V Film".into(),
                category: Some("Capacitors".into()),
                package_or_mounting: Some("Through-Hole 5mm".into()),
                quantity_required: 3,
                reference_designators: "C5, C6, C7".into(),
                included_in_kit: false,
                component_id: None,
                substitution_note: "".into(),
                notes: "".into(),
            },
        )
        .await
        .unwrap();

        upsert_shopping_list_override(
            &db,
            "unlinked:Capacitors:47nF:Through-Hole 5mm",
            Some(10),
            Some("Mouser"),
            Some("505-MKS2.047/63/5"),
            None,
        )
        .await
        .unwrap();

        let report = compute_shopping_list(
            &db,
            &ShoppingListQuery {
                kit_build_ids: vec![build1.id.clone(), build2.id.clone()],
                include_low_stock_restock: false,
                supplier_filter: None,
            },
        )
        .await
        .unwrap();
        assert_eq!(report.lines.len(), 2);

        let lines_to_receive: Vec<ReceiveShoppingListLine> = report
            .lines
            .iter()
            .map(ReceiveShoppingListLine::from_shopping_list_line)
            .collect();

        let receive_res =
            receive_shopping_list_lines(&db, Some(&binder.id), &lines_to_receive)
                .await
                .unwrap();

        assert_eq!(receive_res.updated_lots.len(), 2);
        assert_eq!(receive_res.created_components.len(), 1);
        assert_eq!(receive_res.linked_bom_line_ids.len(), 2);

        // 1. Existing 4.7kΩ component lot at Binder A was incremented from 2 + 3 = 5 (single merged lot)
        let updated_r = get_component(&db, &r_4k7.id).await.unwrap().unwrap();
        assert_eq!(updated_r.lots.len(), 1);
        assert_eq!(updated_r.total_on_hand, 5);
        assert_eq!(updated_r.available_stock, 5);

        // 2. Unlinked 47nF spec created a new catalog Component with 10 in stock at Binder A and linked both BOM lines
        let created_cap = &receive_res.created_components[0];
        assert_eq!(created_cap.category, "Capacitors");
        assert_eq!(created_cap.canonical_value_display, "47nF");
        assert_eq!(created_cap.total_on_hand, 10);
        assert_eq!(created_cap.lots[0].supplier, "Mouser");
        assert_eq!(created_cap.lots[0].supplier_part_no, "505-MKS2.047/63/5");

        let b1_line_after = crate::builds::get_bom_line(&db, &unlinked_b1.id)
            .await
            .unwrap()
            .unwrap();
        let b2_line_after = crate::builds::get_bom_line(&db, &unlinked_b2.id)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(
            b1_line_after.component_id.as_deref(),
            Some(created_cap.id.as_str())
        );
        assert_eq!(
            b2_line_after.component_id.as_deref(),
            Some(created_cap.id.as_str())
        );

        // 3. Recomputing the shopping list now shows 0 shortfalls!
        let post_receive_report = compute_shopping_list(
            &db,
            &ShoppingListQuery {
                kit_build_ids: vec![build1.id.clone(), build2.id.clone()],
                include_low_stock_restock: false,
                supplier_filter: None,
            },
        )
        .await
        .unwrap();
        assert!(post_receive_report.lines.is_empty());
    }
}
