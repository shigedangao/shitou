use ocrs::{ImageSource, OcrEngine, OcrEngineParams};
use regex::RegexSet;
use rten::Model;
use std::fs;
use std::{
    path::PathBuf,
    sync::mpsc::{Sender, channel},
    thread::JoinHandle,
};
use uuid::Uuid;

/// Ocr contains a set of method to interact with the OCR Model
pub struct Ocr {
    engine: OcrEngine,
    regex_set: RegexSet,
}

impl Ocr {
    /// Create a new OCR Model
    ///
    /// # Arguments
    ///
    /// * `detection_model_path` - PathBuf
    /// * `recgonition_model_path`- PathBuf
    /// * `indices` - Vec<S>
    pub fn new<S: AsRef<str>>(
        detection_model_path: PathBuf,
        recognition_model_path: PathBuf,
        indices: Vec<S>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let detection_model = Model::load_file(detection_model_path)?;
        let recognition_model = Model::load_file(recognition_model_path)?;

        let engine = OcrEngine::new(OcrEngineParams {
            detection_model: Some(detection_model),
            recognition_model: Some(recognition_model),
            ..Default::default()
        })?;

        let set = RegexSet::new(&indices)?;

        Ok(Self {
            engine,
            regex_set: set,
        })
    }

    /// Spawn a thread which process the image and find whether the name is available in the profile name
    pub fn start_process_image_loop(self) -> Result<Sender<PathBuf>, Box<dyn std::error::Error>> {
        let (path_tx, rx) = channel::<PathBuf>();

        let _: JoinHandle<Result<(), String>> = std::thread::spawn(move || {
            while let Ok(image_path) = rx.recv() {
                let text = self
                    .process_image(&image_path)
                    .expect("Processing image crash");

                let matches = self
                    .regex_set
                    .matches(&text)
                    .into_iter()
                    .collect::<Vec<_>>();

                // If there are no matches then we just delete the screenshot
                match matches.is_empty() {
                    true => {
                        println!("Removing no matches image");
                        fs::remove_file(&image_path).map_err(|err| err.to_string())?;
                    }
                    false => {
                        // Create a cropped image of the person that'll be passed to the face matching python script.
                        println!("Found matches with indices");
                        self.crop_image_and_store(&image_path)
                            .expect("Expect to crop image");
                    }
                }
            }

            Ok(())
        });

        Ok(path_tx)
    }

    /// Process the image and extract any text found from the image
    ///
    /// # Arguments
    ///
    /// * `image_path` - PathBuf
    pub fn process_image(
        &self,
        image_path: &PathBuf,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let img = image::open(image_path).map(|img| img.into_rgb8())?;
        let img_source = ImageSource::from_bytes(img.as_raw(), img.dimensions())?;

        let ocr_input = self.engine.prepare_input(img_source)?;

        let text = self.engine.get_text(&ocr_input)?;

        Ok(text)
    }

    /// Store the cropped image for face matching done in python
    ///
    /// # Arguments
    ///
    /// * `image_path` - &PathBuf
    fn crop_image_and_store(&self, image_path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
        let img = image::open(image_path)?;

        let mut cropped_img_path = PathBuf::from("./output/cropped");
        match image_path.file_name() {
            Some(fname) => cropped_img_path.push(fname),
            None => cropped_img_path.push(format!("{}.png", Uuid::new_v4())),
        };

        let cropped_img = image::imageops::crop_imm(&img, 50, 480, 1000, 1000);
        cropped_img.to_image().save(cropped_img_path)?;

        Ok(())
    }
}
