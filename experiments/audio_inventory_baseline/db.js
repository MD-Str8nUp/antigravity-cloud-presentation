const path = require('node:path');
const fs = require('node:fs');
const { DatabaseSync } = require('node:sqlite');

const DATA_DIR = path.join(__dirname, 'data');
if (!fs.existsSync(DATA_DIR)) {
  fs.mkdirSync(DATA_DIR, { recursive: true });
}

const DB_PATH = path.join(DATA_DIR, 'inventory.sqlite');
const db = new DatabaseSync(DB_PATH);

// Enable WAL and Foreign Keys
db.exec('PRAGMA journal_mode = WAL;');
db.exec('PRAGMA foreign_keys = ON;');

// Initialize Schema
db.exec(`
  CREATE TABLE IF NOT EXISTS components (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    category TEXT NOT NULL,
    value TEXT NOT NULL DEFAULT '',
    package_footprint TEXT NOT NULL DEFAULT '',
    specs TEXT NOT NULL DEFAULT '',
    qty_on_hand INTEGER NOT NULL DEFAULT 0,
    min_qty INTEGER NOT NULL DEFAULT 0,
    storage_location TEXT NOT NULL DEFAULT '',
    supplier TEXT NOT NULL DEFAULT '',
    supplier_sku TEXT NOT NULL DEFAULT '',
    unit_cost REAL NOT NULL DEFAULT 0,
    notes TEXT NOT NULL DEFAULT '',
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
  );

  CREATE TABLE IF NOT EXISTS projects (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    project_type TEXT NOT NULL DEFAULT 'Eurorack Module',
    format_specs TEXT NOT NULL DEFAULT '',
    status TEXT NOT NULL DEFAULT 'Planning',
    source_vendor TEXT NOT NULL DEFAULT '',
    doc_url TEXT NOT NULL DEFAULT '',
    notes TEXT NOT NULL DEFAULT '',
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
  );

  CREATE TABLE IF NOT EXISTS bom_items (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    project_id INTEGER NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    component_id INTEGER REFERENCES components(id) ON DELETE SET NULL,
    designators TEXT NOT NULL DEFAULT '',
    part_name TEXT NOT NULL,
    category TEXT NOT NULL DEFAULT '',
    value TEXT NOT NULL DEFAULT '',
    package_footprint TEXT NOT NULL DEFAULT '',
    qty_required INTEGER NOT NULL DEFAULT 1,
    qty_allocated INTEGER NOT NULL DEFAULT 0,
    is_consumed INTEGER NOT NULL DEFAULT 0,
    is_populated INTEGER NOT NULL DEFAULT 0,
    notes TEXT NOT NULL DEFAULT ''
  );

  CREATE TABLE IF NOT EXISTS build_logs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    project_id INTEGER NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    log_type TEXT NOT NULL DEFAULT 'Progress',
    entry TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
  );
`);

/**
 * Normalize electronic component values for smart BOM matching.
 * Understands R/k/M notations (e.g. 4k7 -> 4.7k, 100kΩ -> 100k, 470R -> 470r),
 * capacitor equivalences (0.1uF -> 100nf, 0.01uF -> 10nf, 2n2 -> 2.2nf),
 * and IC/part prefixes (TL072CP -> tl072).
 */
function normalizeElectronicValue(rawVal = '') {
  let s = String(rawVal)
    .trim()
    .toLowerCase()
    .replace(/ohms?|ω|Ω/g, 'r')
    .replace(/μ|µ/g, 'u')
    .replace(/\s+/g, '');

  if (!s) return '';

  // Strip trailing 'r' after unit+fraction (e.g. 4k7Ω -> 4k7r -> 4k7)
  s = s.replace(/^(\d+[kmnup]\d+)r$/, '$1');

  // Convert R/k/M/n/u/p middle-decimal notation: 4k7 -> 4.7k, 2n2 -> 2.2n, 1u5 -> 1.5u, 47r0 -> 47.0r
  const rkmMatch = s.match(/^(\d+)([rkmnup])(\d+)(f?)$/);
  if (rkmMatch) {
    const [, whole, unit, frac, farad] = rkmMatch;
    s = `${whole}.${frac}${unit}${farad}`;
  }

  // Strip trailing 'r' if preceded by k or m (e.g. 100kr -> 100k)
  s = s.replace(/([km])r$/, '$1');

  // Normalize plain ohms and number+r for resistors (e.g. 470, 470r, 470.0r -> 470r; 4.7r -> 4.7r)
  if (/^\d+(\.\d+)?r?$/.test(s)) {
    s = s.replace(/r$/, '').replace(/\.0+$/, '') + 'r';
  }

  // Normalize capacitors to picofarads for exact equivalence (0.1uF == 100nF == 100000pF)
  const capMatch = s.match(/^(\d*\.?\d+)(p|n|u)f?$/);
  if (capMatch) {
    const num = parseFloat(capMatch[1]);
    const unit = capMatch[2];
    if (!Number.isNaN(num)) {
      const mult = unit === 'u' ? 1e6 : unit === 'n' ? 1e3 : 1;
      const pf = Math.round(num * mult * 1000) / 1000;
      return `cap:${pf}pf`;
    }
  }

  return s;
}

function normalizeFootprint(rawFp = '') {
  const s = String(rawFp).trim().toLowerCase();
  if (!s) return '';
  if (s.includes('0603')) return '0603';
  if (s.includes('0805')) return '0805';
  if (s.includes('1206')) return '1206';
  if (s.includes('dip-8') || s.includes('dip8')) return 'dip-8';
  if (s.includes('dip-14') || s.includes('dip14')) return 'dip-14';
  if (s.includes('dip-16') || s.includes('dip16')) return 'dip-16';
  if (s.includes('soic-8') || s.includes('soic8')) return 'soic-8';
  if (s.includes('soic-14') || s.includes('soic14')) return 'soic-14';
  if (s.includes('soic-16') || s.includes('soic16')) return 'soic-16';
  if (s.includes('to-92') || s.includes('to92')) return 'to-92';
  if (s.includes('to-220') || s.includes('to220')) return 'to-220';
  if (s.includes('thonkiconn') || s.includes('pj398sm') || s.includes('3.5mm')) return 'thonkiconn';
  if (s.includes('9mm')) return '9mm';
  if (s.includes('16mm')) return '16mm';
  if (s.includes('3pdt')) return '3pdt';
  if (s.includes('axial') || s.includes('1/4w') || s.includes('0.25w') || s.includes('tht')) return 'tht';
  return s;
}

/**
 * Find the best matching component from inventory for a BOM line.
 */
function findBestComponentMatch(bomLine, allComponents) {
  const targetVal = normalizeElectronicValue(bomLine.value);
  const rawValLower = String(bomLine.value || '').trim().toLowerCase();
  const rawNameLower = String(bomLine.part_name || '').trim().toLowerCase();
  const targetFp = normalizeFootprint(bomLine.package_footprint);
  const targetCat = String(bomLine.category || '').trim().toLowerCase();

  let best = null;
  let bestScore = 0;

  for (const comp of allComponents) {
    let score = 0;
    const compVal = normalizeElectronicValue(comp.value);
    const compRawVal = String(comp.value || '').trim().toLowerCase();
    const compName = String(comp.name || '').trim().toLowerCase();
    const compFp = normalizeFootprint(comp.package_footprint);
    const compCat = String(comp.category || '').trim().toLowerCase();

    // Value matching
    if (targetVal && compVal && targetVal === compVal) {
      score += 60;
    } else if (rawValLower && compRawVal && (compRawVal.includes(rawValLower) || rawValLower.includes(compRawVal))) {
      score += 40;
    } else if (rawValLower && compName.includes(rawValLower)) {
      score += 35;
    } else if (rawNameLower && compRawVal && rawNameLower.includes(compRawVal) && compRawVal.length >= 3) {
      score += 30;
    }

    if (score === 0) continue; // Require at least some value/part identity match

    // Footprint matching
    if (targetFp && compFp) {
      if (targetFp === compFp) {
        score += 25;
      } else if (
        String(comp.package_footprint).toLowerCase().includes(String(bomLine.package_footprint).toLowerCase()) ||
        String(bomLine.package_footprint).toLowerCase().includes(String(comp.package_footprint).toLowerCase())
      ) {
        score += 15;
      } else {
        // Conflicting explicit footprints (e.g. 0603 vs THT or DIP-8 vs SOIC-8)
        score -= 20;
      }
    }

    // Category / Name similarity
    if (targetCat && compCat && targetCat === compCat) {
      score += 15;
    } else if (rawNameLower) {
      if (rawNameLower.includes('resistor') && compCat.includes('resistor')) score += 12;
      if ((rawNameLower.includes('cap') || rawNameLower.includes('mlcc') || rawNameLower.includes('film')) && compCat.includes('capacitor')) score += 12;
      if ((rawNameLower.includes('pot') || rawNameLower.includes('trimmer')) && compCat.includes('potentiometer')) score += 12;
      if ((rawNameLower.includes('jack') || rawNameLower.includes('switch') || rawNameLower.includes('socket') || rawNameLower.includes('header')) && compCat.includes('jack')) score += 12;
      if ((rawNameLower.includes('op-amp') || rawNameLower.includes('ic') || rawNameLower.includes('diode') || rawNameLower.includes('transistor') || rawNameLower.includes('led')) && compCat.includes('semiconductor')) score += 12;
    }

    if (score > bestScore && score >= 45) {
      bestScore = score;
      best = comp;
    }
  }

  return best;
}

/**
 * Seed realistic Eurorack & Guitar Pedal DIY inventory and kit builds if the DB is empty.
 */
function seedIfEmpty() {
  const { count } = db.prepare('SELECT COUNT(*) as count FROM components').get();
  if (count > 0) return;

  const insertComp = db.prepare(`
    INSERT INTO components (
      name, category, value, package_footprint, specs,
      qty_on_hand, min_qty, storage_location, supplier, supplier_sku, unit_cost, notes
    ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
  `);

  const seedComponents = [
    // Resistors
    ['100kΩ Metal Film Resistor', 'Resistors', '100k', 'THT Axial 1/4W', '1% 0.25W 50ppm', 140, 25, 'Resistor Binder A - p.6', 'Tayda', 'A-2214', 0.02, 'Standard audio/CV biasing & op-amp feedback'],
    ['10kΩ Metal Film Resistor', 'Resistors', '10k', 'THT Axial 1/4W', '1% 0.25W 50ppm', 95, 25, 'Resistor Binder A - p.3', 'Tayda', 'A-2202', 0.02, 'Universal pull-up/divider'],
    ['1kΩ Metal Film Resistor', 'Resistors', '1k', 'THT Axial 1/4W', '1% 0.25W 50ppm', 80, 20, 'Resistor Binder A - p.1', 'Tayda', 'A-2190', 0.02, 'Eurorack output protection & LED current limiting'],
    ['4.7kΩ Metal Film Resistor', 'Resistors', '4.7k', 'THT Axial 1/4W', '1% 0.25W', 42, 20, 'Resistor Binder A - p.2', 'Tayda', 'A-2198', 0.02, ''],
    ['47kΩ Metal Film Resistor', 'Resistors', '47k', 'THT Axial 1/4W', '1% 0.25W', 60, 20, 'Resistor Binder A - p.5', 'Tayda', 'A-2210', 0.02, ''],
    ['1MΩ Metal Film Resistor', 'Resistors', '1M', 'THT Axial 1/4W', '1% 0.25W', 35, 15, 'Resistor Binder A - p.9', 'Tayda', 'A-2226', 0.02, 'Guitar pedal input impedance / anti-pop pulldown'],
    ['100kΩ SMD Chip Resistor', 'Resistors', '100k', 'SMD 0603', '0.1% Thin Film 25ppm', 180, 50, 'SMD Book 0603 - Row 14', 'Mouser', '667-ERA-3AEB104V', 0.08, 'Precision 1V/Oct CV summing'],
    ['10kΩ SMD Chip Resistor', 'Resistors', '10k', 'SMD 0603', '1% Thick Film', 210, 50, 'SMD Book 0603 - Row 8', 'Mouser', '71-CRCW060310K0FKEA', 0.03, ''],
    ['10kΩ Multiturn Trim Pot (3296W)', 'Resistors', '10k Trim', 'THT 3296W Top-Adjust', '25-Turn Cermet', 8, 5, 'Drawer Bin B-04', 'Tayda', 'A-594', 0.35, 'V/Oct & offset calibration trimmer'],

    // Capacitors
    ['100nF Multilayer Ceramic Capacitor', 'Capacitors', '100nF', 'THT 2.54mm Pitch', '50V X7R Decoupling', 160, 40, 'Cap Drawer C-01', 'Tayda', 'A-4008', 0.03, 'IC power rail bypass (0.1uF)'],
    ['100nF SMD Ceramic Capacitor', 'Capacitors', '100nF', 'SMD 0603', '50V X7R', 250, 50, 'SMD Book 0603 - Cap p.2', 'Mouser', '80-C0603C104K5R', 0.03, 'SMD IC decoupling'],
    ['10uF Electrolytic Capacitor', 'Capacitors', '10uF', 'Radial 5x11mm', '50V 105°C Low ESR', 45, 20, 'Cap Drawer C-08', 'Mouser', '647-UVR1H100MDD', 0.12, 'Eurorack rail entry filtering & audio coupling'],
    ['47uF Electrolytic Capacitor', 'Capacitors', '47uF', 'Radial 6.3x11mm', '35V Nichicon Audio', 14, 15, 'Cap Drawer C-09', 'Mouser', '647-UFW1V470MED', 0.22, 'Pedal power supply filtering (Low stock!)'],
    ['22nF Box Film Capacitor (WIMA)', 'Capacitors', '22nF', 'Box 5mm Pitch', '63V Polyester 5%', 28, 10, 'Film Cap Bin F-03', 'Mouser', '505-MKS2.022/63/5', 0.28, 'Guitar pedal tone stack'],
    ['1nF C0G/NP0 Film/Ceramic Cap', 'Capacitors', '1nF', 'SMD 0603', '50V C0G/NP0 1%', 18, 20, 'SMD Book 0603 - Cap p.1', 'Mouser', '81-GRM1885C1H102FA01', 0.14, 'VCF integrator high-stability cap'],

    // Semiconductors & ICs
    ['TL072CP Dual Low-Noise JFET Op-Amp', 'Semiconductors & ICs', 'TL072', 'DIP-8', 'Texas Instruments JFET Input', 24, 10, 'IC ESD Box #1 - Slot A2', 'Mouser', '595-TL072CP', 0.65, 'Workhorse audio & CV dual op-amp'],
    ['TL074CN Quad JFET Op-Amp', 'Semiconductors & ICs', 'TL074', 'DIP-14', 'Texas Instruments Quad Op-Amp', 12, 8, 'IC ESD Box #1 - Slot A3', 'Mouser', '595-TL074CN', 0.78, ''],
    ['TL072CDR Dual Op-Amp (SMD)', 'Semiconductors & ICs', 'TL072', 'SOIC-8', 'SOIC-8 Surface Mount', 16, 10, 'IC ESD Box #2 - SMD', 'Mouser', '595-TL072CDR', 0.52, ''],
    ['LM13700N Dual OTA', 'Semiconductors & ICs', 'LM13700', 'DIP-16', 'Dual Operational Transconductance Amp', 4, 6, 'IC ESD Box #1 - Slot B1', 'Thonk', 'IC-LM13700-DIP', 2.40, 'Classic VCF/VCA core IC — getting scarce in DIP!'],
    ['SSI2144 Four-Pole VCF IC', 'Semiconductors & ICs', 'SSI2144', 'SSOP-16', 'Sound Semiconductor Ladder/4-Pole VCF', 5, 3, 'IC ESD Box #2 - Synth ICs', 'Thonk', 'SSI-2144', 4.50, 'Used in Mutable Ripples 2020 & modern filters'],
    ['1N4148 High-Speed Switching Diode', 'Semiconductors & ICs', '1N4148', 'DO-35 Axial', '100V 200mA Silicon', 120, 30, 'Diode Drawer D-01', 'Tayda', 'A-157', 0.02, 'Soft/hard clipping & logic steering'],
    ['1N5817 Schottky Barrier Diode', 'Semiconductors & ICs', '1N5817', 'DO-41 Axial', '20V 1A Low Vf Drop', 32, 15, 'Diode Drawer D-02', 'Tayda', 'A-161', 0.06, 'Reverse polarity protection for pedals & Eurorack'],
    ['1N34A Germanium Point-Contact Diode', 'Semiconductors & ICs', '1N34A', 'DO-7 Axial', 'NOS Germanium Vf ≈ 0.28V', 6, 8, 'Vintage Semis Tin #1', 'StompBoxParts', 'D-1N34A', 1.25, 'Klon Centaur & vintage fuzz clipping pair'],
    ['2N3904 NPN General Purpose Transistor', 'Semiconductors & ICs', '2N3904', 'TO-92', '40V 200mA NPN BJT', 65, 20, 'Transistor Drawer T-01', 'Tayda', 'A-111', 0.04, ''],

    // Potentiometers & Encoders
    ['100kΩ Linear Potentiometer (B100k)', 'Potentiometers & Encoders', 'B100k', 'Alpha 9mm Vertical', 'Linear Taper • 6mm Round/D Shaft', 14, 10, 'Eurorack Pots Bin P-01', 'Thonk', 'ALPHA-9MM-B100K', 1.35, 'Eurorack CV & filter cutoff controls'],
    ['10kΩ Linear Potentiometer (B10k)', 'Potentiometers & Encoders', 'B10k', 'Alpha 9mm Vertical', 'Linear Taper • T18 Knurled Shaft', 9, 10, 'Eurorack Pots Bin P-02', 'Thonk', 'ALPHA-9MM-B10K', 1.35, ''],
    ['100kΩ Audio/Log Potentiometer (A100k)', 'Potentiometers & Encoders', 'A100k', 'Alpha 16mm PCB', 'Logarithmic Taper • Right-Angle PCB', 7, 8, 'Pedal Pots Bin P-10', 'Tayda', 'A-1970', 1.10, 'Pedal volume / output level'],
    ['10kΩ Linear Dual-Gang Pot (B10k Dual)', 'Potentiometers & Encoders', 'B10k Dual', 'Alpha 16mm PCB', 'Dual-Gang Linear • Right-Angle PCB', 2, 3, 'Pedal Pots Bin P-12', 'StompBoxParts', 'POT-16-DUAL-B10K', 2.25, 'Klon Centaur Gain dual-gang pot'],

    // Jacks, Switches & Hardware
    ['Thonkiconn 3.5mm Mono Jack (PJ398SM)', 'Jacks, Switches & Hardware', 'PJ398SM', 'Thonkiconn', 'QingPu Vertical PCB Mount + Knurled Nut', 42, 30, 'Eurorack Jacks Bin J-01', 'Thonk', 'PJ398SM', 0.55, 'Standard Eurorack patch jack'],
    ['3PDT Latching Footswitch (Blue)', 'Jacks, Switches & Hardware', '3PDT', '3PDT Solder Lug', '9-Pin True Bypass Stomp Switch', 5, 4, 'Pedal Hardware Crate H-01', 'StompBoxParts', 'SW-3PDT-PRO', 2.60, 'True bypass pedal switch'],
    ['1/4" Mono Enclosed Switched Jack', 'Jacks, Switches & Hardware', '1/4" Mono Jack', '6.35mm PCB/Lug', 'Neutrik / Lumberg Style Mono Jack', 12, 8, 'Pedal Hardware Crate H-02', 'Tayda', 'A-1120', 0.75, 'Guitar pedal In/Out jacks'],
    ['Eurorack 10-Pin Shrouded Power Header', 'Jacks, Switches & Hardware', '2x5 Box Header', 'THT 2.54mm 2x5', '10-Pin Shrouded IDC Male Header', 15, 6, 'Drawer Bin H-05', 'Tayda', 'A-296', 0.18, 'Standard Eurorack module power connector'],
    ['125B Die-Cast Aluminum Enclosure', 'Jacks, Switches & Hardware', '125B Enclosure', '125B (122x66x39mm)', 'Powder-Coated Matte Silver', 2, 2, 'Enclosure Shelf E-01', 'Tayda', 'A-5122', 6.99, 'Top-jack friendly guitar pedal enclosure']
  ];

  db.exec('BEGIN');
  try {
    for (const c of seedComponents) {
      insertComp.run(...c);
    }

    const insertProject = db.prepare(`
      INSERT INTO projects (
        name, project_type, format_specs, status, source_vendor, doc_url, notes
      ) VALUES (?, ?, ?, ?, ?, ?, ?)
    `);

    // Project 1: Mutable Instruments Ripples (2020) - Eurorack VCF
    const p1 = insertProject.run(
      'Mutable Instruments Ripples (2020 Liquid Filter)',
      'Eurorack Module',
      '8HP • +12V: 35mA / -12V: 32mA • 25mm Deep',
      'Soldering',
      'Amazingsynth / Pusherman PCB & Panel',
      'https://pichenettes.github.io/mutable-instruments-diy-archive/ripples_2020/',
      'Classic 2-pole / 4-pole liquid multimode VCF built around the SSI2144 chip with built-in final VCA and drive.'
    );
    const p1Id = Number(p1.lastInsertRowid);

    // Project 2: Klon Centaur Professional Overdrive - Guitar Pedal
    const p2 = insertProject.run(
      'Klon Centaur Transparent Overdrive (125B)',
      'Guitar Pedal',
      '125B Enclosure • 9V DC (Internal ±18V Charge Pump)',
      'Planning',
      'PedalPCB (Kliche Mini / Standard)',
      'https://www.pedalpcb.com/',
      'Dual-gang gain pot mixes clean headroom with germanium diode clipping network. Needs matched 1N34A diodes and MAX1044/TC1044SCPA charge pump.'
    );
    const p2Id = Number(p2.lastInsertRowid);

    // Project 3: Befaco Rampage / Dual Slope Generator
    const p3 = insertProject.run(
      'AI Synthesis AI004 OTA Vintage Filter',
      'Eurorack Module',
      '8HP • +12V: 28mA / -12V: 26mA • THT Friendly',
      'Parts Ordered',
      'AI Synthesis',
      'https://aisynthesis.com/',
      'Through-hole dual LM13700 MS-20 style high-pass / low-pass resonant filter.'
    );
    const p3Id = Number(p3.lastInsertRowid);

    const allComps = db.prepare('SELECT * FROM components').all();
    const findId = (val, fp) => {
      const m = findBestComponentMatch({ value: val, package_footprint: fp, part_name: val }, allComps);
      return m ? m.id : null;
    };

    const insertBom = db.prepare(`
      INSERT INTO bom_items (
        project_id, component_id, designators, part_name, category, value,
        package_footprint, qty_required, qty_allocated, is_consumed, is_populated, notes
      ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
    `);

    // BOM for Project 1 (Ripples 2020 - Soldering in progress, parts allocated)
    const p1Bom = [
      [p1Id, findId('SSI2144', 'SSOP-16'), 'U1', 'SSI2144 Four-Pole VCF IC', 'Semiconductors & ICs', 'SSI2144', 'SSOP-16', 1, 1, 0, 1, 'Soldered first with flux pen; inspected pins under loupe'],
      [p1Id, findId('TL072', 'SOIC-8'), 'U2, U3, U4', 'TL072CDR Dual Op-Amp', 'Semiconductors & ICs', 'TL072', 'SOIC-8', 3, 3, 0, 1, 'Audio buffer & CV conditioning op-amps'],
      [p1Id, findId('100k', 'SMD 0603'), 'R1-R14', '100kΩ Precision Resistor', 'Resistors', '100k', 'SMD 0603', 14, 14, 0, 1, '0.1% thin film for 1V/Oct exponential converter network'],
      [p1Id, findId('100nF', 'SMD 0603'), 'C1-C10', '100nF Decoupling Cap', 'Capacitors', '100nF', 'SMD 0603', 10, 10, 0, 1, 'Power rail bypass caps'],
      [p1Id, findId('1nF', 'SMD 0603'), 'C11, C12, C13, C14', '1nF C0G Filter Cap', 'Capacitors', '1nF', 'SMD 0603', 4, 4, 0, 1, 'Matched C0G/NP0 integrator caps'],
      [p1Id, findId('PJ398SM', 'Thonkiconn'), 'J1-J9', 'Thonkiconn 3.5mm Mono Jack', 'Jacks, Switches & Hardware', 'PJ398SM', 'Thonkiconn', 9, 9, 0, 0, 'Wait to solder until front panel dry-fit'],
      [p1Id, findId('B100k', 'Alpha 9mm Vertical'), 'RV1, RV2, RV3', '100kΩ Linear Potentiometer', 'Potentiometers & Encoders', 'B100k', 'Alpha 9mm Vertical', 3, 3, 0, 0, 'Freq, Resonance, FM attenuverter'],
      [p1Id, findId('2x5 Box Header', 'THT 2.54mm 2x5'), 'J_PWR', 'Eurorack 10-Pin Shrouded Header', 'Jacks, Switches & Hardware', '2x5 Box Header', 'THT 2.54mm 2x5', 1, 1, 0, 1, 'Check red stripe -12V orientation']
    ];
    for (const row of p1Bom) insertBom.run(...row);

    // BOM for Project 2 (Klon Centaur - Planning, shows mix of in-stock & shortages)
    const p2Bom = [
      [p2Id, findId('TL072', 'DIP-8'), 'IC1, IC2', 'TL072CP Dual Op-Amp', 'Semiconductors & ICs', 'TL072', 'DIP-8', 2, 2, 0, 0, 'Use sockets'],
      [p2Id, null, 'IC3', 'TC1044SCPA / MAX1044 Voltage Converter', 'Semiconductors & ICs', 'TC1044SCPA', 'DIP-8', 1, 0, 0, 0, 'Ensure "S" high-frequency suffix so charge pump whine is above 20kHz!'],
      [p2Id, findId('1N34A', 'DO-7 Axial'), 'D1, D2', '1N34A Germanium Clipping Diode', 'Semiconductors & ICs', '1N34A', 'DO-7 Axial', 2, 2, 0, 0, 'Measure Vf ~0.28V on component tester'],
      [p2Id, findId('B10k Dual', 'Alpha 16mm PCB'), 'VR1 (GAIN)', '10kΩ Dual-Gang Linear Pot', 'Potentiometers & Encoders', 'B10k Dual', 'Alpha 16mm PCB', 1, 1, 0, 0, 'Controls clean blend + overdrive gain simultaneously'],
      [p2Id, findId('A100k', 'Alpha 16mm PCB'), 'VR2, VR3', '100kΩ Audio/Log Potentiometer', 'Potentiometers & Encoders', 'A100k', 'Alpha 16mm PCB', 2, 0, 0, 0, 'Treble & Output controls'],
      [p2Id, findId('22nF', 'Box 5mm Pitch'), 'C4, C7, C11', '22nF Film Capacitor', 'Capacitors', '22nF', 'Box 5mm Pitch', 3, 0, 0, 0, 'WIMA box caps'],
      [p2Id, findId('3PDT', '3PDT Solder Lug'), 'SW1', '3PDT Footswitch', 'Jacks, Switches & Hardware', '3PDT', '3PDT Solder Lug', 1, 0, 0, 0, ''],
      [p2Id, findId('125B Enclosure', '125B'), 'ENC1', '125B Die-Cast Enclosure', 'Jacks, Switches & Hardware', '125B Enclosure', '125B (122x66x39mm)', 1, 0, 0, 0, 'Drill template #125B-3KNOB']
    ];
    for (const row of p2Bom) insertBom.run(...row);

    // BOM for Project 3 (AI004 Filter - shows LM13700 stock tightness)
    const p3Bom = [
      [p3Id, findId('LM13700', 'DIP-16'), 'U1, U2', 'LM13700N Dual OTA', 'Semiconductors & ICs', 'LM13700', 'DIP-16', 2, 2, 0, 0, 'Core OTA filter stages'],
      [p3Id, findId('TL074', 'DIP-14'), 'U3', 'TL074CN Quad Op-Amp', 'Semiconductors & ICs', 'TL074', 'DIP-14', 1, 1, 0, 0, ''],
      [p3Id, findId('100k', 'THT Axial 1/4W'), 'R1-R12', '100kΩ Metal Film Resistor', 'Resistors', '100k', 'THT Axial 1/4W', 12, 12, 0, 0, ''],
      [p3Id, findId('PJ398SM', 'Thonkiconn'), 'J1-J6', 'Thonkiconn 3.5mm Mono Jack', 'Jacks, Switches & Hardware', 'PJ398SM', 'Thonkiconn', 6, 0, 0, 0, '']
    ];
    for (const row of p3Bom) insertBom.run(...row);

    // Build Logs
    const insertLog = db.prepare(`
      INSERT INTO build_logs (project_id, log_type, entry, created_at)
      VALUES (?, ?, ?, datetime('now', ?))
    `);
    insertLog.run(p1Id, 'Progress', 'Allocated all SMD passives, SSI2144, and panel hardware into anti-static project tray #1.', '-3 days');
    insertLog.run(p1Id, 'Progress', 'Reflowed U1 (SSI2144 SSOP-16) and U2-U4 (TL072 SOIC-8) with drag soldering and no-clean tacky flux. Continuity checked all pins to adjacent pads — zero bridges.', '-2 days');
    insertLog.run(p1Id, 'Calibration', 'Populated 0603 0.1% resistors and C0G 1nF filter caps. Ready to mount Thonkiconn jacks and Alpha 9mm pots to panel before final soldering and V/Oct tracking calibration.', '-1 day');
    insertLog.run(p2Id, 'Note', 'Checked inventory for Klon build: matched a pair of NOS 1N34A germanium diodes at Vf = 0.278V and 0.281V. Need to order TC1044SCPA charge pump IC.', '-12 hours');

    db.exec('COMMIT');
  } catch (err) {
    db.exec('ROLLBACK');
    throw err;
  }
}

seedIfEmpty();

/**
 * Query all components with computed `qty_allocated` and `qty_available`.
 */
function getAllComponents() {
  const sql = `
    SELECT
      c.*,
      COALESCE(SUM(CASE WHEN b.is_consumed = 0 THEN b.qty_allocated ELSE 0 END), 0) AS qty_allocated,
      MAX(0, c.qty_on_hand - COALESCE(SUM(CASE WHEN b.is_consumed = 0 THEN b.qty_allocated ELSE 0 END), 0)) AS qty_available,
      COUNT(DISTINCT CASE WHEN b.qty_allocated > 0 AND b.is_consumed = 0 THEN b.project_id END) AS allocated_project_count
    FROM components c
    LEFT JOIN bom_items b ON b.component_id = c.id
    GROUP BY c.id
    ORDER BY c.category ASC, c.name ASC
  `;
  return db.prepare(sql).all();
}

function getComponentById(id) {
  const sql = `
    SELECT
      c.*,
      COALESCE(SUM(CASE WHEN b.is_consumed = 0 THEN b.qty_allocated ELSE 0 END), 0) AS qty_allocated,
      MAX(0, c.qty_on_hand - COALESCE(SUM(CASE WHEN b.is_consumed = 0 THEN b.qty_allocated ELSE 0 END), 0)) AS qty_available
    FROM components c
    LEFT JOIN bom_items b ON b.component_id = c.id
    WHERE c.id = ?
    GROUP BY c.id
  `;
  const comp = db.prepare(sql).get(id);
  if (!comp) return null;

  // Also include which projects currently allocate or use this component
  const allocations = db.prepare(`
    SELECT
      b.id AS bom_item_id,
      b.project_id,
      p.name AS project_name,
      p.status AS project_status,
      b.designators,
      b.qty_required,
      b.qty_allocated,
      b.is_consumed
    FROM bom_items b
    JOIN projects p ON p.id = b.project_id
    WHERE b.component_id = ?
    ORDER BY p.updated_at DESC
  `).all(id);

  return { ...comp, allocations };
}

/**
 * Get all projects with BOM readiness and allocation summary metrics.
 */
function getAllProjects() {
  const projects = db.prepare(`
    SELECT * FROM projects
    ORDER BY
      CASE status
        WHEN 'Soldering' THEN 1
        WHEN 'Testing & Calibration' THEN 2
        WHEN 'Ready to Build' THEN 3
        WHEN 'Parts Ordered' THEN 4
        WHEN 'Planning' THEN 5
        WHEN 'Troubleshooting / On Hold' THEN 6
        WHEN 'Completed' THEN 7
        ELSE 8
      END,
      updated_at DESC
  `).all();

  const componentsMap = new Map(getAllComponents().map((c) => [c.id, c]));
  const allBomItems = db.prepare('SELECT * FROM bom_items ORDER BY id ASC').all();
  const bomByProject = new Map();
  for (const item of allBomItems) {
    const list = bomByProject.get(item.project_id) || [];
    list.push(item);
    bomByProject.set(item.project_id, list);
  }

  return projects.map((proj) => {
    const bom = bomByProject.get(proj.id) || [];
    const remainingAvail = new Map();
    let totalLines = bom.length;
    let totalPartsRequired = 0;
    let totalPartsAllocated = 0;
    let readyLines = 0;
    let shortageLines = 0;
    let populatedLines = 0;
    let estimatedCost = 0;

    for (const item of bom) {
      totalPartsRequired += item.qty_required;
      totalPartsAllocated += item.qty_allocated;
      if (item.is_populated) populatedLines++;

      const comp = item.component_id ? componentsMap.get(item.component_id) : null;
      if (comp) {
        estimatedCost += (comp.unit_cost || 0) * item.qty_required;
        if (!remainingAvail.has(comp.id)) {
          remainingAvail.set(comp.id, comp.qty_available);
        }
      }

      const stillNeeded = Math.max(0, item.qty_required - item.qty_allocated);
      if (item.is_consumed || stillNeeded === 0) {
        readyLines++;
      } else if (comp) {
        const avail = remainingAvail.get(comp.id) || 0;
        if (avail >= stillNeeded) {
          remainingAvail.set(comp.id, avail - stillNeeded);
          readyLines++;
        } else {
          remainingAvail.set(comp.id, 0);
          shortageLines++;
        }
      } else {
        shortageLines++;
      }
    }

    return {
      ...proj,
      summary: {
        total_lines: totalLines,
        ready_lines: readyLines,
        shortage_lines: shortageLines,
        populated_lines: populatedLines,
        total_parts_required: totalPartsRequired,
        total_parts_allocated: totalPartsAllocated,
        estimated_cost: Math.round(estimatedCost * 100) / 100
      }
    };
  });
}

/**
 * Get full project detail including enriched BOM lines and build logs.
 */
function getProjectDetail(projectId) {
  const project = db.prepare('SELECT * FROM projects WHERE id = ?').get(projectId);
  if (!project) return null;

  const allComponents = getAllComponents();
  const componentsMap = new Map(allComponents.map((c) => [c.id, c]));
  const remainingAvail = new Map(allComponents.map((c) => [c.id, c.qty_available]));

  const rawBom = db.prepare('SELECT * FROM bom_items WHERE project_id = ? ORDER BY id ASC').all(projectId);

  const bom = rawBom.map((item) => {
    const comp = item.component_id ? componentsMap.get(item.component_id) : null;
    const suggestedMatch = !comp ? findBestComponentMatch(item, allComponents) : null;

    const stillNeededToAllocate = item.is_consumed ? 0 : Math.max(0, item.qty_required - item.qty_allocated);
    const compAvailable = comp ? (remainingAvail.get(comp.id) ?? 0) : 0;
    const shortageQty = item.is_consumed
      ? 0
      : Math.max(0, stillNeededToAllocate - compAvailable);

    let stock_status = 'unlinked';
    if (item.is_consumed) {
      stock_status = 'consumed';
    } else if (item.qty_allocated >= item.qty_required) {
      stock_status = 'allocated';
    } else if (comp) {
      if (compAvailable >= stillNeededToAllocate) {
        stock_status = 'in_stock';
        remainingAvail.set(comp.id, compAvailable - stillNeededToAllocate);
      } else if (compAvailable > 0 || item.qty_allocated > 0) {
        stock_status = 'partial';
        remainingAvail.set(comp.id, 0);
      } else {
        stock_status = 'out_of_stock';
      }
    }

    return {
      ...item,
      component: comp
        ? {
            id: comp.id,
            name: comp.name,
            category: comp.category,
            value: comp.value,
            package_footprint: comp.package_footprint,
            storage_location: comp.storage_location,
            supplier: comp.supplier,
            supplier_sku: comp.supplier_sku,
            unit_cost: comp.unit_cost,
            qty_on_hand: comp.qty_on_hand,
            qty_allocated: comp.qty_allocated,
            qty_available: comp.qty_available
          }
        : null,
      suggested_component: suggestedMatch
        ? {
            id: suggestedMatch.id,
            name: suggestedMatch.name,
            value: suggestedMatch.value,
            package_footprint: suggestedMatch.package_footprint,
            qty_available: suggestedMatch.qty_available
          }
        : null,
      still_needed_to_allocate: stillNeededToAllocate,
      shortage_qty: shortageQty,
      stock_status
    };
  });

  const logs = db.prepare('SELECT * FROM build_logs WHERE project_id = ? ORDER BY created_at DESC, id DESC').all(projectId);

  return {
    ...project,
    bom,
    logs
  };
}

/**
 * Generate aggregated shopping list across active projects + low-stock inventory.
 */
function getShoppingList() {
  const allComponents = getAllComponents();
  const componentsMap = new Map(allComponents.map((c) => [c.id, c]));

  // Active project unconsumed BOM items in a single query
  const activeBomRows = db.prepare(`
    SELECT b.*, p.name AS project_name
    FROM bom_items b
    JOIN projects p ON p.id = b.project_id
    WHERE p.status != 'Completed' AND b.is_consumed = 0
    ORDER BY p.updated_at DESC, b.id ASC
  `).all();

  const bomShortagesByCompId = new Map();
  const unlinkedShortages = [];

  for (const item of activeBomRows) {
    const unallocatedNeed = Math.max(0, item.qty_required - item.qty_allocated);
    if (unallocatedNeed === 0) continue;

    if (item.component_id && componentsMap.has(item.component_id)) {
      const entry = bomShortagesByCompId.get(item.component_id) || {
        component: componentsMap.get(item.component_id),
        total_unallocated_demand: 0,
        projects: []
      };
      entry.total_unallocated_demand += unallocatedNeed;
      entry.projects.push({
        project_id: item.project_id,
        project_name: item.project_name,
        designators: item.designators,
        needed: unallocatedNeed
      });
      bomShortagesByCompId.set(item.component_id, entry);
    } else {
      unlinkedShortages.push({
        type: 'unlinked_bom',
        bom_item_id: item.id,
        project_id: item.project_id,
        project_name: item.project_name,
        part_name: item.part_name,
        category: item.category || 'Uncategorized',
        value: item.value,
        package_footprint: item.package_footprint,
        designators: item.designators,
        qty_to_order: unallocatedNeed,
        supplier: 'TBD',
        supplier_sku: '',
        unit_cost: 0,
        notes: item.notes
      });
    }
  }

  const linkedShoppingItems = [];

  for (const comp of allComponents) {
    const demandEntry = bomShortagesByCompId.get(comp.id);
    const totalUnallocatedDemand = demandEntry ? demandEntry.total_unallocated_demand : 0;
    const bomShortfall = Math.max(0, totalUnallocatedDemand - comp.qty_available);
    const lowStockShortfall = comp.min_qty > 0 && comp.qty_available <= comp.min_qty
      ? Math.max(0, comp.min_qty - comp.qty_available + totalUnallocatedDemand)
      : 0;

    const qtyToOrder = Math.max(bomShortfall, lowStockShortfall);
    if (qtyToOrder > 0) {
      linkedShoppingItems.push({
        type: bomShortfall > 0 ? 'project_shortage' : 'low_stock_restock',
        component_id: comp.id,
        part_name: comp.name,
        category: comp.category,
        value: comp.value,
        package_footprint: comp.package_footprint,
        qty_on_hand: comp.qty_on_hand,
        qty_allocated: comp.qty_allocated,
        qty_available: comp.qty_available,
        min_qty: comp.min_qty,
        total_unallocated_demand: totalUnallocatedDemand,
        bom_shortfall: bomShortfall,
        qty_to_order: qtyToOrder,
        supplier: comp.supplier || 'Unspecified',
        supplier_sku: comp.supplier_sku || '',
        unit_cost: comp.unit_cost || 0,
        estimated_line_cost: Math.round(qtyToOrder * (comp.unit_cost || 0) * 100) / 100,
        projects: demandEntry ? demandEntry.projects : []
      });
    }
  }

  return {
    linked_items: linkedShoppingItems,
    unlinked_items: unlinkedShortages
  };
}

module.exports = {
  db,
  normalizeElectronicValue,
  normalizeFootprint,
  findBestComponentMatch,
  getAllComponents,
  getComponentById,
  getAllProjects,
  getProjectDetail,
  getShoppingList
};
