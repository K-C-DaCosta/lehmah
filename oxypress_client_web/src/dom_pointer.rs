pub use oxypress_core::DomPointer;
use web_sys::{
    Document, Element, Node, console,
    wasm_bindgen::{JsCast, JsValue},
};

pub trait DomAddressable {
    fn addr_of(node: &web_sys::Node) -> DomPointer;
    fn resolve(&self) -> Option<web_sys::Node>;
}

impl DomAddressable for DomPointer {
    fn addr_of(n: &web_sys::Node) -> DomPointer {
        let mut cur = n.clone();
        let mut pointer = DomPointer::new();

        while cur.node_name() != "HTML" {
            let parent = cur.parent_node().unwrap();
            let child_nodes = parent.child_nodes();

            for k in 0..child_nodes.length() {
                let parents_child = child_nodes.get(k).unwrap();
                if parents_child.is_equal_node(Some(&cur)) {
                    pointer.links.push(k);
                    break;
                }
            }

            cur = parent;
        }

        pointer
    }

    fn resolve(&self) -> Option<web_sys::Node> {
        let mut node = web_sys::window()
            .and_then(|w| w.document())
            .and_then(|doc| doc.document_element())
            .and_then(|doc_elem| doc_elem.dyn_into::<web_sys::Node>().ok());
        for k in self.links.iter().rev().copied() {
            node = node.and_then(|n| n.child_nodes().get(k));
        }
        node
    }
}
