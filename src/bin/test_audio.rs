use std::time::Duration;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use ffmpeg_next::ChannelLayout;
use ffmpeg_next::codec::Id;
use ffmpeg_next::format::Sample;
use ffmpeg_next::format::sample::Type;
use ffmpeg_sys_next::AVSampleFormat;


fn main() {
    let mut codec = ffmpeg_next::codec::encoder::find(Id::MP3).unwrap();
    let mut context = unsafe {
        ffmpeg_next::codec::Context::wrap(ffmpeg_sys_next::avcodec_alloc_context3(codec.as_ptr()), None)
    };
    let mut audio = context.encoder().audio().unwrap();
    audio.set_rate(44100);
    audio.set_channel_layout(ChannelLayout::STEREO);
    audio.set_channels(ChannelLayout::STEREO.channels());
    println!("{:?}", ChannelLayout::STEREO.channels());
    // audio.set_channels(channel_layout.channels());
    audio.set_time_base((1, 32));
    audio.set_bit_rate(64000);
    audio.set_format(Sample::from(AVSampleFormat::AV_SAMPLE_FMT_FLTP));
    let encoder1 = audio.open().unwrap();

    let host = cpal::default_host();
    let output_device = host.default_output_device().unwrap();
    let config1 = output_device.default_output_config().unwrap();
    println!("{:?}", config1.channels());
    let config = config1.config();
    let stream = output_device.build_output_stream(
        &config,
        move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
            // react to stream events and read or write stream data here.
        },
        move |err| {
            // react to errors here.
        },
        None, // None=blocking, Some(Duration)=timeout
    ).unwrap();
    stream.play().unwrap();
    while true {
        std::thread::sleep(Duration::from_secs(30));
    }
}