use cpal::{
    traits::{DeviceTrait, HostTrait},
    BuildStreamError, DefaultStreamConfigError, Device, DeviceNameError, Sample, Stream,
    StreamConfig, SupportedStreamConfig,
};
use thiserror::Error;
use tracing::debug;

pub struct StreamFactory {
    device: Device,
    config: SupportedStreamConfig,
}

impl StreamFactory {
    pub fn new() -> Result<Self, AudioError> {
        let host = cpal::default_host();
        let device = host
            .default_output_device()
            .ok_or(AudioError::DeviceError)?;
        debug!("Output device : {}", device.name()?);
        let config = device.default_output_config()?;
        debug!("Default output config : {:?}", config);
        Ok(Self { device, config })
    }

    pub fn generate_stream<F, T>(&self, f: F) -> Result<Stream, AudioError>
    where
        F: Fn(&SampleRequestOptions) -> T + Send + 'static,
        T: Sample,
    {
        match self.config.sample_format() {
            cpal::SampleFormat::I16 => self.make_stream::<F, T>(f),
            cpal::SampleFormat::U16 => self.make_stream::<F, T>(f),
            cpal::SampleFormat::F32 => self.make_stream::<F, T>(f),
        }
    }

    fn make_stream<F, T>(&self, f: F) -> Result<Stream, AudioError>
    where
        F: Fn(&SampleRequestOptions) -> T + Send + 'static,
        T: Sample,
    {
        let config: StreamConfig = self.config.clone().into();
        let sample_rate = config.sample_rate.0 as f32;
        let sample_clock = 0f32;
        let nchannels = config.channels as usize;
        let mut request = SampleRequestOptions {
            sample_rate,
            sample_clock,
        };
        let err_fn = |err| eprintln!("Error building output sound stream: {}", err);
        let stream = self.device.build_output_stream(
            &config,
            move |output: &mut [T], _: &cpal::OutputCallbackInfo| {
                for frame in output.chunks_mut(nchannels) {
                    request.tick();
                    let s = f(&request);
                    let value = Sample::from::<T>(&s);
                    for sample in frame.iter_mut() {
                        *sample = value;
                    }
                }
            },
            err_fn,
        )?;
        Ok(stream)
    }
}

pub struct SampleRequestOptions {
    pub sample_rate: f32,
    pub sample_clock: f32,
}

impl SampleRequestOptions {
    fn tick(&mut self) {
        self.sample_clock = (self.sample_clock + 1.0) % self.sample_rate;
    }
}

#[derive(Error, Debug)]
pub enum AudioError {
    #[error("default output device is not available")]
    DeviceError,

    #[error(transparent)]
    DeviceNameError(#[from] DeviceNameError),

    #[error(transparent)]
    DefaultStreamConfigError(#[from] DefaultStreamConfigError),

    #[error(transparent)]
    BuildStreamError(#[from] BuildStreamError),
}
