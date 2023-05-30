extern crate scrap;

use std::ffi::c_char;
use std::fs::File;
use std::ptr;
use std::time::Duration;
use ffmpeg_next::{format, Frame, Packet};
use ffmpeg_next::decoder::Video;
use ffmpeg_next::encoder::Encoder;
use ratelimit_meter::{DirectRateLimiter, GCRA};
use ratelimit_rs::Bucket;


pub mod commons;

fn main() {
    use scrap::{Capturer, Display};
    use std::io::Write;
    use std::io::ErrorKind::WouldBlock;
    use std::process::{Command, Stdio};
    let d = Display::primary().unwrap();
    let (w, h) = (d.width() , d.height() );

    let ww = 4096;
    let hh = 2160;
    // let child = Command::new("ffplay")
    //     .args(&[
    //         "-f", "rawvideo",
    //         "-pixel_format", "bgr0",
    //         "-video_size", &format!("{}x{}", w, h),
    //         "-"
    //     ])
    //     .stdin(Stdio::piped())
    //     .spawn()
    //     .expect("This example requires ffplay.");

    // let mut out = child.stdin.unwrap();
    let mut capturer = Capturer::new(d).unwrap();
    let mut bucket = Bucket::new(Duration::from_millis(1000 / 30), 5, 1, 5);

    let mut file = File::create("test.h265").unwrap();

    // let encode_ctx = EncodeContext {
    //     name: String::from("h264_mf"),
    //     width: 1920,
    //     height: 1080,
    //     pixfmt: AVPixelFormat::AV_PIX_FMT_YUV420P,
    //     align: 0,
    //     bitrate: 0,
    //     timebase: [1, 30],
    //     gop: 60,
    //     quality: Quality_Default,
    //     rc: RC_DEFAULT,
    // };
    // let encoder = Encoder::new(encode_ctx).unwrap();


    let pixel_format = ffmpeg_sys_next::AVPixelFormat::AV_PIX_FMT_YUV420P;
    unsafe {
        let mut codec = ffmpeg_next::codec::encoder::find_by_name("nvenc_hevc").unwrap();
        let mut context = ffmpeg_next::codec::Context::wrap(ffmpeg_sys_next::avcodec_alloc_context3(codec.as_ptr()), None);
        let mut frame1 = Frame::empty();
        (*frame1.as_mut_ptr()).width = ww;
        (*frame1.as_mut_ptr()).height = hh;
        (*frame1.as_mut_ptr()).format = pixel_format as i32;
        let mut ret = ffmpeg_sys_next::av_frame_get_buffer(frame1.as_mut_ptr(), 0);

        (*context.as_mut_ptr()).width = ww;
        (*context.as_mut_ptr()).height = hh;
        (*context.as_mut_ptr()).pix_fmt = pixel_format;
        (*context.as_mut_ptr()).has_b_frames = 0;
        (*context.as_mut_ptr()).max_b_frames = 0;
        (*context.as_mut_ptr()).gop_size = 15;

        (*context.as_mut_ptr()).time_base = ffmpeg_sys_next::av_make_q(1, 30);
        (*context.as_mut_ptr()).framerate = ffmpeg_sys_next::av_inv_q((*context.as_mut_ptr()).time_base);
        // (*context.as_mut_ptr()).flags |= ffmpeg_next::codec::Flags::LOW_DELAY.bits() as i32;
        (*context.as_mut_ptr()).flags |= ffmpeg_sys_next::AV_CODEC_FLAG2_LOCAL_HEADER;
        // c->thread_count = 4;
        // c->thread_type = FF_THREAD_SLICE;
        // ret = ffmpeg_sys_next::avcodec_open2(context.as_mut_ptr(), codec.as_mut_ptr(), ptr::null_mut());
        // if ret < 0 {
        //     panic!("xxx");
        // }
        ffmpeg_sys_next::av_opt_set((*context.as_mut_ptr()).priv_data, "rc" .as_ptr() as *const c_char, "cbr" .as_ptr() as *const c_char, 0);
        ffmpeg_sys_next::av_opt_set((*context.as_mut_ptr()).priv_data, "crf" .as_ptr() as *const c_char, "25" .as_ptr() as *const c_char, 0);
        // ffmpeg_sys_next::av_opt_set((*context.as_mut_ptr()).priv_data, "preset" .as_ptr() as *const c_char, "medium" .as_ptr() as *const c_char, 0);
        let mut encoder: Encoder = Encoder(context);
        //encoder.set_bit_rate(8*1024*1024);


        let video = ffmpeg_next::codec::encoder::video::Video(encoder);
        let mut ee = video.open().unwrap();
        let sws_context = ffmpeg_sys_next::sws_getContext(4096, 2160, ffmpeg_sys_next::AVPixelFormat::AV_PIX_FMT_BGRA, ww, hh, pixel_format, ffmpeg_sys_next::SWS_BILINEAR, ptr::null_mut(), ptr::null_mut(), ptr::null_mut());
        let mut frame_index = 0;
        loop {
            bucket.wait_max_duration(1, Duration::from_millis(1000));
            match capturer.frame() {
                Ok(frame) => {
                    //  Write the frame, removing end-of-row padding.
                    let stride = frame.len() / h;
                    let rowlen = 4 * w;
                    let src_strides: &[i32] = &[rowlen as i32];
                    let ret = ffmpeg_sys_next::sws_scale(sws_context, &frame.as_ptr(), src_strides.as_ptr(),
                                                         0, 2160,
                                                         (*frame1.as_mut_ptr()).data.as_ptr(), (*frame1.as_mut_ptr()).linesize.as_ptr());
                    if ret <0 {
                        panic!("xsdcsdc");
                    }
                    ee.send_frame(&frame1).unwrap();
                    let mut encoded = Packet::empty();
                    if ee.receive_packet(&mut encoded).is_ok() {
                        file.write_all(encoded.data().unwrap()).unwrap();
                    }
                    // for row in frame.chunks(stride) {
                    //     let row = &row[..rowlen];
                    //
                    //
                    //     out.write_all(row).unwrap();
                    // }
                    //out.write_all(&*frame);
                }
                Err(ref e) if e.kind() == WouldBlock => {
                    // Wait for the frame.
                }
                Err(_) => {
                    // We're done here.
                    break;
                }
            }
        }
    }

}