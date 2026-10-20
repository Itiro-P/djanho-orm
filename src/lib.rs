//! Interface principal da biblioteca.
//!
//! Importe os derives daqui para marcar structs como models e enums como
//! conjuntos de opções do Djanho.

/// Derive que registra uma struct como model e seus campos como metadados.
/// Campos com `foreign_key` criam relações 1:N; `many_to_many` registra N:N.
///
/// O crate também exporta `DjanhoChoice` para enums usados como opções.
pub use djanho_macros::{Djanho, DjanhoChoice};
