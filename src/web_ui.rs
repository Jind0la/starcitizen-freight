//! Embedded web UI assets (compiled into binary via rust-embed)

use rust_embed::Embed;

#[derive(Embed)]
#[folder = "src/web/"]
#[exclude(remove = "*.map")]
struct Assets;

pub fn get(name: &str) -> Option<rust_embed::EmbeddedFile> {
    Assets::get(name)
}
