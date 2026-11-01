pub mod error;
pub mod library;
pub mod schematic;
pub mod sexp;
pub mod types;

pub use error::{Error, Result};
pub use schematic::label::{
    GlobalLabel, GlobalLabelCollection, HierarchicalLabel, HierarchicalLabelCollection, Label,
    LabelCollection,
};
pub use schematic::misc::{Junction, NoConnect, Text};
pub use schematic::sheet::{Sheet, SheetCollection, SheetInstance, SheetPin};
pub use schematic::symbol::{kicad_orientation, Symbol, SymbolCollection};
pub use schematic::wire::{Wire, WireCollection};
pub use schematic::{LocatedElement, Schematic};
pub use types::{At, ChangeSet, Property};

/// Keep this crate's unit tests out of the developer's home (see
/// `copperpour-test-home`).
#[cfg(test)]
#[ctor::ctor(unsafe)]
fn isolate_test_home() {
    copperpour_test_home::isolate();
}
