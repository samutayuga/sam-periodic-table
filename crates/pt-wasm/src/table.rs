use crate::element::WasmElement;
use pt_services::PeriodicTable as ServiceTable;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct PeriodicTable(ServiceTable);

#[wasm_bindgen]
impl PeriodicTable {
    /// Loads all 118 elements from bundled data. Synchronous.
    pub fn load() -> Result<PeriodicTable, JsError> {
        ServiceTable::load_bundled()
            .map(PeriodicTable)
            .map_err(|e| JsError::new(&e.to_string()))
    }

    pub fn by_symbol(&self, symbol: &str) -> Option<WasmElement> {
        self.0.by_symbol(symbol).map(WasmElement::from)
    }

    pub fn by_name(&self, name: &str) -> Option<WasmElement> {
        self.0.by_name(name).map(WasmElement::from)
    }

    pub fn by_atomic_number(&self, z: u8) -> Option<WasmElement> {
        self.0.by_atomic_number(z).map(WasmElement::from)
    }

    pub fn by_atomic_mass(&self, mass: f64, tolerance: f64) -> Option<WasmElement> {
        self.0.by_atomic_mass(mass, tolerance).map(WasmElement::from)
    }

    /// Returns all 118 elements as a JS array (serialised via serde-wasm-bindgen).
    pub fn all(&self) -> Result<JsValue, JsError> {
        let v: Vec<WasmElement> = self.0.all().map(WasmElement::from).collect();
        serde_wasm_bindgen::to_value(&v).map_err(|e| JsError::new(&e.to_string()))
    }

    /// Returns "Solid", "Liquid", or "Gas" for the given symbol at temperature_k.
    /// Returns undefined if symbol unknown or melting/boiling points absent.
    pub fn state_at(&self, symbol: &str, temperature_k: f64) -> Option<String> {
        let view = self.0.by_symbol(symbol)?;
        view.state_at(temperature_k).map(|s| format!("{s:?}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wasm_bindgen_test::*;

    #[wasm_bindgen_test]
    fn loads_all_elements() {
        let pt = PeriodicTable::load().unwrap();
        let all = pt.all().unwrap();
        assert!(all.is_array());
        assert_eq!(js_sys::Array::from(&all).length(), 118);
    }

    #[wasm_bindgen_test]
    fn by_symbol_returns_iron() {
        let pt = PeriodicTable::load().unwrap();
        let fe = pt.by_symbol("Fe").unwrap();
        assert_eq!(fe.atomic_number, 26);
        assert_eq!(fe.name, "Iron");
        assert_eq!(fe.group, 8);
        assert_eq!(fe.block, "D");
    }

    #[wasm_bindgen_test]
    fn by_atomic_number_returns_gold() {
        let pt = PeriodicTable::load().unwrap();
        let au = pt.by_atomic_number(79).unwrap();
        assert_eq!(au.symbol, "Au");
    }

    #[wasm_bindgen_test]
    fn by_name_case_insensitive() {
        let pt = PeriodicTable::load().unwrap();
        let h = pt.by_name("hydrogen").unwrap();
        assert_eq!(h.atomic_number, 1);
    }

    #[wasm_bindgen_test]
    fn by_atomic_mass_finds_nearest() {
        let pt = PeriodicTable::load().unwrap();
        let fe = pt.by_atomic_mass(55.845, 0.1).unwrap();
        assert_eq!(fe.symbol, "Fe");
    }

    #[wasm_bindgen_test]
    fn state_at_returns_liquid_for_iron_above_melting() {
        let pt = PeriodicTable::load().unwrap();
        let state = pt.state_at("Fe", 1900.0).unwrap();
        assert_eq!(state, "Liquid");
    }

    #[wasm_bindgen_test]
    fn unknown_symbol_returns_none() {
        let pt = PeriodicTable::load().unwrap();
        assert!(pt.by_symbol("XX").is_none());
    }
}
