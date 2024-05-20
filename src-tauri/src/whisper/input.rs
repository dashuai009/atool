use burn::prelude::{Backend, Tensor};
use ffmpeg_next::{frame, media};
use ffmpeg_next as ffmpeg;
use whisper::audio::{log_mel_spectrogram, N_SAMPLES};

pub fn load_audio_waveform_with_ffmpeg(input_file: &str) -> Result<Vec<f32>, ffmpeg::Error> {
    ffmpeg::init()?;

    let mut ictx = ffmpeg::format::input(&input_file)?;
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
    let mut context =
        ffmpeg::codec::context::Context::from_parameters(input_audio_stream.parameters())?;
    unsafe {
        //  Guessed Channel Layout: mono
        let xx = context.as_mut_ptr();
        if (*xx).ch_layout.order == ffmpeg_next::ffi::AVChannelOrder::AV_CHANNEL_ORDER_UNSPEC {
            // let mut NewLayout = std::mem::zeroed::<ffmpeg_next::ffi::AVChannelLayout>();
            // ffmpeg_next::ffi::av_channel_layout_default(&mut NewLayout, (*xx).ch_layout.nb_channels);
            // let res = ffmpeg_next::ffi::av_channel_layout_copy(&mut (*xx).ch_layout, &NewLayout);
            // println!("res = {res}");
            // std::mem::forget(NewLayout);
        }
        // let par = input_audio_stream.parameters().as_mut_ptr();
        // let x = (*par).ch_layout;

        // (*par).get
        // ffmpeg::ffi::av_c
        // println!("channel = {:?} nb_channel {:?}", x.order, x.nb_channels);

        // if (*par).ch_layout. == 0 {
        //     (*par).ch_layout = ffmpeg::util::channel_layout::ChannelLayout::MONO.bits()
        // };
    }

    let mut decoder = context.decoder().audio()?;
    decoder.set_parameters(input_audio_stream.parameters())?;

    // let src_format = decoder.format();
    // let src_rate = decoder.rate();
    // let src_channel_layout = decoder.channel_layout();
    //

    let mut frame = frame::Audio::empty();
    let mut res = vec![];
    for (stream, packet) in ictx.packets() {
        if stream.index() != audio_stream_index {
            continue;
        }
        decoder.send_packet(&packet)?;
        while decoder.receive_frame(&mut frame).is_ok() {
            let mut out_frame = frame::Audio::empty();
            {
                let src_format = frame.format();
                // unsafe {
                //     let s = (*frame.as_ptr()).format;
                //     println!("frame format = {s:?}");
                // }
                let src_rate = frame.rate();
                let src_channel_layout = frame.channel_layout();

                let dst_rate = 16000u32;

                let mut swr = ffmpeg::software::resampling::Context::get(
                    src_format,
                    src_channel_layout,
                    src_rate,
                    ffmpeg::util::format::Sample::F32(ffmpeg::util::format::sample::Type::Packed), // AV_SAMPLE_FMT_FLT
                    ffmpeg::util::channel_layout::ChannelLayout::MONO,
                    dst_rate,
                )?;
                let _resample_res = swr.run(&frame, &mut out_frame)?;
            }
            // let in_format = frame.format();
            // unsafe {
            //     let raw_in = frame.as_mut_ptr();
            //     let raw_swr = swr.as_mut_ptr();
            //     // let res = av_channel_layout_compare(&(*raw_in).ch_layout, (*raw_swr).in_ch_layout);
            //     // println!("av_channel_layout_compare = {res}");
            // }
            unsafe {
                let out_frame = out_frame.as_mut_ptr();
                let tmp_slice = std::slice::from_raw_parts(
                    (*(*out_frame).extended_data) as *mut f32,
                    (*out_frame).nb_samples as usize,
                ); // the dst_format in swr is AV_SAMPLE_FMT_FLT, f32
                res.extend_from_slice(tmp_slice);
            }
        }
    }
    Ok(res)
}


pub fn filepath_to_mels<B: Backend>(file_path: &str, device: &B::Device) -> (Tensor<B, 3>, usize) {
    let mut wave = load_audio_waveform_with_ffmpeg(&file_path).unwrap();
    let wave_duration = wave.len();
    let pad_len = if wave.len() % N_SAMPLES == 0 { 0 } else { N_SAMPLES - wave.len() % N_SAMPLES };
    for _i in 0..pad_len {
        wave.push(0.0);
    }
    let audio_len = wave.len();
    let audio = Tensor::<B, 2>::from_floats(
        burn::tensor::Data::new(wave, [audio_len / N_SAMPLES, N_SAMPLES].into()),
        &device,
    );
    (log_mel_spectrogram(audio), wave_duration)
}