mod editor;

fn main() {
    lyrebird_renderer::run::<editor::Editor>().unwrap();
}