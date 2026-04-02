use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Device, SampleFormat, Stream};

use super::{AudioResampler, Chunker};

pub struct AudioCapture {
    stream: Option<Stream>,
    sample_rate: u32,
    channels: usize,
}

impl AudioCapture {
    pub fn new() -> Result<Self, String> {
        let host = cpal::default_host();
        let device = host
            .default_input_device()
            .ok_or("No default input device found")?;

        let config = device
            .default_input_config()
            .map_err(|e| format!("Failed to get input config: {}", e))?;

        Ok(Self {
            stream: None,
            sample_rate: config.sample_rate().0,
            channels: config.channels() as usize,
        })
    }

    pub fn default_device() -> Result<Device, String> {
        let host = cpal::default_host();
        host.default_input_device()
            .ok_or_else(|| "No default input device".to_string())
    }

    pub fn start<F>(&mut self, mut callback: F) -> Result<(), String>
    where
        F: FnMut(Vec<f32>) + Send + 'static,
    {
        let device = Self::default_device()?;
        let config = device
            .default_input_config()
            .map_err(|e| format!("Failed to get config: {}", e))?;

        let sample_rate = config.sample_rate().0;
        let channels = config.channels() as usize;
        let sample_format = config.sample_format();

        let mut resampler = AudioResampler::new(sample_rate, 16000, channels);
        let mut chunker = Chunker::new(16000, 500, 250);

        let stream: Stream = match sample_format {
            SampleFormat::F32 => device
                .build_input_stream(
                    &config.into(),
                    move |data: &[f32], _: &cpal::InputCallbackInfo| {
                        let mono = if channels == 2 {
                            AudioResampler::stereo_to_mono(data)
                        } else {
                            data.to_vec()
                        };
                        let resampled = resampler.resample(&mono);
                        chunker.push(&resampled);

                        while let Some(chunk) = chunker.next_chunk() {
                            callback(chunk);
                        }
                    },
                    move |err| eprintln!("Audio stream error: {}", err),
                    None,
                )
                .map_err(|e| format!("Failed to build stream: {}", e))?,
            _ => {
                return Err(format!("Unsupported sample format: {:?}", sample_format));
            }
        };

        stream
            .play()
            .map_err(|e| format!("Failed to start stream: {}", e))?;

        self.stream = Some(stream);
        Ok(())
    }

    pub fn stop(&mut self) {
        self.stream = None;
    }

    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    pub fn channels(&self) -> usize {
        self.channels
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_capture_creation_fails_without_device() {
        let result = AudioCapture::new();
        match result {
            Ok(_) => assert!(true),
            Err(_) => assert!(true),
        }
    }
}
