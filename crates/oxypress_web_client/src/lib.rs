cfg_if::cfg_if! {
    if #[cfg(target_arch = "wasm32")] {
        pub mod dom_pointer;
        pub mod entry_point;
    }else{
    }
}

