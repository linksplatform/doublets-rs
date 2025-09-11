use std::collections::HashMap;
use wasm_bindgen::prelude::*;
use serde::{Deserialize, Serialize};

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

// This is like the `extern` block in C.
#[wasm_bindgen]
extern "C" {
    // Bind the `console.log` function from the browser.
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

// Define a macro to make it easier to call `console.log`.
macro_rules! console_log {
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LinkData {
    pub id: usize,
    pub source: usize,
    pub target: usize,
}

#[derive(Serialize, Deserialize)]
pub struct DoubletsState {
    pub links: Vec<LinkData>,
    pub count: usize,
    pub operations: Vec<String>,
}

/// A simplified doublets store for demonstration purposes
/// This implements the core concept of doublets without requiring
/// the full complexity of the main doublets library
#[wasm_bindgen]
pub struct DoubletsDemo {
    links: HashMap<usize, LinkData>,
    next_id: usize,
    operations_log: Vec<String>,
}

#[wasm_bindgen]
impl DoubletsDemo {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Result<DoubletsDemo, JsValue> {
        console_error_panic_hook::set_once();
        
        let mut demo = DoubletsDemo {
            links: HashMap::new(),
            next_id: 1,
            operations_log: Vec::new(),
        };
        
        demo.log_operation("Initialized doublets store (simplified demo version)".to_string());
        console_log!("Doublets demo initialized with simplified store");
        Ok(demo)
    }

    #[wasm_bindgen]
    pub fn create_link(&mut self, source: usize, target: usize) -> Result<usize, JsValue> {
        if source == 0 || target == 0 {
            return Err(JsValue::from_str("Source and target must be > 0"));
        }

        let link_id = self.next_id;
        let link = LinkData {
            id: link_id,
            source,
            target,
        };
        
        self.links.insert(link_id, link);
        self.next_id += 1;
        
        self.log_operation(format!("Created link {}: {} -> {}", link_id, source, target));
        console_log!("Created link {}: {} -> {}", link_id, source, target);
        Ok(link_id)
    }

    #[wasm_bindgen]
    pub fn create_point(&mut self) -> Result<usize, JsValue> {
        // Create a point: a link that points to itself
        // We'll use the next available ID as both the link ID and the point value
        let point_value = self.next_id;
        let link_id = self.create_link(point_value, point_value)?;
        
        self.log_operation(format!("Created point: {} (self-referencing link)", point_value));
        console_log!("Created point: {}", point_value);
        Ok(link_id)
    }

    #[wasm_bindgen]
    pub fn delete_link(&mut self, link_id: usize) -> Result<bool, JsValue> {
        if let Some(link) = self.links.remove(&link_id) {
            self.log_operation(format!("Deleted link {}: {} -> {}", link_id, link.source, link.target));
            console_log!("Deleted link {}: {} -> {}", link_id, link.source, link.target);
            Ok(true)
        } else {
            self.log_operation(format!("Link {} not found for deletion", link_id));
            console_log!("Link {} not found", link_id);
            Ok(false)
        }
    }

    #[wasm_bindgen]
    pub fn get_link_count(&self) -> usize {
        self.links.len()
    }

    #[wasm_bindgen]
    pub fn get_all_links(&self) -> JsValue {
        let mut links: Vec<&LinkData> = self.links.values().collect();
        links.sort_by_key(|link| link.id);
        
        serde_wasm_bindgen::to_value(&links).unwrap_or(JsValue::NULL)
    }

    #[wasm_bindgen]
    pub fn get_state(&self) -> JsValue {
        let mut links: Vec<&LinkData> = self.links.values().collect();
        links.sort_by_key(|link| link.id);
        
        let state = DoubletsState {
            links: links.into_iter().cloned().collect(),
            count: self.links.len(),
            operations: self.operations_log.clone(),
        };
        
        serde_wasm_bindgen::to_value(&state).unwrap_or(JsValue::NULL)
    }

    #[wasm_bindgen]
    pub fn clear_operations_log(&mut self) {
        self.operations_log.clear();
        console_log!("Operations log cleared");
    }

    #[wasm_bindgen]
    pub fn search_links(&self, source: Option<usize>, target: Option<usize>) -> JsValue {
        let filtered_links: Vec<&LinkData> = self.links
            .values()
            .filter(|link| {
                let source_match = source.map_or(true, |s| link.source == s);
                let target_match = target.map_or(true, |t| link.target == t);
                source_match && target_match
            })
            .collect();
        
        console_log!("Search found {} links", filtered_links.len());
        serde_wasm_bindgen::to_value(&filtered_links).unwrap_or(JsValue::NULL)
    }

    #[wasm_bindgen]
    pub fn get_link(&self, link_id: usize) -> JsValue {
        if let Some(link) = self.links.get(&link_id) {
            serde_wasm_bindgen::to_value(link).unwrap_or(JsValue::NULL)
        } else {
            JsValue::NULL
        }
    }

    #[wasm_bindgen]
    pub fn update_link(&mut self, link_id: usize, new_source: usize, new_target: usize) -> Result<bool, JsValue> {
        if new_source == 0 || new_target == 0 {
            return Err(JsValue::from_str("Source and target must be > 0"));
        }

        if let Some(link) = self.links.get_mut(&link_id) {
            let old_source = link.source;
            let old_target = link.target;
            link.source = new_source;
            link.target = new_target;
            
            self.log_operation(format!(
                "Updated link {}: {} -> {} (was {} -> {})", 
                link_id, new_source, new_target, old_source, old_target
            ));
            console_log!("Updated link {}: {} -> {}", link_id, new_source, new_target);
            Ok(true)
        } else {
            self.log_operation(format!("Link {} not found for update", link_id));
            console_log!("Link {} not found", link_id);
            Ok(false)
        }
    }

    #[wasm_bindgen]
    pub fn clear_all_links(&mut self) {
        let count = self.links.len();
        self.links.clear();
        self.next_id = 1;
        
        self.log_operation(format!("Cleared all {} links from store", count));
        console_log!("Cleared all {} links", count);
    }

    fn log_operation(&mut self, operation: String) {
        self.operations_log.push(operation);
    }
}

// Export a `greet` function from Rust to JavaScript, that alerts a greeting.
#[wasm_bindgen]
pub fn greet(name: &str) {
    console_log!("Hello, {}! This is the Doublets WebAssembly demo.", name);
}

// Called when the wasm module is instantiated
#[wasm_bindgen(start)]
pub fn main() {
    console_log!("Doublets WebAssembly demo loaded!");
}

#[wasm_bindgen]
pub fn get_demo_info() -> JsValue {
    let info = serde_json::json!({
        "name": "Doublets WebAssembly Demo",
        "version": "0.1.0",
        "description": "A simplified demonstration of doublets operations using WebAssembly",
        "note": "This is a simplified version for demonstration purposes. The full doublets-rs library provides more advanced features."
    });
    
    serde_wasm_bindgen::to_value(&info).unwrap_or(JsValue::NULL)
}