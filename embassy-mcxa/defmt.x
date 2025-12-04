/*
 Dummy defmt.x to satisfy -Tdefmt.x when building without the `defmt` feature.
 When `defmt` is enabled, the real defmt.x provided by the defmt crate is used via the linker search path.
 This file intentionally contains no SECTIONS so it won't override ram.ld.
*/

