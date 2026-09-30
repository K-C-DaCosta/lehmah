fn main() {
    #[cfg(target_arch = "wasm32")]
    {
        yew::Renderer::<oxypress_web_client::entry_point::App>::new().render();
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        print!("Stubbed. This is a Yew project. Use Trunk to build");
    }
}
