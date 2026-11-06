use crate::catalog::{
    Component, CreateComponent, CreateInitialStockLot, CreateStockLot, LocationKind,
    create_component_tx, get_component, list_components, upsert_or_merge_stock_lot_tx,
};
use crate::db::Db;
use crate::spec_parser::{
    Category, ParsedSpec, infer_category_from_designators, json_f64_attr, json_str_attr,
    parse_spec,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ImportMatchStatus {
    #[serde(rename = "exact_match")]
    ExactMatch,
    #[serde(rename = "near_match")]
    NearOrAmbiguousMatch,
    #[serde(rename = "no_match")]
    NoMatch,
}

impl ImportMatchStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            ImportMatchStatus::ExactMatch => "exact_match",
            ImportMatchStatus::NearOrAmbiguousMatch => "near_match",
            ImportMatchStatus::NoMatch => "no_match",
        }
    }

    pub fn display_label(&self) -> &'static str {
        match self {
            ImportMatchStatus::ExactMatch => "Exact Match",
            ImportMatchStatus::NearOrAmbiguousMatch => "Near / Ambiguous Match",
            ImportMatchStatus::NoMatch => "No Match",
        }
    }

    pub fn from_label(s: &str) -> Self {
        match s.trim().to_lowercase().as_str() {
            "exact_match" | "exact match" => ImportMatchStatus::ExactMatch,
            "near_match" | "near / ambiguous match" | "near_or_ambiguous_match" | "ambiguous" => {
                ImportMatchStatus::NearOrAmbiguousMatch
            }
            _ => ImportMatchStatus::NoMatch,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ImportChosenAction {
    #[serde(rename = "add_stock_lot")]
    AddStockLot,
    #[serde(rename = "create_new")]
    CreateNew,
    #[serde(rename = "link_component")]
    LinkComponent,
    #[serde(rename = "keep_unlinked")]
    KeepUnlinked,
    #[serde(rename = "skip")]
    Skip,
}

impl ImportChosenAction {
    pub fn as_str(&self) -> &'static str {
        match self {
            ImportChosenAction::AddStockLot => "add_stock_lot",
            ImportChosenAction::CreateNew => "create_new",
            ImportChosenAction::LinkComponent => "link_component",
            ImportChosenAction::KeepUnlinked => "keep_unlinked",
            ImportChosenAction::Skip => "skip",
        }
    }

    pub fn from_label(s: &str) -> Self {
        match s.trim().to_lowercase().as_str() {
            "add_stock_lot" | "add / increment stock lot" | "increment_lot" => {
                ImportChosenAction::AddStockLot
            }
            "link_component" | "link existing component" | "link" => {
                ImportChosenAction::LinkComponent
            }
            "keep_unlinked" | "keep unlinked" | "unlinked" => ImportChosenAction::KeepUnlinked,
            "skip" | "ignore" => ImportChosenAction::Skip,
            _ => ImportChosenAction::CreateNew,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ImportCandidate {
    pub component_id: String,
    pub name: String,
    pub category: String,
    pub package_or_mounting: String,
    pub canonical_value_display: String,
    pub raw_spec_text: String,
    pub available_stock: i64,
    pub total_on_hand: i64,
    pub match_reason: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ImportRow {
    pub id: String,
    pub batch_id: String,
    pub row_index: i64,
    pub raw_spec_text: String,
    pub category: String,
    pub subtype: String,
    pub package_or_mounting: String,
    pub quantity: i64,
    pub location_path: String,
    pub supplier: String,
    pub supplier_part_no: String,
    pub datasheet_url: String,
    pub reference_designators: String,
    pub included_in_kit: bool,
    pub notes: String,
    pub parsed_spec: ParsedSpec,
    pub match_status: ImportMatchStatus,
    pub matched_component_id: Option<String>,
    pub matched_component_name: Option<String>,
    pub candidates: Vec<ImportCandidate>,
    pub chosen_action: ImportChosenAction,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ImportBatchDetails {
    pub id: String,
    pub target_kind: String,
    pub kit_build_id: Option<String>,
    pub import_mode: String,
    pub default_location_id: Option<String>,
    pub default_supplier: String,
    pub group_designators: bool,
    pub status: String,
    pub rows: Vec<ImportRow>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct StageComponentImportInput {
    pub csv_text: String,
    pub default_location_id: Option<String>,
    pub default_supplier: Option<String>,
    pub preset_name: Option<String>,
    pub column_mapping: Option<ColumnMapping>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommitComponentImportSummary {
    pub batch_id: String,
    pub components_created: usize,
    pub stock_lots_merged_or_created: usize,
    pub rows_skipped: usize,
    pub locations_created: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum BomImportMode {
    #[default]
    #[serde(rename = "append")]
    Append,
    #[serde(rename = "replace_unallocated")]
    ReplaceUnallocated,
}

impl BomImportMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            BomImportMode::Append => "append",
            BomImportMode::ReplaceUnallocated => "replace_unallocated",
        }
    }

    pub fn display_label(&self) -> &'static str {
        match self {
            BomImportMode::Append => "Append",
            BomImportMode::ReplaceUnallocated => "Replace Unallocated Lines",
        }
    }

    pub fn from_label(s: &str) -> Self {
        match s.trim().to_lowercase().as_str() {
            "replace_unallocated" | "replace unallocated lines" | "replace" => {
                BomImportMode::ReplaceUnallocated
            }
            _ => BomImportMode::Append,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StageBomImportInput {
    pub kit_build_id: String,
    pub csv_text: String,
    pub import_mode: BomImportMode,
    pub group_designators: bool,
    pub preset_name: Option<String>,
    pub column_mapping: Option<ColumnMapping>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommitBomImportSummary {
    pub batch_id: String,
    pub kit_build_id: String,
    pub bom_lines_created: usize,
    pub bom_lines_linked: usize,
    pub unallocated_lines_replaced: usize,
    pub rows_skipped: usize,
}

fn parse_bool_flag(s: &str) -> bool {
    matches!(
        s.trim().to_lowercase().as_str(),
        "1" | "true" | "yes" | "y" | "included" | "included in kit"
    )
}

fn split_designators_clean(s: &str) -> Vec<String> {
    s.split(|c: char| c == ',' || c == ';')
        .map(str::trim)
        .filter(|d| !d.is_empty())
        .map(str::to_string)
        .collect()
}

struct StagedBomDraftRow {
    raw_spec_text: String,
    category: Category,
    package_or_mounting: String,
    quantity: i64,
    designators: Vec<String>,
    included_in_kit: bool,
    notes_list: Vec<String>,
    parsed: ParsedSpec,
}

pub async fn stage_bom_import(
    db: &Db,
    input: StageBomImportInput,
) -> anyhow::Result<ImportBatchDetails> {
    if crate::builds::get_kit_build(db, &input.kit_build_id)
        .await?
        .is_none()
    {
        anyhow::bail!("Kit Build '{}' does not exist", input.kit_build_id);
    }

    let (headers, records) = parse_tabular_text(&input.csv_text)?;
    if records.is_empty() {
        anyhow::bail!("CSV / tabular input contains no data rows");
    }

    let (_detected_sup, mapping) = resolve_preset_mapping(
        db,
        input.preset_name.as_deref(),
        &headers,
        input.column_mapping.as_ref(),
    )
    .await?;

    let mut drafts: Vec<StagedBomDraftRow> = Vec::new();
    let mut group_indices: std::collections::HashMap<String, usize> =
        std::collections::HashMap::new();

    for record in &records {
        let mut raw_spec_text = extract_cell(&headers, record, &mapping.raw_spec_text).to_string();
        let mapped_name = extract_cell(&headers, record, &mapping.name);
        if raw_spec_text.is_empty() && !mapped_name.is_empty() {
            raw_spec_text = mapped_name.to_string();
        }
        let ref_des_raw = extract_cell(&headers, record, &mapping.reference_designators);
        if raw_spec_text.is_empty() && record.len() >= 2 && !ref_des_raw.is_empty() {
            raw_spec_text = record[1].clone();
        } else if raw_spec_text.is_empty() && !record.is_empty() {
            raw_spec_text = record[0].clone();
        }
        if raw_spec_text.trim().is_empty() {
            continue;
        }

        let explicit_cat_str = extract_cell(&headers, record, &mapping.category);
        let explicit_cat = Category::from_label(explicit_cat_str);
        let des_cat = infer_category_from_designators(ref_des_raw);
        let cat_hint = explicit_cat.or(des_cat);
        let parsed = parse_spec(&raw_spec_text, cat_hint);
        let final_cat = cat_hint
            .or(parsed.inferred_category)
            .unwrap_or(Category::OtherCustom);

        let explicit_pkg = extract_cell(&headers, record, &mapping.package_or_mounting);
        let package_or_mounting = if !explicit_pkg.is_empty() {
            explicit_pkg.to_string()
        } else {
            parsed.package_or_mounting.clone().unwrap_or_default()
        };

        let des_list = split_designators_clean(ref_des_raw);
        let default_qty = des_list.len().max(1) as i64;
        let qty_str = extract_cell(&headers, record, &mapping.quantity);
        let quantity = parse_i64_qty(qty_str, default_qty).max(1);

        let inc_str = extract_cell(&headers, record, &mapping.included_in_kit);
        let included_in_kit = parse_bool_flag(inc_str);
        let note_str = extract_cell(&headers, record, &mapping.notes);

        if input.group_designators {
            let val_key = if !parsed.canonical_value_display.trim().is_empty() {
                parsed.canonical_value_display.trim().to_lowercase()
            } else {
                raw_spec_text.trim().to_lowercase()
            };
            let group_key = format!(
                "{}|{}|pkg={}|kit={}|taper={}|tol={}|w={}|v={}|diel={}|comp={}",
                final_cat.as_str(),
                val_key,
                package_or_mounting.trim().to_lowercase(),
                included_in_kit,
                parsed.taper.as_deref().unwrap_or("").to_lowercase(),
                parsed.tolerance.as_deref().unwrap_or("").to_lowercase(),
                parsed
                    .power_rating_watts
                    .map(|w| format!("{w:.4}"))
                    .unwrap_or_default(),
                parsed
                    .voltage_rating_volts
                    .map(|v| format!("{v:.3}"))
                    .unwrap_or_default(),
                parsed.dielectric.as_deref().unwrap_or("").to_lowercase(),
                parsed.composition.as_deref().unwrap_or("").to_lowercase(),
            );

            if let Some(&existing_idx) = group_indices.get(&group_key) {
                let existing = &mut drafts[existing_idx];
                existing.quantity += quantity;
                for d in des_list {
                    if !existing
                        .designators
                        .iter()
                        .any(|ed| ed.eq_ignore_ascii_case(&d))
                    {
                        existing.designators.push(d);
                    }
                }
                if !note_str.is_empty()
                    && !existing
                        .notes_list
                        .iter()
                        .any(|en| en.eq_ignore_ascii_case(note_str))
                {
                    existing.notes_list.push(note_str.to_string());
                }
                continue;
            } else {
                group_indices.insert(group_key, drafts.len());
            }
        }

        let mut notes_list = Vec::new();
        if !note_str.is_empty() {
            notes_list.push(note_str.to_string());
        }
        drafts.push(StagedBomDraftRow {
            raw_spec_text,
            category: final_cat,
            package_or_mounting,
            quantity,
            designators: des_list,
            included_in_kit,
            notes_list,
            parsed,
        });
    }

    let all_components = list_components(db).await?;
    let batch_id = format!("batch-{}", Uuid::new_v4());
    let mut conn = db.connect().await?;
    let tx = conn.transaction().await?;

    tx.execute(
        "INSERT INTO import_batches (
            id, target_kind, kit_build_id, import_mode, default_location_id,
            default_supplier, group_designators, status
        ) VALUES (?1, 'bom', ?2, ?3, NULL, '', ?4, 'staged')",
        (
            batch_id.as_str(),
            input.kit_build_id.as_str(),
            input.import_mode.as_str(),
            if input.group_designators { 1i64 } else { 0i64 },
        ),
    )
    .await?;

    for (idx, draft) in drafts.into_iter().enumerate() {
        let ref_des_joined = draft.designators.join(", ");
        let notes_joined = draft.notes_list.join("; ");

        let (match_status, matched_component_id, candidates, chosen_action) =
            if draft.included_in_kit {
                (
                    ImportMatchStatus::NoMatch,
                    None,
                    Vec::new(),
                    ImportChosenAction::KeepUnlinked,
                )
            } else {
                let (ms, mcid, _mname, cands) = classify_row_against_catalog(
                    &all_components,
                    draft.category.as_str(),
                    &draft.raw_spec_text,
                    &draft.package_or_mounting,
                    "",
                    "",
                    &draft.parsed,
                    true,
                );
                let action = match ms {
                    ImportMatchStatus::ExactMatch => ImportChosenAction::LinkComponent,
                    ImportMatchStatus::NearOrAmbiguousMatch | ImportMatchStatus::NoMatch => {
                        ImportChosenAction::KeepUnlinked
                    }
                };
                (ms, mcid, cands, action)
            };

        let row_id = format!("impr-{}", Uuid::new_v4());
        let parsed_json_str = serde_json::to_string(&draft.parsed)?;
        let candidates_json_str = serde_json::to_string(&candidates)?;

        tx.execute(
            "INSERT INTO import_rows (
                id, batch_id, row_index, raw_spec_text, category, subtype,
                package_or_mounting, quantity, location_path, supplier, supplier_part_no,
                datasheet_url, reference_designators, included_in_kit, notes,
                parsed_json, match_status, matched_component_id, candidates_json, chosen_action
            ) VALUES (?1, ?2, ?3, ?4, ?5, '', ?6, ?7, '', '', '', '', ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)",
            turso::params![
                row_id.as_str(),
                batch_id.as_str(),
                idx as i64,
                draft.raw_spec_text.as_str(),
                draft.category.as_str(),
                draft.package_or_mounting.as_str(),
                draft.quantity,
                ref_des_joined.as_str(),
                if draft.included_in_kit { 1i64 } else { 0i64 },
                notes_joined.as_str(),
                parsed_json_str.as_str(),
                match_status.as_str(),
                matched_component_id,
                candidates_json_str.as_str(),
                chosen_action.as_str(),
            ],
        )
        .await?;
    }

    tx.commit().await?;

    get_import_batch(db, &batch_id)
        .await?
        .ok_or_else(|| anyhow::anyhow!("Failed to load staged BOM import batch '{batch_id}'"))
}

pub async fn commit_bom_import_batch(
    db: &Db,
    batch_id: &str,
) -> anyhow::Result<CommitBomImportSummary> {
    let batch = get_import_batch(db, batch_id)
        .await?
        .ok_or_else(|| anyhow::anyhow!("Import batch '{batch_id}' not found"))?;

    if batch.target_kind != "bom" {
        anyhow::bail!(
            "Import batch '{batch_id}' has target_kind '{}', expected 'bom'",
            batch.target_kind
        );
    }
    if batch.status == "committed" {
        anyhow::bail!("Import batch '{batch_id}' has already been committed");
    }

    let kit_build_id = batch
        .kit_build_id
        .clone()
        .ok_or_else(|| anyhow::anyhow!("BOM import batch '{batch_id}' has no kit_build_id"))?;

    let mut conn = db.connect().await?;
    let tx = conn.transaction().await?;
    let mut unallocated_lines_replaced = 0usize;

    if BomImportMode::from_label(&batch.import_mode) == BomImportMode::ReplaceUnallocated {
        let deleted = tx
            .execute(
                "DELETE FROM bom_lines
                 WHERE kit_build_id = ?1
                   AND fulfillment_status = 'Unallocated'
                   AND included_in_kit = 0",
                (kit_build_id.as_str(),),
            )
            .await?;
        unallocated_lines_replaced = deleted as usize;
    }

    let mut sort_rows = tx
        .query(
            "SELECT COALESCE(MAX(sort_order), 0) FROM bom_lines WHERE kit_build_id = ?1",
            (kit_build_id.as_str(),),
        )
        .await?;
    let mut next_sort_order = if let Some(r) = sort_rows.next().await? {
        r.get::<i64>(0)? + 1
    } else {
        1
    };
    drop(sort_rows);

    let mut bom_lines_created = 0usize;
    let mut bom_lines_linked = 0usize;
    let mut rows_skipped = 0usize;

    for row in &batch.rows {
        if row.chosen_action == ImportChosenAction::Skip {
            rows_skipped += 1;
            continue;
        }

        let resolved_component_id = if row.included_in_kit {
            None
        } else if row.chosen_action == ImportChosenAction::CreateNew
            && row.matched_component_id.is_none()
        {
            let created_comp_id = create_component_tx(
                &tx,
                CreateComponent {
                    name: String::new(),
                    category: row.category.clone(),
                    subtype: row.subtype.clone(),
                    package_or_mounting: row.package_or_mounting.clone(),
                    raw_spec_text: row.raw_spec_text.clone(),
                    canonical_value_display: Some(row.parsed_spec.canonical_value_display.clone()),
                    resistance_ohms: row.parsed_spec.resistance_ohms,
                    capacitance_pf: row.parsed_spec.capacitance_pf,
                    inductance_uh: row.parsed_spec.inductance_uh,
                    voltage_rating_volts: row.parsed_spec.voltage_rating_volts,
                    low_stock_threshold: 0,
                    datasheet_url: String::new(),
                    notes: row.notes.clone(),
                    specs_json: Some(row.parsed_spec.specs_json.clone()),
                    initial_lot: None,
                },
            )
            .await?;
            Some(created_comp_id)
        } else if matches!(
            row.chosen_action,
            ImportChosenAction::LinkComponent | ImportChosenAction::AddStockLot
        ) {
            let cid = row.matched_component_id.as_deref().ok_or_else(|| {
                anyhow::anyhow!(
                    "BOM import row {} ('{}') is set to '{}' but has no matched_component_id",
                    row.row_index,
                    row.raw_spec_text,
                    row.chosen_action.as_str()
                )
            })?;
            let mut comp_rows = tx
                .query("SELECT id FROM components WHERE id = ?1", (cid,))
                .await?;
            if comp_rows.next().await?.is_none() {
                anyhow::bail!(
                    "BOM import row {} ('{}') references non-existent Component '{}'",
                    row.row_index,
                    row.raw_spec_text,
                    cid
                );
            }
            Some(cid.to_string())
        } else {
            None
        };

        let (fulfillment_status, quantity_allocated, included_in_kit_int) = if row.included_in_kit {
            ("Included in Kit", row.quantity.max(1), 1i64)
        } else {
            ("Unallocated", 0i64, 0i64)
        };

        let bom_id = format!("bom-{}", Uuid::new_v4());
        let specs_json_str = serde_json::to_string(&row.parsed_spec.specs_json)?;
        let canonical_display = if !row.parsed_spec.canonical_value_display.is_empty() {
            row.parsed_spec.canonical_value_display.as_str()
        } else {
            row.raw_spec_text.as_str()
        };

        tx.execute(
            "INSERT INTO bom_lines (
                id, kit_build_id, component_id, source_lot_id, raw_spec_text,
                canonical_value_display, category, package_or_mounting,
                resistance_ohms, capacitance_pf, inductance_uh, voltage_rating_volts,
                specs_json, quantity_required, quantity_allocated, fulfillment_status,
                included_in_kit, reference_designators, substitution_note, notes, sort_order
            ) VALUES (?1, ?2, ?3, NULL, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, '', ?18, ?19)",
            turso::params![
                bom_id.as_str(),
                kit_build_id.as_str(),
                resolved_component_id.clone(),
                row.raw_spec_text.as_str(),
                canonical_display,
                row.category.as_str(),
                row.package_or_mounting.as_str(),
                row.parsed_spec.resistance_ohms,
                row.parsed_spec.capacitance_pf,
                row.parsed_spec.inductance_uh,
                row.parsed_spec.voltage_rating_volts,
                specs_json_str.as_str(),
                row.quantity.max(1),
                quantity_allocated,
                fulfillment_status,
                included_in_kit_int,
                row.reference_designators.as_str(),
                row.notes.as_str(),
                next_sort_order,
            ],
        )
        .await?;

        next_sort_order += 1;
        bom_lines_created += 1;
        if resolved_component_id.is_some() {
            bom_lines_linked += 1;
        }
    }

    tx.execute(
        "UPDATE import_batches SET status = 'committed' WHERE id = ?1",
        (batch_id,),
    )
    .await?;
    tx.commit().await?;

    Ok(CommitBomImportSummary {
        batch_id: batch_id.to_string(),
        kit_build_id,
        bom_lines_created,
        bom_lines_linked,
        unallocated_lines_replaced,
        rows_skipped,
    })
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct ColumnMapping {
    pub raw_spec_text: Option<String>,
    pub name: Option<String>,
    pub category: Option<String>,
    pub subtype: Option<String>,
    pub package_or_mounting: Option<String>,
    pub quantity: Option<String>,
    pub location_path: Option<String>,
    pub supplier: Option<String>,
    pub supplier_part_no: Option<String>,
    pub datasheet_url: Option<String>,
    pub reference_designators: Option<String>,
    pub included_in_kit: Option<String>,
    pub notes: Option<String>,
}

impl ColumnMapping {
    pub fn merged_with(&self, over: &ColumnMapping) -> ColumnMapping {
        fn pick(a: &Option<String>, b: &Option<String>) -> Option<String> {
            b.as_deref()
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .map(str::to_string)
                .or_else(|| {
                    a.as_deref()
                        .map(str::trim)
                        .filter(|s| !s.is_empty())
                        .map(str::to_string)
                })
        }
        ColumnMapping {
            raw_spec_text: pick(&self.raw_spec_text, &over.raw_spec_text),
            name: pick(&self.name, &over.name),
            category: pick(&self.category, &over.category),
            subtype: pick(&self.subtype, &over.subtype),
            package_or_mounting: pick(&self.package_or_mounting, &over.package_or_mounting),
            quantity: pick(&self.quantity, &over.quantity),
            location_path: pick(&self.location_path, &over.location_path),
            supplier: pick(&self.supplier, &over.supplier),
            supplier_part_no: pick(&self.supplier_part_no, &over.supplier_part_no),
            datasheet_url: pick(&self.datasheet_url, &over.datasheet_url),
            reference_designators: pick(&self.reference_designators, &over.reference_designators),
            included_in_kit: pick(&self.included_in_kit, &over.included_in_kit),
            notes: pick(&self.notes, &over.notes),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImportPreset {
    pub id: String,
    pub name: String,
    pub target_kind: String,
    pub is_builtin: bool,
    pub default_supplier: Option<String>,
    pub mapping: ColumnMapping,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SaveImportPresetInput {
    pub name: String,
    pub target_kind: String,
    pub mapping: ColumnMapping,
}

pub fn builtin_import_presets() -> Vec<ImportPreset> {
    vec![
        ImportPreset {
            id: "preset-mouser".into(),
            name: "Mouser".into(),
            target_kind: "components".into(),
            is_builtin: true,
            default_supplier: Some("Mouser".into()),
            mapping: ColumnMapping {
                supplier_part_no: Some("Mouser Part Number".into()),
                name: Some("Mfr. Part Number".into()),
                raw_spec_text: Some("Description".into()),
                quantity: Some("Order Qty.".into()),
                datasheet_url: Some("Datasheet".into()),
                notes: Some("Customer No".into()),
                ..Default::default()
            },
        },
        ImportPreset {
            id: "preset-digikey".into(),
            name: "DigiKey".into(),
            target_kind: "components".into(),
            is_builtin: true,
            default_supplier: Some("DigiKey".into()),
            mapping: ColumnMapping {
                supplier_part_no: Some("Digi-Key Part Number".into()),
                name: Some("Manufacturer Part Number".into()),
                raw_spec_text: Some("Description".into()),
                quantity: Some("Quantity".into()),
                notes: Some("Customer Reference".into()),
                ..Default::default()
            },
        },
        ImportPreset {
            id: "preset-tayda".into(),
            name: "Tayda".into(),
            target_kind: "components".into(),
            is_builtin: true,
            default_supplier: Some("Tayda".into()),
            mapping: ColumnMapping {
                supplier_part_no: Some("Tayda SKU".into()),
                raw_spec_text: Some("Product Name".into()),
                quantity: Some("Qty".into()),
                ..Default::default()
            },
        },
        ImportPreset {
            id: "preset-lcsc".into(),
            name: "LCSC".into(),
            target_kind: "components".into(),
            is_builtin: true,
            default_supplier: Some("LCSC".into()),
            mapping: ColumnMapping {
                supplier_part_no: Some("LCSC Part Number".into()),
                name: Some("Manufacture Part Number".into()),
                package_or_mounting: Some("Package".into()),
                raw_spec_text: Some("Description".into()),
                quantity: Some("Order Qty.".into()),
                notes: Some("Customer NO.".into()),
                ..Default::default()
            },
        },
    ]
}

fn find_header_match(headers: &[String], candidates: &[&str]) -> Option<String> {
    for &cand in candidates {
        if let Some(found) = headers
            .iter()
            .find(|h| h.trim().eq_ignore_ascii_case(cand))
        {
            return Some(found.clone());
        }
    }
    None
}

pub fn detect_supplier_and_mapping_from_headers(
    headers: &[String],
) -> (Option<String>, ColumnMapping) {
    let mut detected_supplier: Option<String> = None;

    if find_header_match(headers, &["mouser part number", "mouser #", "mouser part #"]).is_some() {
        detected_supplier = Some("Mouser".into());
    } else if find_header_match(
        headers,
        &[
            "digi-key part number",
            "digikey part number",
            "digi-key part #",
        ],
    )
    .is_some()
    {
        detected_supplier = Some("DigiKey".into());
    } else if find_header_match(headers, &["tayda sku", "sku"]).is_some()
        && find_header_match(headers, &["product name", "item name"]).is_some()
    {
        detected_supplier = Some("Tayda".into());
    } else if find_header_match(headers, &["lcsc part number", "lcsc part #"]).is_some() {
        detected_supplier = Some("LCSC".into());
    }

    let supplier_part_no = find_header_match(
        headers,
        &[
            "mouser part number",
            "mouser #",
            "mouser part #",
            "digi-key part number",
            "digikey part number",
            "digi-key part #",
            "tayda sku",
            "lcsc part number",
            "lcsc part #",
            "supplier_part_no",
            "supplier part #",
            "supplier part number",
            "sku",
            "part_no",
            "part number",
        ],
    );

    let raw_spec_text = find_header_match(
        headers,
        &[
            "raw_spec_text",
            "spec",
            "specification",
            "value",
            "description",
            "product name",
            "item name",
            "part spec",
            "component",
            "part name",
            "comment",
            "mfr. part number",
            "manufacturer part number",
            "manufacture part number",
            "mpn",
        ],
    );

    let name = find_header_match(
        headers,
        &[
            "name",
            "component name",
            "mfr. part number",
            "manufacturer part number",
            "manufacture part number",
            "mpn",
        ],
    );

    let category = find_header_match(headers, &["category", "group", "type", "component_category"]);
    let subtype = find_header_match(headers, &["subtype", "sub_category", "sub-category"]);
    let package_or_mounting = find_header_match(
        headers,
        &[
            "package_or_mounting",
            "package",
            "footprint",
            "mounting",
            "case",
        ],
    );
    let quantity = find_header_match(
        headers,
        &[
            "quantity",
            "qty",
            "quantity_required",
            "order qty.",
            "order qty",
            "quantity ordered",
            "count",
        ],
    );
    let location_path = find_header_match(
        headers,
        &[
            "location_path",
            "location",
            "storage_location",
            "storage location",
            "container",
            "bin",
            "bin path",
        ],
    );
    let supplier = find_header_match(headers, &["supplier", "vendor", "distributor"]);
    let datasheet_url = find_header_match(headers, &["datasheet_url", "datasheet", "url"]);
    let reference_designators = find_header_match(
        headers,
        &[
            "reference_designators",
            "reference designators",
            "designators",
            "designator",
            "reference",
            "references",
            "ref",
            "refdes",
            "part",
            "parts",
        ],
    );
    let included_in_kit = find_header_match(
        headers,
        &["included_in_kit", "included in kit", "in_kit", "kit"],
    );
    let notes = find_header_match(
        headers,
        &[
            "notes",
            "note",
            "customer no",
            "customer no.",
            "customer reference",
            "lot_notes",
        ],
    );

    (
        detected_supplier,
        ColumnMapping {
            raw_spec_text,
            name,
            category,
            subtype,
            package_or_mounting,
            quantity,
            location_path,
            supplier,
            supplier_part_no,
            datasheet_url,
            reference_designators,
            included_in_kit,
            notes,
        },
    )
}

pub async fn save_import_preset(
    db: &Db,
    input: SaveImportPresetInput,
) -> anyhow::Result<ImportPreset> {
    let name = input.name.trim();
    if name.is_empty() {
        anyhow::bail!("Import preset name cannot be empty");
    }
    let target_kind = if input.target_kind.trim().eq_ignore_ascii_case("bom") {
        "bom"
    } else {
        "components"
    };
    let mapping_json = serde_json::to_string(&input.mapping)?;
    let conn = db.connect().await?;

    let mut existing = conn
        .query(
            "SELECT id FROM import_presets WHERE LOWER(name) = LOWER(?1)",
            (name,),
        )
        .await?;
    let existing_id_opt = if let Some(row) = existing.next().await? {
        Some(row.get::<String>(0)?)
    } else {
        None
    };
    drop(existing);

    let id = if let Some(existing_id) = existing_id_opt {
        conn.execute(
            "UPDATE import_presets SET name = ?1, target_kind = ?2, mapping_json = ?3 WHERE id = ?4",
            (name, target_kind, mapping_json.as_str(), existing_id.as_str()),
        )
        .await?;
        existing_id
    } else {
        let new_id = format!("preset-{}", Uuid::new_v4());
        conn.execute(
            "INSERT INTO import_presets (id, name, target_kind, mapping_json) VALUES (?1, ?2, ?3, ?4)",
            (new_id.as_str(), name, target_kind, mapping_json.as_str()),
        )
        .await?;
        new_id
    };

    Ok(ImportPreset {
        id,
        name: name.to_string(),
        target_kind: target_kind.to_string(),
        is_builtin: false,
        default_supplier: None,
        mapping: input.mapping,
    })
}

pub async fn list_import_presets(
    db: &Db,
    target_kind: Option<&str>,
) -> anyhow::Result<Vec<ImportPreset>> {
    let mut out: Vec<ImportPreset> = builtin_import_presets()
        .into_iter()
        .filter(|p| {
            target_kind
                .map(str::trim)
                .filter(|k| !k.is_empty())
                .is_none_or(|k| p.target_kind.eq_ignore_ascii_case(k))
        })
        .collect();

    let conn = db.connect().await?;
    let mut rows = conn
        .query(
            "SELECT id, name, target_kind, mapping_json FROM import_presets ORDER BY name ASC",
            (),
        )
        .await?;
    while let Some(row) = rows.next().await? {
        let id = row.get::<String>(0)?;
        let name = row.get::<String>(1)?;
        let tk = row.get::<String>(2)?;
        let mapping_str = row.get::<String>(3)?;
        if let Some(filter_k) = target_kind.map(str::trim).filter(|k| !k.is_empty())
            && !tk.eq_ignore_ascii_case(filter_k)
        {
            continue;
        }
        let mapping: ColumnMapping = serde_json::from_str(&mapping_str).unwrap_or_default();
        out.push(ImportPreset {
            id,
            name,
            target_kind: tk,
            is_builtin: false,
            default_supplier: None,
            mapping,
        });
    }

    Ok(out)
}

fn parse_tabular_text(text: &str) -> anyhow::Result<(Vec<String>, Vec<Vec<String>>)> {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        anyhow::bail!("CSV / tabular input cannot be empty");
    }

    let first_line = trimmed.lines().next().unwrap_or("");
    let delimiter = if first_line.contains('\t') && !first_line.contains(',') {
        b'\t'
    } else {
        b','
    };

    let mut rdr = csv::ReaderBuilder::new()
        .delimiter(delimiter)
        .flexible(true)
        .trim(csv::Trim::All)
        .from_reader(trimmed.as_bytes());

    let headers: Vec<String> = rdr
        .headers()?
        .iter()
        .map(|h| h.trim().to_string())
        .collect();

    let mut records = Vec::new();
    for result in rdr.records() {
        let rec = result?;
        let fields: Vec<String> = rec.iter().map(|f| f.trim().to_string()).collect();
        if fields.iter().all(|f| f.is_empty()) {
            continue;
        }
        records.push(fields);
    }

    Ok((headers, records))
}

fn extract_cell<'a>(
    headers: &[String],
    row: &'a [String],
    mapped_col: &Option<String>,
) -> &'a str {
    let Some(col_name) = mapped_col.as_deref().map(str::trim).filter(|s| !s.is_empty()) else {
        return "";
    };
    if let Some(idx) = headers
        .iter()
        .position(|h| h.trim().eq_ignore_ascii_case(col_name))
    {
        return row.get(idx).map(String::as_str).unwrap_or("").trim();
    }
    if let Ok(col_idx) = col_name.parse::<usize>() {
        return row.get(col_idx).map(String::as_str).unwrap_or("").trim();
    }
    ""
}

fn parse_i64_qty(s: &str, default_val: i64) -> i64 {
    let clean: String = s
        .trim()
        .chars()
        .filter(|c| c.is_ascii_digit() || *c == '-')
        .collect();
    if clean.is_empty() {
        default_val
    } else {
        clean.parse::<i64>().unwrap_or(default_val)
    }
}

fn packages_match(a: &str, b: &str) -> bool {
    let al = a.trim().to_lowercase();
    let bl = b.trim().to_lowercase();
    if al.is_empty() || bl.is_empty() {
        return false;
    }
    al == bl || al.contains(&bl) || bl.contains(&al)
}

fn classify_row_against_catalog(
    all_components: &[Component],
    category: &str,
    raw_spec_text: &str,
    package_or_mounting: &str,
    supplier: &str,
    supplier_part_no: &str,
    parsed: &ParsedSpec,
    is_bom_import: bool,
) -> (ImportMatchStatus, Option<String>, Option<String>, Vec<ImportCandidate>) {
    // 1. Check direct Supplier + Supplier Part # match on existing Stock Lots
    if !supplier.trim().is_empty() && !supplier_part_no.trim().is_empty() {
        let sup_matches: Vec<&Component> = all_components
            .iter()
            .filter(|c| {
                c.lots.iter().any(|l| {
                    l.supplier.trim().eq_ignore_ascii_case(supplier.trim())
                        && l.supplier_part_no
                            .trim()
                            .eq_ignore_ascii_case(supplier_part_no.trim())
                })
            })
            .collect();

        if sup_matches.len() == 1 {
            let comp = sup_matches[0];
            let cand = ImportCandidate {
                component_id: comp.id.clone(),
                name: comp.name.clone(),
                category: comp.category.clone(),
                package_or_mounting: comp.package_or_mounting.clone(),
                canonical_value_display: comp.canonical_value_display.clone(),
                raw_spec_text: comp.raw_spec_text.clone(),
                available_stock: comp.available_stock,
                total_on_hand: comp.total_on_hand,
                match_reason: format!(
                    "Exact supplier match ({} {})",
                    supplier.trim(),
                    supplier_part_no.trim()
                ),
            };
            return (
                ImportMatchStatus::ExactMatch,
                Some(comp.id.clone()),
                Some(comp.name.clone()),
                vec![cand],
            );
        }
    }

    // 2. Parametric value / part_code matching
    let has_numeric = parsed.resistance_ohms.is_some()
        || parsed.capacitance_pf.is_some()
        || parsed.inductance_uh.is_some();
    let row_part_code = json_str_attr(&parsed.specs_json, "part_code");
    let row_taper = json_str_attr(&parsed.specs_json, "taper");
    let row_tolerance = json_str_attr(&parsed.specs_json, "tolerance");
    let row_wattage = json_f64_attr(&parsed.specs_json, "power_rating_watts");
    let row_dielectric = json_str_attr(&parsed.specs_json, "dielectric");
    let row_composition = json_str_attr(&parsed.specs_json, "composition");

    let mut scored_candidates: Vec<(ImportCandidate, bool, i64)> = Vec::new();

    for comp in all_components {
        if category != "Other / Custom" && !category.is_empty() && comp.category != category {
            continue;
        }

        let mut value_compatible = false;
        let mut reasons = Vec::new();

        if has_numeric {
            if let Some(ohms) = parsed.resistance_ohms {
                if comp
                    .resistance_ohms
                    .is_some_and(|co| (co - ohms).abs() < 1e-3)
                {
                    if let Some(tap) = row_taper {
                        let comp_tap = json_str_attr(&comp.specs_json, "taper");
                        if comp_tap.is_some_and(|ct| ct.eq_ignore_ascii_case(tap)) {
                            value_compatible = true;
                            reasons.push(format!("{} Taper {}", comp.canonical_value_display, tap));
                        }
                    } else {
                        value_compatible = true;
                        reasons.push(comp.canonical_value_display.clone());
                    }
                }
            } else if let Some(pf) = parsed.capacitance_pf {
                if comp
                    .capacitance_pf
                    .is_some_and(|cp| (cp - pf).abs() < 1e-3)
                {
                    value_compatible = true;
                    reasons.push(comp.canonical_value_display.clone());
                }
            } else if let Some(uh) = parsed.inductance_uh
                && comp
                    .inductance_uh
                    .is_some_and(|cu| (cu - uh).abs() < 1e-3)
            {
                value_compatible = true;
                reasons.push(comp.canonical_value_display.clone());
            }
        } else {
            let comp_code = json_str_attr(&comp.specs_json, "part_code");
            if let (Some(r_code), Some(c_code)) = (row_part_code, comp_code) {
                let ru = r_code.to_uppercase();
                let cu = c_code.to_uppercase();
                if ru == cu || cu.starts_with(&ru) || ru.starts_with(&cu) {
                    value_compatible = true;
                    reasons.push(format!("Part code {c_code}"));
                }
            }
            if !value_compatible {
                let needle = if !parsed.canonical_value_display.trim().is_empty() {
                    parsed.canonical_value_display.trim().to_lowercase()
                } else {
                    raw_spec_text.trim().to_lowercase()
                };
                if !needle.is_empty() {
                    let hay = format!(
                        "{} {} {}",
                        comp.canonical_value_display, comp.name, comp.raw_spec_text
                    )
                    .to_lowercase();
                    if hay.contains(&needle) {
                        value_compatible = true;
                        reasons.push(comp.canonical_value_display.clone());
                    }
                }
            }
        }

        if !value_compatible {
            continue;
        }

        let mut score = 100i64;
        let mut all_explicit_constraints_met = true;
        let mut missing_catalog_package_or_key_attr = false;

        // Check package_or_mounting
        if !package_or_mounting.trim().is_empty() {
            if packages_match(&comp.package_or_mounting, package_or_mounting) {
                score += 15;
                reasons.push(comp.package_or_mounting.clone());
            } else {
                all_explicit_constraints_met = false;
            }
        } else if !comp.package_or_mounting.trim().is_empty() && !is_bom_import {
            missing_catalog_package_or_key_attr = true;
        }

        // Check tolerance
        let comp_tol = json_str_attr(&comp.specs_json, "tolerance");
        if let Some(tol) = row_tolerance {
            if comp_tol.is_some_and(|ct| ct.eq_ignore_ascii_case(tol)) {
                score += 15;
                reasons.push(tol.to_string());
            } else {
                all_explicit_constraints_met = false;
            }
        } else if comp_tol.is_some() && !is_bom_import && row_wattage.is_none() && row_composition.is_none() {
            missing_catalog_package_or_key_attr = true;
        }

        // Check wattage
        if let Some(w) = row_wattage {
            if json_f64_attr(&comp.specs_json, "power_rating_watts")
                .is_some_and(|cw| (cw - w).abs() < 1e-4)
            {
                score += 15;
                reasons.push(format!("{w}W"));
            } else {
                all_explicit_constraints_met = false;
            }
        }

        // Check voltage rating
        if let Some(v) = parsed.voltage_rating_volts {
            if comp
                .voltage_rating_volts
                .is_some_and(|cv| (cv - v).abs() < 1e-3)
            {
                score += 15;
                reasons.push(format!("{v}V"));
            } else {
                all_explicit_constraints_met = false;
            }
        } else if comp.voltage_rating_volts.is_some() && !is_bom_import {
            missing_catalog_package_or_key_attr = true;
        }

        // Check dielectric
        if let Some(diel) = row_dielectric {
            if json_str_attr(&comp.specs_json, "dielectric").is_some_and(|cd| {
                cd.to_lowercase().contains(&diel.to_lowercase())
                    || diel.to_lowercase().contains(&cd.to_lowercase())
            }) {
                score += 10;
                reasons.push(diel.to_string());
            } else {
                all_explicit_constraints_met = false;
            }
        }

        // Check composition
        if let Some(comp_mat) = row_composition {
            if json_str_attr(&comp.specs_json, "composition").is_some_and(|cc| {
                cc.to_lowercase().contains(&comp_mat.to_lowercase())
                    || comp_mat.to_lowercase().contains(&cc.to_lowercase())
            }) {
                score += 10;
                reasons.push(comp_mat.to_string());
            } else {
                all_explicit_constraints_met = false;
            }
        }

        let is_exact_candidate =
            all_explicit_constraints_met && !missing_catalog_package_or_key_attr;

        let match_reason = if is_exact_candidate {
            format!("Exact parametric match ({})", reasons.join(" · "))
        } else {
            format!("Near / compatible match ({})", reasons.join(" · "))
        };

        scored_candidates.push((
            ImportCandidate {
                component_id: comp.id.clone(),
                name: comp.name.clone(),
                category: comp.category.clone(),
                package_or_mounting: comp.package_or_mounting.clone(),
                canonical_value_display: comp.canonical_value_display.clone(),
                raw_spec_text: comp.raw_spec_text.clone(),
                available_stock: comp.available_stock,
                total_on_hand: comp.total_on_hand,
                match_reason,
            },
            is_exact_candidate,
            score,
        ));
    }

    scored_candidates.sort_by(|a, b| {
        b.1.cmp(&a.1)
            .then_with(|| b.2.cmp(&a.2))
            .then_with(|| b.0.available_stock.cmp(&a.0.available_stock))
            .then_with(|| a.0.name.cmp(&b.0.name))
    });

    if scored_candidates.is_empty() {
        return (ImportMatchStatus::NoMatch, None, None, Vec::new());
    }

    let exact_count = scored_candidates.iter().filter(|(_, ex, _)| *ex).count();
    let total_count = scored_candidates.len();
    let candidates: Vec<ImportCandidate> = scored_candidates.into_iter().map(|(c, _, _)| c).collect();

    if exact_count == 1 && (total_count == 1 || !is_bom_import) {
        let best = &candidates[0];
        (
            ImportMatchStatus::ExactMatch,
            Some(best.component_id.clone()),
            Some(best.name.clone()),
            candidates,
        )
    } else if is_bom_import && total_count == 1 {
        let best = &candidates[0];
        (
            ImportMatchStatus::ExactMatch,
            Some(best.component_id.clone()),
            Some(best.name.clone()),
            candidates,
        )
    } else {
        (ImportMatchStatus::NearOrAmbiguousMatch, None, None, candidates)
    }
}

async fn resolve_preset_mapping(
    db: &Db,
    preset_name: Option<&str>,
    headers: &[String],
    explicit_mapping: Option<&ColumnMapping>,
) -> anyhow::Result<(Option<String>, ColumnMapping)> {
    let (detected_sup, detected_map) = detect_supplier_and_mapping_from_headers(headers);

    let (preset_sup, preset_map) = if let Some(pname) =
        preset_name.map(str::trim).filter(|s| !s.is_empty())
    {
        let all_presets = list_import_presets(db, None).await?;
        if let Some(found) = all_presets
            .into_iter()
            .find(|p| p.name.eq_ignore_ascii_case(pname) || p.id == pname)
        {
            (found.default_supplier, found.mapping)
        } else {
            (None, ColumnMapping::default())
        }
    } else {
        (None, ColumnMapping::default())
    };

    let merged = detected_map.merged_with(&preset_map);
    let final_map = if let Some(exp) = explicit_mapping {
        merged.merged_with(exp)
    } else {
        merged
    };
    let final_sup = preset_sup.or(detected_sup);

    Ok((final_sup, final_map))
}

pub async fn stage_component_import(
    db: &Db,
    input: StageComponentImportInput,
) -> anyhow::Result<ImportBatchDetails> {
    let (headers, records) = parse_tabular_text(&input.csv_text)?;
    if records.is_empty() {
        anyhow::bail!("CSV / tabular input contains no data rows");
    }

    let (detected_supplier, mapping) = resolve_preset_mapping(
        db,
        input.preset_name.as_deref(),
        &headers,
        input.column_mapping.as_ref(),
    )
    .await?;

    let default_location_id = input
        .default_location_id
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string);

    let default_supplier = input
        .default_supplier
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .or(detected_supplier)
        .unwrap_or_default();

    let all_components = list_components(db).await?;

    let batch_id = format!("batch-{}", Uuid::new_v4());
    let mut conn = db.connect().await?;
    let tx = conn.transaction().await?;

    tx.execute(
        "INSERT INTO import_batches (
            id, target_kind, kit_build_id, import_mode, default_location_id,
            default_supplier, group_designators, status
        ) VALUES (?1, 'components', NULL, 'append', ?2, ?3, 0, 'staged')",
        (
            batch_id.as_str(),
            default_location_id.clone(),
            default_supplier.as_str(),
        ),
    )
    .await?;

    for (idx, record) in records.iter().enumerate() {
        let mut raw_spec_text = extract_cell(&headers, record, &mapping.raw_spec_text).to_string();
        let mapped_name = extract_cell(&headers, record, &mapping.name);
        if raw_spec_text.is_empty() && !mapped_name.is_empty() {
            raw_spec_text = mapped_name.to_string();
        } else if !mapped_name.is_empty()
            && !raw_spec_text
                .to_lowercase()
                .contains(&mapped_name.to_lowercase())
        {
            // Combine MPN + Description when both are mapped from supplier CSVs
            raw_spec_text = format!("{mapped_name} {raw_spec_text}");
        }
        if raw_spec_text.is_empty() && !record.is_empty() {
            raw_spec_text = record[0].clone();
        }
        if raw_spec_text.trim().is_empty() {
            continue;
        }

        let explicit_cat_str = extract_cell(&headers, record, &mapping.category);
        let ref_des = extract_cell(&headers, record, &mapping.reference_designators);
        let cat_hint = Category::from_label(explicit_cat_str)
            .or_else(|| infer_category_from_designators(ref_des));
        let parsed = parse_spec(&raw_spec_text, cat_hint);
        let final_cat = cat_hint
            .or(parsed.inferred_category)
            .unwrap_or(Category::OtherCustom);

        let subtype = extract_cell(&headers, record, &mapping.subtype);
        let explicit_pkg = extract_cell(&headers, record, &mapping.package_or_mounting);
        let package_or_mounting = if !explicit_pkg.is_empty() {
            explicit_pkg.to_string()
        } else {
            parsed.package_or_mounting.clone().unwrap_or_default()
        };

        let qty_str = extract_cell(&headers, record, &mapping.quantity);
        let quantity = parse_i64_qty(qty_str, 1).max(0);
        let location_path = extract_cell(&headers, record, &mapping.location_path);
        let row_sup = extract_cell(&headers, record, &mapping.supplier);
        let supplier = if !row_sup.is_empty() {
            row_sup.to_string()
        } else {
            default_supplier.clone()
        };
        let supplier_part_no = extract_cell(&headers, record, &mapping.supplier_part_no);
        let datasheet_url = extract_cell(&headers, record, &mapping.datasheet_url);
        let notes = extract_cell(&headers, record, &mapping.notes);

        let (match_status, matched_component_id, _matched_name, candidates) =
            classify_row_against_catalog(
                &all_components,
                final_cat.as_str(),
                &raw_spec_text,
                &package_or_mounting,
                &supplier,
                supplier_part_no,
                &parsed,
                false,
            );

        let chosen_action = match match_status {
            ImportMatchStatus::ExactMatch => ImportChosenAction::AddStockLot,
            ImportMatchStatus::NearOrAmbiguousMatch | ImportMatchStatus::NoMatch => {
                ImportChosenAction::CreateNew
            }
        };

        let row_id = format!("impr-{}", Uuid::new_v4());
        let parsed_json_str = serde_json::to_string(&parsed)?;
        let candidates_json_str = serde_json::to_string(&candidates)?;

        tx.execute(
            "INSERT INTO import_rows (
                id, batch_id, row_index, raw_spec_text, category, subtype,
                package_or_mounting, quantity, location_path, supplier, supplier_part_no,
                datasheet_url, reference_designators, included_in_kit, notes,
                parsed_json, match_status, matched_component_id, candidates_json, chosen_action
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, 0, ?14, ?15, ?16, ?17, ?18, ?19)",
            turso::params![
                row_id.as_str(),
                batch_id.as_str(),
                idx as i64,
                raw_spec_text.as_str(),
                final_cat.as_str(),
                subtype,
                package_or_mounting.as_str(),
                quantity,
                location_path,
                supplier.as_str(),
                supplier_part_no,
                datasheet_url,
                ref_des,
                notes,
                parsed_json_str.as_str(),
                match_status.as_str(),
                matched_component_id,
                candidates_json_str.as_str(),
                chosen_action.as_str(),
            ],
        )
        .await?;
    }

    tx.commit().await?;

    get_import_batch(db, &batch_id)
        .await?
        .ok_or_else(|| anyhow::anyhow!("Failed to load staged import batch '{batch_id}'"))
}

async fn map_import_row(db: &Db, row: &turso::Row) -> anyhow::Result<ImportRow> {
    let id = row.get::<String>(0)?;
    let batch_id = row.get::<String>(1)?;
    let row_index = row.get::<i64>(2)?;
    let raw_spec_text = row.get::<String>(3)?;
    let category = row.get::<String>(4)?;
    let subtype = row.get::<String>(5)?;
    let package_or_mounting = row.get::<String>(6)?;
    let quantity = row.get::<i64>(7)?;
    let location_path = row.get::<String>(8)?;
    let supplier = row.get::<String>(9)?;
    let supplier_part_no = row.get::<String>(10)?;
    let datasheet_url = row.get::<String>(11)?;
    let reference_designators = row.get::<String>(12)?;
    let included_in_kit_int = row.get::<i64>(13)?;
    let notes = row.get::<String>(14)?;
    let parsed_json_str = row.get::<String>(15)?;
    let match_status_str = row.get::<String>(16)?;
    let matched_component_id = row.get::<Option<String>>(17)?;
    let candidates_json_str = row.get::<String>(18)?;
    let chosen_action_str = row.get::<String>(19)?;

    let parsed_spec: ParsedSpec = serde_json::from_str(&parsed_json_str)
        .unwrap_or_else(|_| parse_spec(&raw_spec_text, Category::from_label(&category)));
    let candidates: Vec<ImportCandidate> =
        serde_json::from_str(&candidates_json_str).unwrap_or_default();

    let matched_component_name = if let Some(ref cid) = matched_component_id {
        if let Some(cand) = candidates.iter().find(|c| &c.component_id == cid) {
            Some(cand.name.clone())
        } else {
            get_component(db, cid).await?.map(|c| c.name)
        }
    } else {
        None
    };

    Ok(ImportRow {
        id,
        batch_id,
        row_index,
        raw_spec_text,
        category,
        subtype,
        package_or_mounting,
        quantity,
        location_path,
        supplier,
        supplier_part_no,
        datasheet_url,
        reference_designators,
        included_in_kit: included_in_kit_int != 0,
        notes,
        parsed_spec,
        match_status: ImportMatchStatus::from_label(&match_status_str),
        matched_component_id,
        matched_component_name,
        candidates,
        chosen_action: ImportChosenAction::from_label(&chosen_action_str),
    })
}

pub async fn get_import_row(db: &Db, row_id: &str) -> anyhow::Result<Option<ImportRow>> {
    let conn = db.connect().await?;
    let mut rows = conn
        .query(
            "SELECT
                id, batch_id, row_index, raw_spec_text, category, subtype,
                package_or_mounting, quantity, location_path, supplier, supplier_part_no,
                datasheet_url, reference_designators, included_in_kit, notes,
                parsed_json, match_status, matched_component_id, candidates_json, chosen_action
             FROM import_rows
             WHERE id = ?1",
            (row_id,),
        )
        .await?;
    if let Some(row) = rows.next().await? {
        Ok(Some(map_import_row(db, &row).await?))
    } else {
        Ok(None)
    }
}

pub async fn get_import_batch(
    db: &Db,
    batch_id: &str,
) -> anyhow::Result<Option<ImportBatchDetails>> {
    let conn = db.connect().await?;
    let mut b_rows = conn
        .query(
            "SELECT id, target_kind, kit_build_id, import_mode, default_location_id, default_supplier, group_designators, status
             FROM import_batches
             WHERE id = ?1",
            (batch_id,),
        )
        .await?;
    let Some(b_row) = b_rows.next().await? else {
        return Ok(None);
    };

    let id = b_row.get::<String>(0)?;
    let target_kind = b_row.get::<String>(1)?;
    let kit_build_id = b_row.get::<Option<String>>(2)?;
    let import_mode = b_row.get::<String>(3)?;
    let default_location_id = b_row.get::<Option<String>>(4)?;
    let default_supplier = b_row.get::<String>(5)?;
    let group_designators = b_row.get::<i64>(6)? != 0;
    let status = b_row.get::<String>(7)?;
    drop(b_rows);

    let mut r_rows = conn
        .query(
            "SELECT
                id, batch_id, row_index, raw_spec_text, category, subtype,
                package_or_mounting, quantity, location_path, supplier, supplier_part_no,
                datasheet_url, reference_designators, included_in_kit, notes,
                parsed_json, match_status, matched_component_id, candidates_json, chosen_action
             FROM import_rows
             WHERE batch_id = ?1
             ORDER BY row_index ASC",
            (batch_id,),
        )
        .await?;
    let mut rows = Vec::new();
    while let Some(r_row) = r_rows.next().await? {
        rows.push(map_import_row(db, &r_row).await?);
    }

    Ok(Some(ImportBatchDetails {
        id,
        target_kind,
        kit_build_id,
        import_mode,
        default_location_id,
        default_supplier,
        group_designators,
        status,
        rows,
    }))
}

pub async fn update_import_row_resolution(
    db: &Db,
    row_id: &str,
    chosen_action: ImportChosenAction,
    matched_component_id: Option<&str>,
) -> anyhow::Result<ImportRow> {
    let existing = get_import_row(db, row_id)
        .await?
        .ok_or_else(|| anyhow::anyhow!("Import row '{row_id}' not found"))?;

    let normalized_cid = matched_component_id
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string);

    let final_cid = match chosen_action {
        ImportChosenAction::AddStockLot | ImportChosenAction::LinkComponent => {
            let cid = normalized_cid
                .or(existing.matched_component_id.clone())
                .ok_or_else(|| {
                    anyhow::anyhow!(
                        "Selecting '{}' requires a target catalog Component ID",
                        chosen_action.as_str()
                    )
                })?;
            if get_component(db, &cid).await?.is_none() {
                anyhow::bail!("Target catalog Component '{cid}' does not exist");
            }
            Some(cid)
        }
        ImportChosenAction::CreateNew
        | ImportChosenAction::KeepUnlinked
        | ImportChosenAction::Skip => None,
    };

    let conn = db.connect().await?;
    conn.execute(
        "UPDATE import_rows SET chosen_action = ?1, matched_component_id = ?2 WHERE id = ?3",
        (chosen_action.as_str(), final_cid, row_id),
    )
    .await?;

    get_import_row(db, row_id)
        .await?
        .ok_or_else(|| anyhow::anyhow!("Import row '{row_id}' not found after update"))
}

fn infer_location_kind(segment: &str) -> LocationKind {
    let lower = segment.to_lowercase();
    if lower.contains("crate") {
        LocationKind::Crate
    } else if lower.contains("box") {
        LocationKind::Box
    } else if lower.contains("binder") || lower.contains("page") {
        LocationKind::Binder
    } else if lower.contains("drawer") || lower.contains("cabinet") {
        LocationKind::Drawer
    } else if lower.contains("bag") || lower.contains("pouch") {
        LocationKind::Bag
    } else {
        LocationKind::Other
    }
}

async fn list_locations_in_tx(
    tx: &turso::transaction::Transaction<'_>,
) -> anyhow::Result<Vec<(String, String, Option<String>, String)>> {
    let mut rows = tx
        .query(
            "WITH RECURSIVE location_tree(id, name, parent_id, breadcrumb_path) AS (
                SELECT id, name, parent_id, name AS breadcrumb_path
                FROM storage_locations
                WHERE parent_id IS NULL
                UNION ALL
                SELECT c.id, c.name, c.parent_id, p.breadcrumb_path || ' > ' || c.name
                FROM storage_locations c
                JOIN location_tree p ON c.parent_id = p.id
            )
            SELECT id, name, parent_id, breadcrumb_path FROM location_tree",
            (),
        )
        .await?;
    let mut out = Vec::new();
    while let Some(r) = rows.next().await? {
        out.push((
            r.get::<String>(0)?,
            r.get::<String>(1)?,
            r.get::<Option<String>>(2)?,
            r.get::<String>(3)?,
        ));
    }
    Ok(out)
}

async fn resolve_or_create_location_path_tx(
    tx: &turso::transaction::Transaction<'_>,
    path_str: &str,
) -> anyhow::Result<(Option<String>, usize)> {
    let trimmed = path_str.trim();
    if trimmed.is_empty() {
        return Ok((None, 0));
    }

    let existing_locations = list_locations_in_tx(tx).await?;
    // First check exact match on id or full breadcrumb_path
    if let Some((id, _, _, _)) = existing_locations
        .iter()
        .find(|(id, _, _, breadcrumb)| id == trimmed || breadcrumb.eq_ignore_ascii_case(trimmed))
    {
        return Ok((Some(id.clone()), 0));
    }

    let segments: Vec<&str> = if trimmed.contains('>') {
        trimmed
            .split('>')
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .collect()
    } else {
        vec![trimmed]
    };

    if segments.len() == 1 {
        if let Some((id, _, _, _)) = existing_locations
            .iter()
            .find(|(_, name, _, _)| name.eq_ignore_ascii_case(segments[0]))
        {
            return Ok((Some(id.clone()), 0));
        }
    }

    let mut current_parent_id: Option<String> = None;
    let mut created_count = 0usize;

    for seg in segments {
        let all_now = list_locations_in_tx(tx).await?;
        if let Some((existing_id, _, _, _)) = all_now
            .iter()
            .find(|(_, name, parent_id, _)| {
                *parent_id == current_parent_id && name.eq_ignore_ascii_case(seg)
            })
        {
            current_parent_id = Some(existing_id.clone());
        } else {
            let kind = infer_location_kind(seg);
            let loc_id = format!("loc-{}", Uuid::new_v4());
            tx.execute(
                "INSERT INTO storage_locations (id, name, kind, parent_id, notes)
                 VALUES (?1, ?2, ?3, ?4, '')",
                (
                    loc_id.as_str(),
                    seg,
                    kind.as_str(),
                    current_parent_id.clone(),
                ),
            )
            .await?;
            created_count += 1;
            current_parent_id = Some(loc_id);
        }
    }

    Ok((current_parent_id, created_count))
}

pub async fn commit_component_import_batch(
    db: &Db,
    batch_id: &str,
) -> anyhow::Result<CommitComponentImportSummary> {
    let batch = get_import_batch(db, batch_id)
        .await?
        .ok_or_else(|| anyhow::anyhow!("Import batch '{batch_id}' not found"))?;

    if batch.target_kind != "components" {
        anyhow::bail!(
            "Import batch '{batch_id}' has target_kind '{}', expected 'components'",
            batch.target_kind
        );
    }
    if batch.status == "committed" {
        anyhow::bail!("Import batch '{batch_id}' has already been committed");
    }

    let mut components_created = 0usize;
    let mut stock_lots_merged_or_created = 0usize;
    let mut rows_skipped = 0usize;
    let mut locations_created = 0usize;

    let mut conn = db.connect().await?;
    let tx = conn.transaction().await?;

    for row in &batch.rows {
        if row.chosen_action == ImportChosenAction::Skip {
            rows_skipped += 1;
            continue;
        }

        let (resolved_loc_id, new_locs) = if !row.location_path.trim().is_empty() {
            resolve_or_create_location_path_tx(&tx, &row.location_path).await?
        } else {
            (batch.default_location_id.clone(), 0)
        };
        locations_created += new_locs;

        let effective_supplier = if !row.supplier.trim().is_empty() {
            row.supplier.trim().to_string()
        } else {
            batch.default_supplier.trim().to_string()
        };

        match row.chosen_action {
            ImportChosenAction::AddStockLot | ImportChosenAction::LinkComponent => {
                let target_cid = row.matched_component_id.as_deref().ok_or_else(|| {
                    anyhow::anyhow!(
                        "Row {} ('{}') is set to add stock lot but has no matched_component_id",
                        row.row_index,
                        row.raw_spec_text
                    )
                })?;
                upsert_or_merge_stock_lot_tx(
                    &tx,
                    CreateStockLot {
                        component_id: target_cid.to_string(),
                        location_id: resolved_loc_id,
                        quantity: row.quantity.max(0),
                        supplier: effective_supplier,
                        supplier_part_no: row.supplier_part_no.clone(),
                        lot_notes: row.notes.clone(),
                    },
                )
                .await?;
                stock_lots_merged_or_created += 1;
            }
            ImportChosenAction::CreateNew | ImportChosenAction::KeepUnlinked => {
                let initial_lot = if row.quantity > 0
                    || resolved_loc_id.is_some()
                    || !effective_supplier.is_empty()
                    || !row.supplier_part_no.trim().is_empty()
                {
                    Some(CreateInitialStockLot {
                        location_id: resolved_loc_id,
                        quantity: row.quantity.max(0),
                        supplier: effective_supplier,
                        supplier_part_no: row.supplier_part_no.clone(),
                        lot_notes: row.notes.clone(),
                    })
                } else {
                    None
                };

                let has_lot = initial_lot.is_some();
                let created_comp_id = create_component_tx(
                    &tx,
                    CreateComponent {
                        name: String::new(),
                        category: row.category.clone(),
                        subtype: row.subtype.clone(),
                        package_or_mounting: row.package_or_mounting.clone(),
                        raw_spec_text: row.raw_spec_text.clone(),
                        canonical_value_display: Some(
                            row.parsed_spec.canonical_value_display.clone(),
                        ),
                        resistance_ohms: row.parsed_spec.resistance_ohms,
                        capacitance_pf: row.parsed_spec.capacitance_pf,
                        inductance_uh: row.parsed_spec.inductance_uh,
                        voltage_rating_volts: row.parsed_spec.voltage_rating_volts,
                        low_stock_threshold: 0,
                        datasheet_url: row.datasheet_url.clone(),
                        notes: row.notes.clone(),
                        specs_json: Some(row.parsed_spec.specs_json.clone()),
                        initial_lot,
                    },
                )
                .await?;

                tx.execute(
                    "UPDATE import_rows SET matched_component_id = ?1 WHERE id = ?2",
                    (created_comp_id.as_str(), row.id.as_str()),
                )
                .await?;

                components_created += 1;
                if has_lot {
                    stock_lots_merged_or_created += 1;
                }
            }
            ImportChosenAction::Skip => {
                rows_skipped += 1;
            }
        }
    }

    tx.execute(
        "UPDATE import_batches SET status = 'committed' WHERE id = ?1",
        (batch_id,),
    )
    .await?;
    tx.commit().await?;

    Ok(CommitComponentImportSummary {
        batch_id: batch_id.to_string(),
        components_created,
        stock_lots_merged_or_created,
        rows_skipped,
        locations_created,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalog::{
        CreateStorageLocation, create_component, create_storage_location, list_storage_locations,
    };
    use crate::db::init_memory_db;

    #[tokio::test]
    async fn detects_supplier_presets_from_headers_and_saves_custom_mapping_presets() {
        let db = init_memory_db().await.expect("memory db");

        // 1. Built-in presets include Mouser, DigiKey, Tayda, and LCSC
        let builtins = builtin_import_presets();
        for supplier in ["Mouser", "DigiKey", "Tayda", "LCSC"] {
            assert!(
                builtins.iter().any(|p| p.name == supplier && p.is_builtin),
                "missing builtin preset {supplier}"
            );
        }

        // 2. Auto-detects Mouser, DigiKey, Tayda, and LCSC from export headers
        let (mouser_sup, mouser_map) = detect_supplier_and_mapping_from_headers(&[
            "Mouser Part Number".into(),
            "Mfr. Part Number".into(),
            "Description".into(),
            "Order Qty.".into(),
            "Customer No".into(),
        ]);
        assert_eq!(mouser_sup.as_deref(), Some("Mouser"));
        assert_eq!(
            mouser_map.supplier_part_no.as_deref(),
            Some("Mouser Part Number")
        );
        assert_eq!(mouser_map.raw_spec_text.as_deref(), Some("Description"));
        assert_eq!(mouser_map.quantity.as_deref(), Some("Order Qty."));

        let (dk_sup, dk_map) = detect_supplier_and_mapping_from_headers(&[
            "Digi-Key Part Number".into(),
            "Manufacturer Part Number".into(),
            "Description".into(),
            "Quantity".into(),
            "Customer Reference".into(),
        ]);
        assert_eq!(dk_sup.as_deref(), Some("DigiKey"));
        assert_eq!(
            dk_map.supplier_part_no.as_deref(),
            Some("Digi-Key Part Number")
        );
        assert_eq!(dk_map.quantity.as_deref(), Some("Quantity"));

        let (tayda_sup, tayda_map) = detect_supplier_and_mapping_from_headers(&[
            "Tayda SKU".into(),
            "Product Name".into(),
            "Qty".into(),
        ]);
        assert_eq!(tayda_sup.as_deref(), Some("Tayda"));
        assert_eq!(tayda_map.supplier_part_no.as_deref(), Some("Tayda SKU"));
        assert_eq!(tayda_map.raw_spec_text.as_deref(), Some("Product Name"));
        assert_eq!(tayda_map.quantity.as_deref(), Some("Qty"));

        let (lcsc_sup, lcsc_map) = detect_supplier_and_mapping_from_headers(&[
            "LCSC Part Number".into(),
            "Manufacture Part Number".into(),
            "Package".into(),
            "Description".into(),
            "Order Qty.".into(),
        ]);
        assert_eq!(lcsc_sup.as_deref(), Some("LCSC"));
        assert_eq!(
            lcsc_map.supplier_part_no.as_deref(),
            Some("LCSC Part Number")
        );
        assert_eq!(lcsc_map.package_or_mounting.as_deref(), Some("Package"));

        // 3. Saves and lists a user-defined custom mapping preset
        let saved = save_import_preset(
            &db,
            SaveImportPresetInput {
                name: "My Bench Spreadsheet".into(),
                target_kind: "components".into(),
                mapping: ColumnMapping {
                    raw_spec_text: Some("Part Spec".into()),
                    category: Some("Group".into()),
                    quantity: Some("Count".into()),
                    location_path: Some("Bin Path".into()),
                    ..Default::default()
                },
            },
        )
        .await
        .expect("save preset");
        assert_eq!(saved.name, "My Bench Spreadsheet");
        assert!(!saved.is_builtin);
        assert_eq!(saved.mapping.raw_spec_text.as_deref(), Some("Part Spec"));
        assert_eq!(saved.mapping.location_path.as_deref(), Some("Bin Path"));

        let all_presets = list_import_presets(&db, Some("components"))
            .await
            .expect("list presets");
        assert!(all_presets.iter().any(|p| p.name == "Mouser"));
        assert!(all_presets.iter().any(|p| p.name == "My Bench Spreadsheet"));
    }

    #[tokio::test]
    async fn stages_component_import_classifies_exact_near_and_no_matches_applies_overrides_and_commits_atomically()
     {
        let db = init_memory_db().await.expect("memory db");

        let binder_a = create_storage_location(
            &db,
            CreateStorageLocation {
                name: "Binder A".into(),
                kind: "Binder".into(),
                parent_id: None,
                notes: "".into(),
            },
        )
        .await
        .expect("create binder a");

        // Seed existing catalog components:
        // 1) 4.7kΩ 1% 1/4W Metal Film Through-Hole Resistor (Tayda A-2214, qty 40 in Binder A)
        let r_4k7 = create_component(
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
                low_stock_threshold: 10,
                datasheet_url: "".into(),
                notes: "".into(),
                specs_json: None,
                initial_lot: Some(CreateInitialStockLot {
                    location_id: Some(binder_a.id.clone()),
                    quantity: 40,
                    supplier: "Tayda".into(),
                    supplier_part_no: "A-2214".into(),
                    lot_notes: "Initial stock".into(),
                }),
            },
        )
        .await
        .expect("create r_4k7");

        // 2) Two 47nF film capacitors (63V and 100V) so an unspecific "0.047uF Film" row is Near / Ambiguous Match
        let c_47n_63v = create_component(
            &db,
            CreateComponent {
                name: "47nF 63V WIMA Film Cap".into(),
                category: "Capacitors".into(),
                subtype: "Film".into(),
                package_or_mounting: "Through-Hole 5mm".into(),
                raw_spec_text: "0.047uF 63V 5% Box Film 5mm".into(),
                canonical_value_display: None,
                resistance_ohms: None,
                capacitance_pf: None,
                inductance_uh: None,
                voltage_rating_volts: None,
                low_stock_threshold: 5,
                datasheet_url: "".into(),
                notes: "".into(),
                specs_json: None,
                initial_lot: Some(CreateInitialStockLot {
                    location_id: Some(binder_a.id.clone()),
                    quantity: 8,
                    supplier: "Mouser".into(),
                    supplier_part_no: "505-MKS2.047/63/5".into(),
                    lot_notes: "".into(),
                }),
            },
        )
        .await
        .expect("create c_47n_63v");

        let c_47n_100v = create_component(
            &db,
            CreateComponent {
                name: "47nF 100V Kemet Film Cap".into(),
                category: "Capacitors".into(),
                subtype: "Film".into(),
                package_or_mounting: "Through-Hole 5mm".into(),
                raw_spec_text: "47n 100V 5% Box Film 5mm".into(),
                canonical_value_display: None,
                resistance_ohms: None,
                capacitance_pf: None,
                inductance_uh: None,
                voltage_rating_volts: None,
                low_stock_threshold: 5,
                datasheet_url: "".into(),
                notes: "".into(),
                specs_json: None,
                initial_lot: Some(CreateInitialStockLot {
                    location_id: Some(binder_a.id.clone()),
                    quantity: 4,
                    supplier: "DigiKey".into(),
                    supplier_part_no: "KEM-47N-100V".into(),
                    lot_notes: "".into(),
                }),
            },
        )
        .await
        .expect("create c_47n_100v");

        // Stage a 3-row CSV import:
        // Row 0: Exact Match via supplier + supplier_part_no (Tayda A-2214, qty 25, default location Binder A)
        // Row 1: Near / Ambiguous Match ("0.047uF Box Film", Through-Hole 5mm, qty 10 — matches both 63V and 100V 47nF caps)
        // Row 2: No Match ("PT2399 Echo Processor DIP-16", qty 5, nested location_path "Crate 1 > Delay ICs Bag")
        let csv_data = "\
Supplier,Supplier Part #,Spec,Category,Package,Qty,Location,Notes
Tayda,A-2214,4.7k 1/4W Metal Film,Resistors,Through-Hole,25,,Restock order
Mouser,,0.047uF Box Film,Capacitors,Through-Hole 5mm,10,Binder A,Ambiguous voltage
Tayda,A-1199,PT2399 Echo Processor DIP-16,Integrated Circuits (ICs),DIP-16,5,Crate 1 > Delay ICs Bag,New delay IC
";

        let batch = stage_component_import(
            &db,
            StageComponentImportInput {
                csv_text: csv_data.into(),
                default_location_id: Some(binder_a.id.clone()),
                default_supplier: Some("Tayda".into()),
                preset_name: None,
                column_mapping: None,
            },
        )
        .await
        .expect("stage component import");

        assert_eq!(batch.status, "staged");
        assert_eq!(batch.rows.len(), 3);

        // Verify nothing was written to components or stock_lots yet during staging
        assert_eq!(list_components(&db).await.unwrap().len(), 3);
        assert_eq!(
            get_component(&db, &r_4k7.id)
                .await
                .unwrap()
                .unwrap()
                .total_on_hand,
            40
        );

        // Row 0: Exact Match -> defaults to AddStockLot on r_4k7
        let row0 = &batch.rows[0];
        assert_eq!(row0.match_status, ImportMatchStatus::ExactMatch);
        assert_eq!(row0.matched_component_id.as_deref(), Some(r_4k7.id.as_str()));
        assert_eq!(row0.chosen_action, ImportChosenAction::AddStockLot);

        // Row 1: Near / Ambiguous Match -> lists both c_47n_63v and c_47n_100v as candidates
        let row1 = &batch.rows[1];
        assert_eq!(row1.match_status, ImportMatchStatus::NearOrAmbiguousMatch);
        assert_eq!(row1.candidates.len(), 2);
        assert!(row1.candidates.iter().any(|c| c.component_id == c_47n_63v.id));
        assert!(row1.candidates.iter().any(|c| c.component_id == c_47n_100v.id));

        // Row 2: No Match -> defaults to CreateNew
        let row2 = &batch.rows[2];
        assert_eq!(row2.match_status, ImportMatchStatus::NoMatch);
        assert_eq!(row2.chosen_action, ImportChosenAction::CreateNew);
        assert_eq!(row2.location_path, "Crate 1 > Delay ICs Bag");

        // User resolves Row 1 (Near Match) to AddStockLot on c_47n_63v
        let updated_row1 = update_import_row_resolution(
            &db,
            &row1.id,
            ImportChosenAction::AddStockLot,
            Some(&c_47n_63v.id),
        )
        .await
        .expect("resolve row1");
        assert_eq!(updated_row1.chosen_action, ImportChosenAction::AddStockLot);
        assert_eq!(
            updated_row1.matched_component_id.as_deref(),
            Some(c_47n_63v.id.as_str())
        );

        // Commit the staged batch atomically
        let summary = commit_component_import_batch(&db, &batch.id)
            .await
            .expect("commit component batch");
        assert_eq!(summary.components_created, 1);
        assert_eq!(summary.stock_lots_merged_or_created, 3);

        // Verify Row 0 incremented r_4k7 in Binder A from 40 -> 65
        let r_4k7_after = get_component(&db, &r_4k7.id).await.unwrap().unwrap();
        assert_eq!(r_4k7_after.total_on_hand, 65);
        assert_eq!(r_4k7_after.lots.len(), 1);

        // Verify Row 1 incremented c_47n_63v in Binder A from 8 -> 18
        let c_63_after = get_component(&db, &c_47n_63v.id).await.unwrap().unwrap();
        assert_eq!(c_63_after.total_on_hand, 18);

        // Verify Row 2 created PT2399 component AND auto-created nested "Crate 1 > Delay ICs Bag"
        let all_comps = list_components(&db).await.unwrap();
        assert_eq!(all_comps.len(), 4);
        let pt2399 = all_comps
            .iter()
            .find(|c| c.raw_spec_text == "PT2399 Echo Processor DIP-16")
            .expect("created PT2399");
        assert_eq!(pt2399.category, "Integrated Circuits (ICs)");
        assert_eq!(pt2399.package_or_mounting, "DIP-16");
        assert_eq!(pt2399.total_on_hand, 5);
        assert_eq!(pt2399.lots.len(), 1);
        assert_eq!(
            pt2399.lots[0].location_breadcrumb,
            "Crate 1 > Delay ICs Bag"
        );
        assert_eq!(pt2399.lots[0].supplier, "Tayda");
        assert_eq!(pt2399.lots[0].supplier_part_no, "A-1199");

        let all_locs = list_storage_locations(&db).await.unwrap();
        assert!(
            all_locs
                .iter()
                .any(|l| l.breadcrumb_path == "Crate 1 > Delay ICs Bag")
        );
    }

    #[tokio::test]
    async fn stages_bom_import_infers_categories_from_designators_groups_identical_specs_and_commits_append_vs_replace_unallocated()
     {
        use crate::builds::{
            CreateBomLine, CreateKitBuild, add_bom_line, create_kit_build,
            list_bom_lines_for_build, transition_bom_line_status,
        };

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

        // Seed catalog components:
        // - 4.7kΩ 1% 1/4W Metal Film (single unambiguous match for 4k7 / 4.7k)
        let r_4k7 = create_component(
            &db,
            CreateComponent {
                name: "4.7kΩ 1% 1/4W Metal Film".into(),
                category: "Resistors".into(),
                subtype: "Fixed".into(),
                package_or_mounting: "Through-Hole".into(),
                raw_spec_text: "4k7 1% 1/4W Metal Film".into(),
                canonical_value_display: None,
                resistance_ohms: None,
                capacitance_pf: None,
                inductance_uh: None,
                voltage_rating_volts: None,
                low_stock_threshold: 5,
                datasheet_url: "".into(),
                notes: "".into(),
                specs_json: None,
                initial_lot: Some(CreateInitialStockLot {
                    location_id: Some(loc.id.clone()),
                    quantity: 30,
                    supplier: "Tayda".into(),
                    supplier_part_no: "A-2214".into(),
                    lot_notes: "".into(),
                }),
            },
        )
        .await
        .unwrap();

        // - TL072IP DIP-8 (single unambiguous match for TL072)
        let ic_tl072 = create_component(
            &db,
            CreateComponent {
                name: "TL072IP Dual JFET Op-Amp".into(),
                category: "Integrated Circuits (ICs)".into(),
                subtype: "Op-Amp".into(),
                package_or_mounting: "DIP-8".into(),
                raw_spec_text: "TL072IP DIP-8".into(),
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
                    location_id: Some(loc.id.clone()),
                    quantity: 5,
                    supplier: "Mouser".into(),
                    supplier_part_no: "595-TL072IP".into(),
                    lot_notes: "".into(),
                }),
            },
        )
        .await
        .unwrap();

        let build = create_kit_build(
            &db,
            CreateKitBuild {
                name: "Tube Screamer TS808".into(),
                project_type: "Overdrive".into(),
                status: Some("Sourcing / Waiting on Parts".into()),
                location_id: None,
                description: "".into(),
            },
        )
        .await
        .unwrap();

        // Add two existing lines before import:
        // 1) A Reserved line (must be preserved even under ReplaceUnallocated)
        let reserved_line = add_bom_line(
            &db,
            CreateBomLine {
                kit_build_id: build.id.clone(),
                raw_spec_text: "TL072IP DIP-8".into(),
                category: Some("Integrated Circuits (ICs)".into()),
                package_or_mounting: Some("DIP-8".into()),
                quantity_required: 1,
                reference_designators: "U1".into(),
                included_in_kit: false,
                component_id: Some(ic_tl072.id.clone()),
                substitution_note: "".into(),
                notes: "Pre-reserved op-amp".into(),
            },
        )
        .await
        .unwrap();
        transition_bom_line_status(&db, &reserved_line.id, "Reserved", None, None)
            .await
            .unwrap();

        // 2) An old Unallocated placeholder line (should be replaced when ReplaceUnallocated is used)
        let old_unalloc = add_bom_line(
            &db,
            CreateBomLine {
                kit_build_id: build.id.clone(),
                raw_spec_text: "Old Placeholder 999k".into(),
                category: Some("Resistors".into()),
                package_or_mounting: None,
                quantity_required: 1,
                reference_designators: "R99".into(),
                included_in_kit: false,
                component_id: None,
                substitution_note: "".into(),
                notes: "".into(),
            },
        )
        .await
        .unwrap();

        // KiCad-style per-placement BOM with NO Category column and separate rows for R1, R2, R5:
        let bom_csv = "\
Reference,Value,Footprint,Qty,Notes
R1,4k7,Through-Hole,1,Input bias
R2,4.7k,Through-Hole,1,Feedback
R5,4700R,Through-Hole,1,Output
C1,0.047uF,Through-Hole 5mm,1,Coupling
D1,1N4148,DO-35 Axial,1,Clipping
Q1,2N3904,TO-92,1,Buffer
U2,TL072,DIP-8,1,Aux stage
RV1,100kA,16mm PCB,1,Level
SW1,3PDT Footswitch,Through-Hole,1,Bypass
L1,10mH,Through-Hole,1,Filter choke
";

        // 1. Stage with group_designators = true and ReplaceUnallocated mode
        let staged = stage_bom_import(
            &db,
            StageBomImportInput {
                kit_build_id: build.id.clone(),
                csv_text: bom_csv.into(),
                import_mode: BomImportMode::ReplaceUnallocated,
                group_designators: true,
                preset_name: None,
                column_mapping: None,
            },
        )
        .await
        .expect("stage bom import");

        // R1, R2, R5 (4k7, 4.7k, 4700R) all normalize to Resistors + 4.7kΩ + Through-Hole -> merged into 1 row!
        // Total 10 input rows -> 8 staged rows
        assert_eq!(staged.rows.len(), 8);

        let r_row = &staged.rows[0];
        assert_eq!(r_row.category, "Resistors");
        assert_eq!(r_row.reference_designators, "R1, R2, R5");
        assert_eq!(r_row.quantity, 3);
        assert_eq!(r_row.match_status, ImportMatchStatus::ExactMatch);
        assert_eq!(
            r_row.matched_component_id.as_deref(),
            Some(r_4k7.id.as_str())
        );

        // Verify category inference from designator prefixes when Category column is unmapped:
        assert_eq!(staged.rows[1].category, "Capacitors"); // C1
        assert_eq!(staged.rows[2].category, "Semiconductors — Discrete"); // D1
        assert_eq!(staged.rows[3].category, "Semiconductors — Discrete"); // Q1
        assert_eq!(staged.rows[4].category, "Integrated Circuits (ICs)"); // U2
        assert_eq!(
            staged.rows[4].matched_component_id.as_deref(),
            Some(ic_tl072.id.as_str())
        );
        assert_eq!(staged.rows[5].category, "Potentiometers & Encoders"); // RV1
        assert_eq!(staged.rows[6].category, "Switches, Jacks & Connectors"); // SW1
        assert_eq!(
            staged.rows[7].category,
            "Inductors, Transformers & Magnetics"
        ); // L1

        // 2. Commit the ReplaceUnallocated BOM batch
        let commit_summary = commit_bom_import_batch(&db, &staged.id)
            .await
            .expect("commit bom import");
        assert_eq!(commit_summary.unallocated_lines_replaced, 1);
        assert_eq!(commit_summary.bom_lines_created, 8);
        assert_eq!(commit_summary.bom_lines_linked, 2); // r_4k7 and ic_tl072

        let final_lines = list_bom_lines_for_build(&db, &build.id).await.unwrap();
        // 1 preserved Reserved line (U1) + 8 newly imported lines = 9 lines; old_unalloc (R99) was replaced!
        assert_eq!(final_lines.len(), 9);
        assert!(final_lines.iter().any(|l| l.id == reserved_line.id && l.fulfillment_status == "Reserved"));
        assert!(!final_lines.iter().any(|l| l.id == old_unalloc.id));

        let imported_r = final_lines
            .iter()
            .find(|l| l.reference_designators == "R1, R2, R5")
            .expect("merged R1, R2, R5 line");
        assert_eq!(imported_r.quantity_required, 3);
        assert_eq!(
            imported_r.component_id.as_deref(),
            Some(r_4k7.id.as_str())
        );

        // 3. Also verify group_designators = false + Append mode keeps single-designator rows separate and appends
        let append_tsv = "Reference\tValue\nR10\t10k\nR11\t10k\n";
        let staged_ungrouped = stage_bom_import(
            &db,
            StageBomImportInput {
                kit_build_id: build.id.clone(),
                csv_text: append_tsv.into(),
                import_mode: BomImportMode::Append,
                group_designators: false,
                preset_name: None,
                column_mapping: None,
            },
        )
        .await
        .expect("stage ungrouped tsv");
        assert_eq!(staged_ungrouped.rows.len(), 2);
        assert_eq!(staged_ungrouped.rows[0].reference_designators, "R10");
        assert_eq!(staged_ungrouped.rows[1].reference_designators, "R11");

        let append_summary = commit_bom_import_batch(&db, &staged_ungrouped.id)
            .await
            .expect("commit append batch");
        assert_eq!(append_summary.unallocated_lines_replaced, 0);
        assert_eq!(append_summary.bom_lines_created, 2);
        assert_eq!(
            list_bom_lines_for_build(&db, &build.id).await.unwrap().len(),
            11
        );
    }

    #[tokio::test]
    async fn rolls_back_component_and_bom_import_commits_atomically_on_mid_batch_error() {
        use crate::builds::{
            CreateBomLine, CreateKitBuild, add_bom_line, create_kit_build,
            list_bom_lines_for_build,
        };

        let db = init_memory_db().await.expect("memory db");

        // 1. Stage a 2-row component import where row 0 creates a new location + component,
        //    and corrupt row 1 to AddStockLot with no matched_component_id.
        let comp_csv = "Value,Qty,Location\n4k7 1% Metal Film,10,Atomic Crate > Bag 1\n10k 1% Metal Film,20,Atomic Crate > Bag 2\n";
        let staged_comp = stage_component_import(
            &db,
            StageComponentImportInput {
                csv_text: comp_csv.into(),
                preset_name: None,
                column_mapping: None,
                default_location_id: None,
                default_supplier: Some("Tayda".into()),
            },
        )
        .await
        .expect("stage component import");
        assert_eq!(staged_comp.rows.len(), 2);

        let conn = db.connect().await.unwrap();
        conn.execute(
            "UPDATE import_rows SET chosen_action = 'add_stock_lot', matched_component_id = NULL WHERE id = ?1",
            (staged_comp.rows[1].id.as_str(),),
        )
        .await
        .unwrap();

        let comp_err = commit_component_import_batch(&db, &staged_comp.id).await;
        assert!(comp_err.is_err(), "expected mid-batch error on row 1");

        // Verify row 0's component and storage locations were rolled back completely
        assert!(list_components(&db).await.unwrap().is_empty());
        assert!(list_storage_locations(&db).await.unwrap().is_empty());
        assert_eq!(
            get_import_batch(&db, &staged_comp.id)
                .await
                .unwrap()
                .unwrap()
                .status,
            "staged"
        );

        // 2. Stage a ReplaceUnallocated BOM import on a build with an existing Unallocated BOM line,
        //    and corrupt row 1 to LinkComponent with a non-existent component ID.
        let build = create_kit_build(
            &db,
            CreateKitBuild {
                name: "Atomic Test Build".into(),
                project_type: "Overdrive".into(),
                status: None,
                location_id: None,
                description: "".into(),
            },
        )
        .await
        .unwrap();

        let existing_unalloc = add_bom_line(
            &db,
            CreateBomLine {
                kit_build_id: build.id.clone(),
                raw_spec_text: "100kB 16mm Pot".into(),
                category: None,
                package_or_mounting: None,
                quantity_required: 1,
                reference_designators: "VR1".into(),
                included_in_kit: false,
                component_id: None,
                substitution_note: "".into(),
                notes: "".into(),
            },
        )
        .await
        .unwrap();

        let bom_csv = "Reference,Value\nR1,4k7\nR2,10k\n";
        let staged_bom = stage_bom_import(
            &db,
            StageBomImportInput {
                kit_build_id: build.id.clone(),
                csv_text: bom_csv.into(),
                import_mode: BomImportMode::ReplaceUnallocated,
                group_designators: false,
                preset_name: None,
                column_mapping: None,
            },
        )
        .await
        .expect("stage bom import");
        assert_eq!(staged_bom.rows.len(), 2);

        conn.execute(
            "UPDATE import_rows SET chosen_action = 'create_new', matched_component_id = NULL WHERE id = ?1",
            (staged_bom.rows[0].id.as_str(),),
        )
        .await
        .unwrap();
        conn.execute(
            "UPDATE import_rows SET chosen_action = 'link_component', matched_component_id = NULL WHERE id = ?1",
            (staged_bom.rows[1].id.as_str(),),
        )
        .await
        .unwrap();

        let bom_err = commit_bom_import_batch(&db, &staged_bom.id).await;
        assert!(bom_err.is_err(), "expected mid-batch error on BOM row 1");

        // Verify existing_unalloc was NOT deleted, row 0's component/line were rolled back, and batch remains 'staged'
        let remaining_lines = list_bom_lines_for_build(&db, &build.id).await.unwrap();
        assert_eq!(remaining_lines.len(), 1);
        assert_eq!(remaining_lines[0].id, existing_unalloc.id);
        assert!(list_components(&db).await.unwrap().is_empty());
        assert_eq!(
            get_import_batch(&db, &staged_bom.id)
                .await
                .unwrap()
                .unwrap()
                .status,
            "staged"
        );
    }
}
