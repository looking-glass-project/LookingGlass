use std::time::Duration;

use crate::traits::{DeviceTrait, HostTrait, StreamTrait};
use crate::{
    BuildStreamError, Data, DefaultStreamConfigError, DeviceNameError, DevicesError,
    InputCallbackInfo, OutputCallbackInfo, PauseStreamError, PlayStreamError, SampleFormat,
    StreamConfig, StreamError, SupportedStreamConfig, SupportedStreamConfigRange,
    SupportedStreamConfigsError,
};

#[derive(Default)]
pub struct Devices;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Device;

pub struct Host;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Stream;

pub struct SupportedInputConfigs;
pub struct SupportedOutputConfigs(Option<SupportedStreamConfigRange>);

// On the PS5 (target_os = "freebsd") this host is the console's audio out: one 48 kHz stereo
// i16 output driven by PS5_VulkanTemplate's platform.c (`audio_start` runs its own thread and
// calls back for 256 frames at a time). Elsewhere it stays cpal's null host.
#[cfg(target_os = "freebsd")]
mod ps5 {
    use super::*;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Mutex;

    pub const RATE: u32 = 48000;
    pub const FRAMES: u32 = 256;

    extern "C" {
        fn audio_start(fill: extern "C" fn(*mut i16, i32, *mut std::ffi::c_void), user: *mut std::ffi::c_void) -> bool;
        fn audio_stop();
    }

    type Callback = Box<dyn FnMut(&mut Data, &OutputCallbackInfo) + Send + 'static>;
    static RUNNING: AtomicBool = AtomicBool::new(false);
    static CALLBACK: Mutex<Option<Callback>> = Mutex::new(None);

    extern "C" fn fill(frames: *mut i16, count: i32, _user: *mut std::ffi::c_void) {
        let samples = count.max(0) as usize * 2;
        let mut guard = CALLBACK.lock().unwrap();
        match guard.as_mut() {
            Some(cb) => {
                let mut data = unsafe { Data::from_parts(frames as *mut (), samples, SampleFormat::I16) };
                let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default();
                let at = crate::StreamInstant::new(now.as_secs() as i64, now.subsec_nanos());
                let info = OutputCallbackInfo { timestamp: crate::OutputStreamTimestamp { callback: at, playback: at } };
                cb(&mut data, &info);
            }
            None => unsafe { std::ptr::write_bytes(frames, 0, samples) },
        }
    }

    pub fn start(cb: Callback) -> Result<(), BuildStreamError> {
        *CALLBACK.lock().unwrap() = Some(cb);
        if RUNNING.swap(true, Ordering::SeqCst) { return Ok(()); }
        if unsafe { audio_start(fill, std::ptr::null_mut()) } {
            Ok(())
        } else {
            RUNNING.store(false, Ordering::SeqCst);
            *CALLBACK.lock().unwrap() = None;
            Err(BuildStreamError::DeviceNotAvailable)
        }
    }

    pub fn stop() {
        if RUNNING.swap(false, Ordering::SeqCst) { unsafe { audio_stop() } }
        *CALLBACK.lock().unwrap() = None;
    }

    pub fn range() -> SupportedStreamConfigRange {
        SupportedStreamConfigRange::new(2, crate::SampleRate(RATE), crate::SampleRate(RATE),
            crate::SupportedBufferSize::Range { min: FRAMES, max: FRAMES }, SampleFormat::I16)
    }
}

#[cfg(target_os = "freebsd")]
impl Drop for Stream {
    fn drop(&mut self) { ps5::stop() }
}

impl Host {
    #[allow(dead_code)]
    pub fn new() -> Result<Self, crate::HostUnavailable> {
        Ok(Host)
    }
}

impl Devices {
    pub fn new() -> Result<Self, DevicesError> {
        Ok(Devices)
    }
}

impl DeviceTrait for Device {
    type SupportedInputConfigs = SupportedInputConfigs;
    type SupportedOutputConfigs = SupportedOutputConfigs;
    type Stream = Stream;

    #[inline]
    fn name(&self) -> Result<String, DeviceNameError> {
        Ok(if cfg!(target_os = "freebsd") { "PS5 audio out" } else { "null" }.to_owned())
    }

    #[inline]
    fn supported_input_configs(
        &self,
    ) -> Result<SupportedInputConfigs, SupportedStreamConfigsError> {
        unimplemented!()
    }

    #[inline]
    fn supported_output_configs(
        &self,
    ) -> Result<SupportedOutputConfigs, SupportedStreamConfigsError> {
        #[cfg(target_os = "freebsd")]
        return Ok(SupportedOutputConfigs(Some(ps5::range())));
        #[cfg(not(target_os = "freebsd"))]
        unimplemented!()
    }

    #[inline]
    fn default_input_config(&self) -> Result<SupportedStreamConfig, DefaultStreamConfigError> {
        unimplemented!()
    }

    #[inline]
    fn default_output_config(&self) -> Result<SupportedStreamConfig, DefaultStreamConfigError> {
        #[cfg(target_os = "freebsd")]
        return Ok(ps5::range().with_max_sample_rate());
        #[cfg(not(target_os = "freebsd"))]
        unimplemented!()
    }

    fn build_input_stream_raw<D, E>(
        &self,
        _config: &StreamConfig,
        _sample_format: SampleFormat,
        _data_callback: D,
        _error_callback: E,
        _timeout: Option<Duration>,
    ) -> Result<Self::Stream, BuildStreamError>
    where
        D: FnMut(&Data, &InputCallbackInfo) + Send + 'static,
        E: FnMut(StreamError) + Send + 'static,
    {
        unimplemented!()
    }

    /// Create an output stream.
    fn build_output_stream_raw<D, E>(
        &self,
        _config: &StreamConfig,
        _sample_format: SampleFormat,
        _data_callback: D,
        _error_callback: E,
        _timeout: Option<Duration>,
    ) -> Result<Self::Stream, BuildStreamError>
    where
        D: FnMut(&mut Data, &OutputCallbackInfo) + Send + 'static,
        E: FnMut(StreamError) + Send + 'static,
    {
        #[cfg(target_os = "freebsd")]
        {
            if _sample_format != SampleFormat::I16 || _config.channels != 2 || _config.sample_rate.0 != ps5::RATE {
                return Err(BuildStreamError::StreamConfigNotSupported);
            }
            ps5::start(Box::new(_data_callback))?;
            return Ok(Stream);
        }
        #[cfg(not(target_os = "freebsd"))]
        unimplemented!()
    }
}

impl HostTrait for Host {
    type Devices = Devices;
    type Device = Device;

    fn is_available() -> bool {
        cfg!(target_os = "freebsd")
    }

    fn devices(&self) -> Result<Self::Devices, DevicesError> {
        Devices::new()
    }

    fn default_input_device(&self) -> Option<Device> {
        None
    }

    fn default_output_device(&self) -> Option<Device> {
        if cfg!(target_os = "freebsd") { Some(Device) } else { None }
    }
}

impl StreamTrait for Stream {
    fn play(&self) -> Result<(), PlayStreamError> {
        Ok(())
    }

    fn pause(&self) -> Result<(), PauseStreamError> {
        Ok(())
    }
}

impl Iterator for Devices {
    type Item = Device;

    #[inline]
    fn next(&mut self) -> Option<Device> {
        None
    }
}

impl Iterator for SupportedInputConfigs {
    type Item = SupportedStreamConfigRange;

    #[inline]
    fn next(&mut self) -> Option<SupportedStreamConfigRange> {
        None
    }
}

impl Iterator for SupportedOutputConfigs {
    type Item = SupportedStreamConfigRange;

    #[inline]
    fn next(&mut self) -> Option<SupportedStreamConfigRange> {
        self.0.take()
    }
}
