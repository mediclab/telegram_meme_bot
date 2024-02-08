opencv::opencv_branch_4! {
    use opencv::{
        core::{Mat, Size},
        imgcodecs,
        imgcodecs::ImreadModes,
        imgproc,
        imgproc::InterpolationFlags,
        prelude::*,
    };
}

#[derive(Clone)]
pub struct ImageHash {
    image: Mat,
}

impl ImageHash {
    pub fn new(filename: &str) -> Self {
        Self {
            image: imgcodecs::imread(filename, ImreadModes::IMREAD_COLOR as i32).unwrap_or_default(),
        }
    }

    pub fn grayscale(mut self) -> Self {
        let mut gray = Mat::default();
        imgproc::cvt_color(&self.image, &mut gray, imgproc::COLOR_BGR2GRAY, 0).unwrap_or_default();

        self.image = gray;
        self
    }

    pub fn resize(mut self, hash_size: i32) -> Self {
        let mut resized = Mat::default();
        imgproc::resize(
            &self.image,
            &mut resized,
            Size::new(hash_size, hash_size),
            0.0,
            0.0,
            InterpolationFlags::INTER_AREA as i32,
        )
        .unwrap_or_default();

        self.image = resized;
        self
    }

    pub fn threshold(mut self) -> Self {
        let mean = opencv::core::mean(&self.image, &Mat::default()).expect("Can't mean");
        let mut t_image = Mat::default();
        imgproc::threshold(&self.image, &mut t_image, mean.0[0], 255.0, 0).unwrap_or_default();

        self.image = t_image;
        self
    }

    pub fn hash(&self) -> Option<String> {
        let a_image = self.image.to_vec_2d::<u8>();

        if a_image.is_err() {
            return None;
        }

        let hash = a_image
            .unwrap()
            .iter()
            .map(|row| {
                row.iter()
                    .map(|item| {
                        if *item == 255 {
                            String::from("1")
                        } else {
                            String::from("0")
                        }
                    })
                    .collect::<Vec<String>>()
                    .join("")
            })
            .collect::<Vec<String>>()
            .join("");

        Some(hash)
    }

    pub fn compare_hashes(hash1: &str, hash2: &str) -> f64 {
        let diffs_num = hash1.chars().zip(hash2.chars()).filter(|(c1, c2)| c1 != c2).count();

        ((hash1.len() - diffs_num) as f64 / hash1.len() as f64) * 100f64
    }
}
