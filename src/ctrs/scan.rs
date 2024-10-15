use std::{fmt::Debug, io::{self, Cursor}, path::{Path, PathBuf}};

use futures::future;
use image::{ImageBuffer, ImageReader, Luma};
use serde::Deserialize;
use tokio::task;

pub type ScanImage = ImageBuffer<Luma<f32>, Vec<f32>>;

#[derive(Deserialize, Debug, Clone)]
pub enum RotationDirection {
    // Both directions are looking down from above
    CW,  // Clockwise
    CCW, // Counterclockwise
}

impl RotationDirection {
    pub fn dir(&self) -> f32 {
        match self {
            RotationDirection::CW => -1.,
            RotationDirection::CCW => 1.,
        }
    }
}

#[derive(Deserialize, Clone)]
pub struct CtScan {
    pub name: String,
    pub direction: RotationDirection,
    pub sod: f32, // source-object-distance
    pub sdd: f32, // source-detector-distance
    pub swept_angle: f32,
    pub pixel_size: f32,

    #[serde(rename = "projections")]
    pub projection_files: Vec<PathBuf>,

    #[serde(skip)]
    pub projection_images: Vec<ScanImage>,
}

// implement Debug for ScanDescriptor but don't print images_files and images as this takes a _long_ time (especially for the latter)
impl Debug for CtScan {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ScanDescriptor")
            .field("name", &self.name)
            .field("direction", &self.direction)
            .field("sod", &self.sod)
            .field("sdd", &self.sdd)
            .field("swept_angle", &self.swept_angle)
            .field("pixel_size", &self.pixel_size)
            .field("projection_files", &"...")
            .field("images", &"...")
            .finish()
    }
}

impl CtScan {
    pub async fn from_file(path: impl Into<PathBuf>) -> io::Result<Self> {
        let path = path.into();

        let file_contents = tokio::fs::read(&path).await?;
        let mut parsed = Self {
            ..serde_json::from_slice(&file_contents)?
        };

        parsed.projection_images = Self::load_images(&path, parsed.projection_files.clone()).await?;

        Ok(parsed)
    }

    async fn load_images(file_path: impl AsRef<Path>, image_files: Vec<PathBuf>) -> io::Result<Vec<ScanImage>> {
        let images_dir = file_path.as_ref().parent().unwrap().join("projections");

        let full_paths = image_files.iter().map(|filename| images_dir.join(filename));

        let image_load_tasks = full_paths.map(|path| {
            task::spawn(async move {
                let bytes = tokio::fs::read(path).await.unwrap();

                ImageReader::new(Cursor::new(bytes))
                    .with_guessed_format()
                    .unwrap()
                    .decode()
                    .unwrap()
                    .to_luma32f()
            })
        });

        let images = future::try_join_all(image_load_tasks).await?;

        Ok(images)
    }
}


