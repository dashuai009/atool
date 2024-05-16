use ffmpeg::{frame, media};
use ffmpeg::{Rescale};
use ffmpeg_next as ffmpeg;
use hound::{self, SampleFormat};

fn load_audio_waveform(filename: &str) -> hound::Result<(Vec<f32>, usize)> {
    let mut reader = hound::WavReader::open(filename)?;
    let spec = reader.spec();

    let duration = reader.duration() as usize;
    let channels = spec.channels as usize;
    let sample_rate = spec.sample_rate as usize;
    let bits_per_sample = spec.bits_per_sample;
    let sample_format = spec.sample_format;

    assert_eq!(sample_rate, 16000, "The audio sample rate must be 16k.");
    assert_eq!(channels, 1, "The audio must be single-channel.");

    let max_int_val = 2_u32.pow(spec.bits_per_sample as u32 - 1) - 1;

    let floats = match sample_format {
        SampleFormat::Float => reader.into_samples::<f32>().collect::<hound::Result<_>>()?,
        SampleFormat::Int => reader
            .into_samples::<i32>()
            .map(|s| s.map(|s| s as f32 / 1 as f32))
            .collect::<hound::Result<_>>()?,
    };

    return Ok((floats, sample_rate));
}

fn test(input_file: &str) -> Result<Vec<f32>, ffmpeg::Error> {
    ffmpeg::init().unwrap();

    let mut ictx = ffmpeg::format::input(&input_file).unwrap();
    let input_audio_stream = ictx
        .streams()
        .best(media::Type::Audio)
        .expect("could not find best audio stream");
    let audio_stream_index = input_audio_stream.index();
    // unsafe {
    //     println!(
    //         "input stream = {:?} par = {:?}",
    //         input,
    //         *input.parameters().as_ptr()
    //     );
    // }
    let context = ffmpeg::codec::context::Context::from_parameters(input_audio_stream.parameters())?;
    unsafe {
        //  Guessed Channel Layout: mono
        let par = input_audio_stream.parameters().as_mut_ptr();
        if (*par).channel_layout == 0 {
            (*par).channel_layout = ffmpeg::util::channel_layout::ChannelLayout::MONO.bits()
        };
    }

    let mut decoder = context.decoder().audio()?;
    decoder.set_parameters(input_audio_stream.parameters())?;

    let src_format = decoder.format();
    let src_rate = decoder.rate();
    let src_channel_layout = decoder.channel_layout();

    let dst_rate = 16000u32;
    let mut swr = ffmpeg::software::resampling::Context::get(
        src_format,
        src_channel_layout,
        src_rate,
        ffmpeg::util::format::Sample::F32(ffmpeg::util::format::sample::Type::Packed), // AV_SAMPLE_FMT_FLT
        ffmpeg::util::channel_layout::ChannelLayout::MONO,
        dst_rate,
    )?;

    let mut frame = frame::Audio::empty();
    let mut res = vec![];
    for (stream, packet) in ictx.packets() {
        if stream.index() != audio_stream_index {
            continue;
        }
        decoder.send_packet(&packet)?;
        while decoder.receive_frame(&mut frame).is_ok() {
            let mut out_frame = frame::Audio::empty();
            let _resample_res = swr.run(&frame, &mut out_frame)?;
            unsafe {
                let out_frame = out_frame.as_mut_ptr();
                let tmp_slice = slice::from_raw_parts(
                    (*(*out_frame).extended_data) as *mut f32,
                    (*out_frame).nb_samples as usize,
                ); // the dst_format in swr is AV_SAMPLE_FMT_FLT, f32
                res.extend_from_slice(tmp_slice);
            }
        }
    }
    Ok(res)
}

#[cfg(test)]
mod test{
    #[test]
    fn test_1(){
        let res = super::test();
        println!("Hello, world! {:?}", res);

    }
}