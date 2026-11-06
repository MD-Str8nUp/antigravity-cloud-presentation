# Audio Electronics Inventory & Kit Builds

A personal, single-user web application for a hobbyist audio electronics builder to catalog electronic components across physical storage locations, manage DIY kit builds and their Bills of Materials (BOMs), check stock readiness, allocate/consume parts during builds, and generate shopping lists for missing components.

## Language

**Component**:
A distinct electronic part, sub-assembly, or hardware specification in the catalog defined by its category, original spec string, and structured parametric attributes (e.g., a `10kΩ 1% 1/4W Metal Film Through-Hole Resistor`, a `TL072IP DIP-8 Op-Amp`, or a `Daisy Seed MCU Board`), whose total quantity on hand is the sum of its **Stock Lots**.
_Avoid_: Item, SKU, Product, Material

**Storage Location**:
A physical container or sub-container in a self-referential nesting hierarchy (e.g., a `Crate` containing a `Bag` containing a smaller `Bag`, `Box`, `Binder`, or `Drawer`) that holds child **Storage Locations**, **Stock Lots**, or **Kit Builds** and can be moved as a unit.
_Avoid_: Bin, Warehouse, Shelf, Slot

**Stock Lot**:
A physical quantity of a specific **Component** stored at a single **Storage Location**, optionally carrying lot-specific supplier details or notes (such as when duplicate bags of the same part live in different containers).
_Avoid_: Stash, Balance, Inventory Record, Batch

**Kit Build**:
A single hobby build project (such as a guitar pedal, synth module, or amplifier kit) that tracks its lifecycle status, physical **Storage Location** (when stored in a project box), Bill of Materials (BOM), build documentation/schematics, and build/troubleshooting notes.
_Avoid_: Project Template, Work Order, Production Run, Assembly

**BOM (Bill of Materials)**:
The list of required parts and quantities needed to complete a specific **Kit Build**.
_Avoid_: Recipe, Parts List, Manifest

**BOM Line**:
A single entry on a **BOM** specifying a required **Component** (or component specification), the quantity required, circuit reference designators (e.g., `R1, R2, R5`), an optional substitution note, and its **Fulfillment Status** for that **Kit Build**.
_Avoid_: BOM Item, Line Item, Requirement

**Fulfillment Status**:
The sourcing and assembly state of a **BOM Line** within a **Kit Build** (`Unallocated`, `Reserved` in place against stock, `Picked` into the **Kit Build**'s physical container, `Installed` on the circuit board, or `Included in Kit` when supplied with a partial kit).
_Avoid_: Allocation State, Line State, Stock State

**Available Stock**:
The quantity of a **Component** free to be assigned to new **Kit Builds**, equal to the sum of its **Stock Lots** minus any quantity currently `Reserved` (in place) across active **Kit Builds**.
_Avoid_: Free Inventory, Uncommitted Balance, Net Stock

**Bench Log Entry**:
A timestamped note, voltage/signal measurement, circuit modification, or troubleshooting observation recorded against a **Kit Build** during assembly or debugging.
_Avoid_: Comment, Audit Log, Ticket Note

**Shopping List**:
A live computed aggregation of unfulfilled **BOM Line** demand across one or more selected **Kit Builds** (and optional low-stock **Components**), grouped by catalog **Component** or normalized specification to calculate **Net Shortfall** and export supplier order files.
_Avoid_: Purchase Order, Requisition, Cart

**Net Shortfall**:
The additional quantity of a **Component** (or unlinked specification) that must be purchased across a selected set of **Kit Builds**, equal to the total unallocated quantity required across those builds minus current **Available Stock** (floored at zero).
_Avoid_: Deficit, Backorder Quantity, Negative Balance
