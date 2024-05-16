// Given an input file, transcode all video streams into H.264 (using libx264)
// while copying audio and subtitle streams.
//
// Invocation:
//
//   transcode-x264 <input> <output> [<x264_opts>]
//
// <x264_opts> is a comma-delimited list of key=val. default is "preset=medium".
// See https://ffmpeg.org/ffmpeg-codecs.html#libx264_002c-libx264rgb and
// https://trac.ffmpeg.org/wiki/Encode/H.264 for available and commonly used
// options.
//
// Examples:
//
//   transcode-x264 input.flv output.mp4
//   transcode-x264 input.mkv output.mkv 'preset=veryslow,crf=18'

extern crate ffmpeg_next as ffmpeg;

use std::env;
use std::time::Instant;
use std::{collections::HashMap, path::Path};

use ffmpeg::{
    codec, decoder, encoder, format, frame, log, media, picture, Dictionary, Packet, Rational,
};
use ffmpeg_next::codec::Id;
use serde::{Deserialize, Serialize};

const DEFAULT_X264_OPTS: &str = "preset=medium";

struct Transcoder {
    ost_index: usize,
    decoder: decoder::Video,
    encoder: encoder::video::Video,
    logging_enabled: bool,
    frame_count: usize,
    last_log_frame_count: usize,
    starting_time: Instant,
    last_log_time: Instant,
}

impl Transcoder {
    fn new(
        ist: &format::stream::Stream,
        octx: &mut format::context::Output,
        ost_index: usize,
        opts: Option<Dictionary>,
        enable_logging: bool,
        to_type: AtoolCodecType,
    ) -> Result<Self, ffmpeg::Error> {
        let global_header = octx.format().flags().contains(format::Flags::GLOBAL_HEADER);
        let decoder = ffmpeg::codec::context::Context::from_parameters(ist.parameters())?
            .decoder()
            .video()?;
        let idd = encoder::find(to_type.into());
        let mut ost = octx.add_stream(encoder::find(to_type.into()))?;
        let mut pars = ost.parameters();
        let mut encoder = codec::context::Context::from_parameters(pars)?
            .encoder()
            .video()?;

        println!("rate = {} height = {}, width = {}, aspect_ratio = {}, format = {:?}",
                 ist.rate(), decoder.height(), decoder.width(), decoder.aspect_ratio(), decoder.format());
        encoder.set_height(decoder.height());
        encoder.set_width(decoder.width());
        encoder.set_aspect_ratio(decoder.aspect_ratio());
        encoder.set_format(decoder.format());
        encoder.set_frame_rate(Some(ist.rate()));
        encoder.set_time_base(ist.rate().invert());
        if global_header {
            encoder.set_flags(codec::Flags::GLOBAL_HEADER);
        }

        if let Some(opt) = opts {
            encoder
                .open_with(opt)
                .expect(&format!("error opening {:?} encoder with supplied settings", to_type));
        } else {
            encoder.open().expect("open filed");
        }
        encoder = codec::context::Context::from_parameters(ost.parameters())?
            .encoder()
            .video()?;
        ost.set_parameters(&encoder);
        Ok(Self {
            ost_index,
            decoder,
            encoder: codec::context::Context::from_parameters(ost.parameters())?
                .encoder()
                .video()?,
            logging_enabled: enable_logging,
            frame_count: 0,
            last_log_frame_count: 0,
            starting_time: Instant::now(),
            last_log_time: Instant::now(),
        })
    }

    fn send_packet_to_decoder(&mut self, packet: &Packet) {
        self.decoder.send_packet(packet).unwrap();
    }

    fn send_eof_to_decoder(&mut self) {
        self.decoder.send_eof().unwrap();
    }

    fn receive_and_process_decoded_frames(
        &mut self,
        octx: &mut format::context::Output,
        ost_time_base: Rational,
    ) {
        let mut frame = frame::Video::empty();
        while self.decoder.receive_frame(&mut frame).is_ok() {
            self.frame_count += 1;
            let timestamp = frame.timestamp();
            self.log_progress(f64::from(
                Rational(timestamp.unwrap_or(0) as i32, 1) * self.decoder.time_base(),
            ));
            frame.set_pts(timestamp);
            frame.set_kind(picture::Type::None);
            self.send_frame_to_encoder(&frame);
            self.receive_and_process_encoded_packets(octx, ost_time_base);
        }
    }

    fn send_frame_to_encoder(&mut self, frame: &frame::Video) {
        self.encoder.send_frame(frame).unwrap();
    }

    fn send_eof_to_encoder(&mut self) {
        self.encoder.send_eof().unwrap();
    }

    fn receive_and_process_encoded_packets(
        &mut self,
        octx: &mut format::context::Output,
        ost_time_base: Rational,
    ) {
        let mut encoded = Packet::empty();
        while self.encoder.receive_packet(&mut encoded).is_ok() {
            encoded.set_stream(self.ost_index);
            encoded.rescale_ts(self.decoder.time_base(), ost_time_base);
            encoded.write_interleaved(octx).unwrap();
        }
    }

    fn log_progress(&mut self, timestamp: f64) {
        if !self.logging_enabled
            || (self.frame_count - self.last_log_frame_count < 100
            && self.last_log_time.elapsed().as_secs_f64() < 1.0)
        {
            return;
        }
        eprintln!(
            "time elpased: \t{:8.2}\tframe count: {:8}\ttimestamp: {:8.2}",
            self.starting_time.elapsed().as_secs_f64(),
            self.frame_count,
            timestamp
        );
        self.last_log_frame_count = self.frame_count;
        self.last_log_time = Instant::now();
    }
}

fn parse_opts<'a>(s: String) -> Option<Dictionary<'a>> {
    let mut dict = Dictionary::new();
    let s = s.split_terminator(',').collect::<Vec<_>>();
    if s.len() == 0 {
        return None;
    }
    for keyval in s {
        let tokens: Vec<&str> = keyval.split('=').collect();
        match tokens[..] {
            [key, val] => dict.set(key, val),
            _ => return None,
        }
    }
    Some(dict)
}

#[derive(Clone, Copy, Serialize, Deserialize, Debug)]
pub enum AtoolCodecType {
    X264,
    X265,
    AV1,
    VP8,
    VP9,
}


impl From<AtoolCodecType> for Id {
    fn from(value: AtoolCodecType) -> Self {
        match value {
            AtoolCodecType::X264 => {
                Id::H264
            }
            AtoolCodecType::X265 => {
                Id::H265
            }
            AtoolCodecType::AV1 => {
                Id::AV1
            }
            AtoolCodecType::VP8 => {
                Id::VP8
            }
            AtoolCodecType::VP9 => {
                Id::VP9
            }
        }
    }
}

#[tauri::command]
pub fn video_convert_cmd(
    input_file: Vec<String>,
    to_type: AtoolCodecType,
    output_dir: String,
    options: String,
) {
    let output_dir = Path::new(&output_dir);
    for file in input_file {
        let file_path = Path::new(&file);

        // 提取文件名
        if let Some(file_name) = file_path.file_name() {
            let target_file_path = output_dir.join(file_name);
            if (!target_file_path.exists()) {
                video_convert(&file, target_file_path.to_str().unwrap(), to_type, &options);
            } else {
                for i in 0..1000 {
                    let extension = file_path.extension().map(|x| format!(".{}", x.to_str().unwrap())).unwrap_or("".to_string());
                    let file_stem = file_path.file_stem().unwrap();
                    let s = format!("{}_{}{}", file_stem.to_str().unwrap(), i, extension);

                    let new_target_file_path = output_dir.join(s);
                    if !new_target_file_path.exists() {
                        video_convert(&file, new_target_file_path.to_str().unwrap(), to_type, &options);
                        break;
                    }
                }
            }
        }
    }
}

fn video_convert(
    input_file: &str,
    output_file: &str,
    to_type: AtoolCodecType,
    options: &str,
) {
    println!("video convert: {} {} {:?} {}", input_file, output_file, to_type, options);
    let x264_opts = parse_opts(options.to_string());

    eprintln!("x264 options: {:?}", x264_opts);

    ffmpeg::init().unwrap();
    log::set_level(log::Level::Info);

    let mut ictx = format::input(&input_file).unwrap();
    let mut octx = format::output(&output_file).unwrap();

    format::context::input::dump(&ictx, 0, Some(input_file));

    let best_video_stream_index = ictx
        .streams()
        .best(media::Type::Video)
        .map(|stream| stream.index());
    let mut stream_mapping: Vec<isize> = vec![0; ictx.nb_streams() as _];
    let mut ist_time_bases = vec![Rational(0, 0); ictx.nb_streams() as _];
    let mut ost_time_bases = vec![Rational(0, 0); ictx.nb_streams() as _];
    let mut transcoders = HashMap::new();
    let mut ost_index = 0;
    for (ist_index, ist) in ictx.streams().enumerate() {
        let ist_medium = ist.parameters().medium();
        if ist_medium != media::Type::Audio
            && ist_medium != media::Type::Video
            && ist_medium != media::Type::Subtitle
        {
            stream_mapping[ist_index] = -1;
            continue;
        }
        stream_mapping[ist_index] = ost_index;
        ist_time_bases[ist_index] = ist.time_base();
        if ist_medium == media::Type::Video {
            // Initialize transcoder for video stream.
            transcoders.insert(
                ist_index,
                Transcoder::new(
                    &ist,
                    &mut octx,
                    ost_index as _,
                    x264_opts.to_owned(),
                    Some(ist_index) == best_video_stream_index,
                    to_type,
                ).unwrap(),
            );
        } else {
            // Set up for stream copy for non-video stream.
            let mut ost = octx.add_stream(encoder::find(codec::Id::None)).unwrap();
            ost.set_parameters(ist.parameters());
            // We need to set codec_tag to 0 lest we run into incompatible codec tag
            // issues when muxing into a different container format. Unfortunately
            // there's no high level API to do this (yet).
            unsafe {
                (*ost.parameters().as_mut_ptr()).codec_tag = 0;
            }
        }
        ost_index += 1;
    }

    octx.set_metadata(ictx.metadata().to_owned());
    format::context::output::dump(&octx, 0, Some(&output_file));
    octx.write_header().unwrap();

    for (ost_index, _) in octx.streams().enumerate() {
        ost_time_bases[ost_index] = octx.stream(ost_index as _).unwrap().time_base();
    }

    for (stream, mut packet) in ictx.packets() {
        let ist_index = stream.index();
        let ost_index = stream_mapping[ist_index];
        if ost_index < 0 {
            continue;
        }
        let ost_time_base = ost_time_bases[ost_index as usize];
        match transcoders.get_mut(&ist_index) {
            Some(transcoder) => {
                packet.rescale_ts(stream.time_base(), transcoder.decoder.time_base());
                transcoder.send_packet_to_decoder(&packet);
                transcoder.receive_and_process_decoded_frames(&mut octx, ost_time_base);
            }
            None => {
                // Do stream copy on non-video streams.
                packet.rescale_ts(ist_time_bases[ist_index], ost_time_base);
                packet.set_position(-1);
                packet.set_stream(ost_index as _);
                packet.write_interleaved(&mut octx).unwrap();
            }
        }
    }

    // Flush encoders and decoders.
    for (ost_index, transcoder) in transcoders.iter_mut() {
        let ost_time_base = ost_time_bases[*ost_index];
        transcoder.send_eof_to_decoder();
        transcoder.receive_and_process_decoded_frames(&mut octx, ost_time_base);
        transcoder.send_eof_to_encoder();
        transcoder.receive_and_process_encoded_packets(&mut octx, ost_time_base);
    }

    octx.write_trailer().unwrap();
}

#[cfg(test)]
mod test {
    extern crate ffmpeg_next as ffmpeg;

    use crate::video_factory::AtoolCodecType;

    fn print_decoder(codec: ffmpeg::codec::codec::Codec) {
        println!("type: decoder");
        println!("\t id: {:?}", codec.id());
        println!("\t name: {}", codec.name());
        println!("\t description: {}", codec.description());
        println!("\t medium: {:?}", codec.medium());
        println!("\t capabilities: {:?}", codec.capabilities());

        if let Some(profiles) = codec.profiles() {
            println!("\t profiles: {:?}", profiles.collect::<Vec<_>>());
        } else {
            println!("\t profiles: none");
        }

        if let Ok(video) = codec.video() {
            if let Some(rates) = video.rates() {
                println!("\t rates: {:?}", rates.collect::<Vec<_>>());
            } else {
                println!("\t rates: any");
            }

            if let Some(formats) = video.formats() {
                println!("\t formats: {:?}", formats.collect::<Vec<_>>());
            } else {
                println!("\t formats: any");
            }
        }

        if let Ok(audio) = codec.audio() {
            if let Some(rates) = audio.rates() {
                println!("\t rates: {:?}", rates.collect::<Vec<_>>());
            } else {
                println!("\t rates: any");
            }

            if let Some(formats) = audio.formats() {
                println!("\t formats: {:?}", formats.collect::<Vec<_>>());
            } else {
                println!("\t formats: any");
            }

            if let Some(layouts) = audio.channel_layouts() {
                println!("\t channel_layouts: {:?}", layouts.collect::<Vec<_>>());
            } else {
                println!("\t channel_layouts: any");
            }
        }

        println!("\t max_lowres: {:?}", codec.max_lowres());
    }

    fn print_encoder(codec: ffmpeg::codec::codec::Codec) {
        println!();
        println!("type: encoder");
        println!("\t id: {:?}", codec.id());
        println!("\t name: {}", codec.name());
        println!("\t description: {}", codec.description());
        println!("\t medium: {:?}", codec.medium());
        println!("\t capabilities: {:?}", codec.capabilities());

        if let Some(profiles) = codec.profiles() {
            println!("\t profiles: {:?}", profiles.collect::<Vec<_>>());
        }

        if let Ok(video) = codec.video() {
            if let Some(rates) = video.rates() {
                println!("\t rates: {:?}", rates.collect::<Vec<_>>());
            } else {
                println!("\t rates: any");
            }

            if let Some(formats) = video.formats() {
                println!("\t formats: {:?}", formats.collect::<Vec<_>>());
            } else {
                println!("\t formats: any");
            }
        }

        if let Ok(audio) = codec.audio() {
            if let Some(rates) = audio.rates() {
                println!("\t rates: {:?}", rates.collect::<Vec<_>>());
            } else {
                println!("\t rates: any");
            }

            if let Some(formats) = audio.formats() {
                println!("\t formats: {:?}", formats.collect::<Vec<_>>());
            } else {
                println!("\t formats: any");
            }

            if let Some(layouts) = audio.channel_layouts() {
                println!("\t channel_layouts: {:?}", layouts.collect::<Vec<_>>());
            } else {
                println!("\t channel_layouts: any");
            }
        }

        println!("\t max_lowres: {:?}", codec.max_lowres());
    }

    #[test]
    fn test_ffmpeg() {
        ffmpeg::init().unwrap();

        // for arg in ["libx264", "hevc", "av1_qsv", "vp9", "vp8"] {
        //     if let Some(codec) = ffmpeg::decoder::find_by_name(&arg) {
        //         print_decoder(codec);
        //     } else {
        //         println!("can't find decoder for {arg}");
        //     }

        //     if let Some(codec) = ffmpeg::encoder::find_by_name(&arg) {
        //         print_encoder(codec);
        //     } else {
        //         println!("can't find encoder for {arg}");
        //     }
        // }

        for arg in [
            ffmpeg::codec::id::Id::H264,
            ffmpeg::codec::id::Id::H265,
            ffmpeg::codec::id::Id::AV1,
            ffmpeg::codec::id::Id::VP9,
            ffmpeg::codec::id::Id::PCM_S16LE,
        ] {
            if let Some(codec) = ffmpeg::decoder::find(arg) {
                print_decoder(codec);
            } else {
                println!("can't find decoder for {:?}", arg);
            }

            if let Some(codec) = ffmpeg::encoder::find(arg) {
                print_encoder(codec);
            } else {
                println!("can't find encoder for {:?}", arg);
            }
        }

       super::video_convert(
           "/Users/dashuai/Downloads/VID_20240124_163100.mp4",
           "/Users/dashuai/Downloads/VID_20240124_163100_7.mp4",
           AtoolCodecType::X264,
           ""
       )
    }
}
