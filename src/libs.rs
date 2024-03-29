use std::error::Error;
use std::fs;
use std::path::Path;
use serde::{Serialize, Deserialize};
use windows::Media::Ocr::{self};
use windows::Graphics::Imaging::{BitmapDecoder, SoftwareBitmap};
use windows::Globalization::Language;
use windows::Storage::{FileAccessMode, StorageFile};
use windows::core::{ HSTRING};


/// 
/// Contains the text and the x,y coordinates of the word
/// 
#[derive(Serialize,Deserialize,Debug)]
pub struct Coordinates {
    pub text:           String,
    pub x :             f64,
    pub y :             f64,
    pub height:         f64,
    pub width:          f64
}


/// 
/// Read the given path to .png file and return a Vec<Coordinates>
/// 
pub fn ocr_with_bounds<T: AsRef<Path>>(png :T, path_to_save:Option<T>) -> Result<Vec<Coordinates>, Box<dyn Error>> {
    let bitmap = open_image_as_bitmap(png)?;
    let map = ocr_from_bitmap_with_bounds(bitmap)?; 
    if let Some(path) = path_to_save {
        let file = serde_json::to_string_pretty(&serde_json::json!(&map)).unwrap();
        std::fs::write(path, file).unwrap();
    }
    Ok(map)
}


/// 
/// Read the given path to .png file and return the text
/// 
/// # Examples
/// 
/// ```
/// use rust_ocr::png_to_text;
/// let img = "image.png";
/// let text = png_to_text(img).unwrap();
/// assert_eq!("Read the given path to .png file and return the text".to_string(),text);
/// ```
/// 
pub fn png_to_text<T: AsRef<Path>>(png :T) -> Result<String, Box<dyn Error>> {
    let bitmap = open_image_as_bitmap(png)?;
    let result = ocr_from_bitmap(bitmap)?;
    Ok(result)
}

fn open_image_as_bitmap<T: AsRef<Path>>(path: T) -> windows::core::Result<SoftwareBitmap> {
    let path = fs::canonicalize(path);
    let path = match path {
        Ok(path) => HSTRING::from(path.to_string_lossy().replace("\\\\?\\", "")),
        Err(_) => {
            panic!();
        }
    };

    let file = StorageFile::GetFileFromPathAsync(&path)?.get()?;

    let a = file.OpenAsync(FileAccessMode::Read)?.get()?;
   

    let bitmap = BitmapDecoder::CreateWithIdAsync(
        BitmapDecoder::PngDecoderId()?,
        &a,
    )?
    .get()?;

    bitmap.GetSoftwareBitmapAsync()?.get()
}

fn ocr_from_bitmap_with_bounds(bitmap: SoftwareBitmap) -> windows::core::Result<Vec<Coordinates>> {
    let lang = &Ocr::OcrEngine::AvailableRecognizerLanguages()?
        .First()?
        .Current()?
        .LanguageTag()?;
    let lang = Language::CreateLanguage(lang)?;
    let engine = Ocr::OcrEngine::TryCreateFromLanguage(&lang)?;

    let result = engine
        .RecognizeAsync(&bitmap)?
        .get()?
        .Lines()?
        ;
    
    let mut collected_words:Vec<Coordinates> = Vec::new();    
    
    result.into_iter().for_each(|line|{
        let words = line.Words().unwrap();
        // get lines stuff
        let line_text = line.Text().unwrap().to_string_lossy();
        let mut _x: f64 = 0.0 as f64;
        let mut _y = 0.0 as f64;
        let mut line_heigth = 0.0 as f64;
        let mut line_width = 0.0 as f64;
        let mut idx = 0;
        words.into_iter().for_each(|word|{
            let rect = word.BoundingRect().unwrap();
            let name = &word.Text().unwrap().to_string_lossy();
            collected_words.push(
                Coordinates{
                    x:rect.X.into(), 
                    y: rect.Y.into(), 
                    text: name.to_string(),
                    height: rect.Height.into(),
                    width: rect.Width.into()
                }
            );
            if idx == 0{
                _x = rect.X as f64;
            }
            if line_heigth < rect.Height as f64 {
                line_heigth = rect.Height as f64;
            }
            line_width += rect.Width as f64;
            if _y < rect.Y as f64 {
                _y = rect.Y as f64;
            }
            idx +=1;
            
        });
        collected_words.push(
            Coordinates{
                x:_x, 
                y: _y, 
                text: line_text,
                height: line_heigth,
                width: line_width
            }
        )
    });

    Ok(collected_words)
}

fn ocr_from_bitmap(bitmap: SoftwareBitmap) -> windows::core::Result<String> {
    let lang = &Ocr::OcrEngine::AvailableRecognizerLanguages()?
        .First()?
        .Current()?
        .LanguageTag()?;
    let lang = Language::CreateLanguage(lang)?;
    let engine = Ocr::OcrEngine::TryCreateFromLanguage(&lang)?;

    let result = engine
        .RecognizeAsync(&bitmap)?
        .get()?
        .Text()?
        .to_string_lossy()        
        ;
    
   

    Ok(result)
}

