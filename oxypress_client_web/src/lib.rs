use oxypress_core::DomPointer;
use web_sys::{
    Document, Element, console,
    wasm_bindgen::{JsCast, JsValue},
};


pub trait DomAddressable {
    fn addr_of(elem: web_sys::Element) -> DomPointer;
    fn deref(&self) -> Option<web_sys::Node>;
}



impl DomAddressable for DomPointer {
    fn addr_of(elem: web_sys::Element) -> DomPointer {
        let mut cur = elem.clone();
        let mut pointer = DomPointer::new();
        while cur.node_name() != "HTML" {
            let parent = cur
                .parent_node()
                .and_then(|p| p.dyn_into::<Element>().ok())
                .unwrap();
            let child_nodes = parent.child_nodes();

            // console_log(format!("parent= {} ", parent.node_name()));
            // console_log(format!("child= {}", cur.node_name()));

            for k in 0..child_nodes.length() {
                let parents_child = child_nodes.get(k);
                // console_log(format!(
                //     "parent {}, child[{}] = {}, equal? = {}",
                //     parent.node_name(),
                //     k,
                //     cur.node_name(),
                //     parent.is_equal_node(child.as_ref())
                // ));
                if parents_child
                    .unwrap()
                    .is_equal_node(cur.clone().dyn_into::<web_sys::Node>().ok().as_ref())
                {
                    // console_log(format!("idx = {}",k));
                    pointer.links.push(k);
                    break;
                }
            }
            cur = parent;
        }

        pointer
    }

    fn deref(&self) -> Option<web_sys::Node> {
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