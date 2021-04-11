use cache_buster::BusterBuilder;

fn main() {
    let types = vec![
        mime::IMAGE_PNG,
        mime::IMAGE_SVG,
        mime::IMAGE_JPEG,
        mime::IMAGE_GIF,
    ];

    let config = BusterBuilder::default()
        .source("../dist")
        .result("./prod")
        .mime_types(types)
        .copy(true)
        .follow_links(true)
        .build()
        .unwrap();

    config.process().unwrap();
}
