pub mod libs;
fn main() {

    let img = "image.png";
    let image_path = "./list.json";

    libs::ocr_with_bounds(img, Some(image_path)).unwrap();

    let text = libs::png_to_text(img).unwrap();

    println!("Text: {}",&text);
    assert_eq!("Read the given path to .png file and return the text".to_string(),text);

}
