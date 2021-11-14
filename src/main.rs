use paradocs::parse_document;
use scraper::Html;

fn main() {
    let slice = include_str!("../slice.html");
    let slice_html = Html::parse_document(slice);
    println!("{:#?}", parse_document(&slice_html));

    let tokio_time = include_str!("../tokio_time.html");
    let tokio_time_html = Html::parse_document(tokio_time);
    println!("{:#?}", parse_document(&tokio_time_html));

    let std_ptr_dyn_metadata = include_str!("../std_ptr_dyn_metadata.html");
    let std_ptr_dyn_metadata_html = Html::parse_document(std_ptr_dyn_metadata);
    println!("{:#?}", parse_document(&std_ptr_dyn_metadata_html));

    let teloxide_types_keyboard = include_str!("../teloxide_types_keyboard.html");
    let teloxide_types_keyboard_html = Html::parse_document(teloxide_types_keyboard);
    println!("{:#?}", parse_document(&teloxide_types_keyboard_html));
}
