use rubato::{
    Resampler, SincFixedIn, SincInterpolationParameters, SincInterpolationType, WindowFunction,
};

pub struct AudioResampler {
    resampler: SincFixedIn<f32>,
    input_channels: usize,
}

impl AudioResampler {
    pub fn new(from_rate: u32, to_rate: u32, channels: usize) -> Self {
        let params = SincInterpolationParameters {
            sinc_len: 256,
            f_cutoff: 0.95,
            interpolation: SincInterpolationType::Linear,
            oversampling_factor: 256,
            window: WindowFunction::BlackmanHarris2,
        };

        let resampler = SincFixedIn::new(
            to_rate as f64 / from_rate as f64,
            2.0,
            params,
            1024,
            channels,
        )
        .expect("Failed to create resampler");

        Self {
            resampler,
            input_channels: channels,
        }
    }

    pub fn resample(&mut self, samples: &[f32]) -> Vec<f32> {
        if samples.is_empty() {
            return Vec::new();
        }

        let input: Vec<Vec<f32>> = (0..self.input_channels)
            .map(|ch| {
                samples
                    .iter()
                    .skip(ch)
                    .step_by(self.input_channels)
                    .copied()
                    .collect()
            })
            .collect();

        let output = self
            .resampler
            .process(&input, None)
            .expect("Resampling failed");

        let total_len: usize = output.iter().map(|c| c.len()).sum();
        let mut result = vec![0.0f32; total_len];

        for (i, channel) in output.iter().enumerate() {
            for (j, &sample) in channel.iter().enumerate() {
                result[j * self.input_channels + i] = sample;
            }
        }

        result
    }

    pub fn stereo_to_mono(stereo: &[f32]) -> Vec<f32> {
        stereo
            .chunks_exact(2)
            .map(|chunk| (chunk[0] + chunk[1]) / 2.0)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stereo_to_mono() {
        let stereo = vec![0.4f32, 0.6, 0.2, 0.8];
        let mono = AudioResampler::stereo_to_mono(&stereo);
        assert_eq!(mono, vec![0.5, 0.5]);
    }

    #[test]
    fn test_stereo_to_mono_empty() {
        let stereo: Vec<f32> = vec![];
        let mono = AudioResampler::stereo_to_mono(&stereo);
        assert!(mono.is_empty());
    }

    #[test]
    fn test_resample_produces_output() {
        let mut resampler = AudioResampler::new(48000, 16000, 1);
        let samples: Vec<f32> = (0..4800).map(|i| (i as f32 * 0.01).sin()).collect();
        let output = resampler.resample(&samples);
        // Verify downsampling happened (output shorter than input)
        assert!(
            output.len() < samples.len(),
            "Expected downsampled output < {} samples, got {}",
            samples.len(),
            output.len()
        );
        assert!(output.len() > 0, "Expected non-empty output");
    }
}
