
use std::io::{self};
use std::path::PathBuf;
use std::process::{Command, Stdio, ChildStdin, Child};
use std::io::{BufWriter, Write};
use std::time::{SystemTime, UNIX_EPOCH};
use clap::{App, Arg};
use std::{thread, time, fs};
use time::{Duration, Instant };

#[cfg(feature = "image_incl")]
use bytebuffer::ByteBuffer;
#[cfg(feature = "image_incl")]
use image::ColorType;
#[cfg(feature = "image_incl")]
use image::ImageEncoder;
#[cfg(feature = "image_incl")]
use image::codecs::png::PngEncoder;
#[cfg(feature = "image_incl")]
use image::io::Reader as ImageReader;


fn main() -> io::Result<()> {
    let version: Option<&'static str> = option_env!("CARGO_PKG_VERSION");
    let authors: Option<&'static str> = option_env!("CARGO_PKG_AUTHORS");
    let matches = App::new("Youtube Still Image Streamer")
        .version(version.unwrap())
        .author(authors.unwrap())
        .about("Using images to stream to Youtube.")
        .long_about(
            r#"Using images to stream to Youtube.
            To change the streamed image you can replace the image in the file system.
            
            You needs an installed ffmpeg."#,
        ).arg(
            Arg::with_name("width")
                .short("w")
                .long("width")
                .value_name("PIXEL")
                .help("Width of the video.")
                .takes_value(true)
                .default_value("3840")
                .validator(validate_u16),
        ).arg(
            Arg::with_name("height")
                .short("h")
                .long("height")
                .value_name("PIXEL")
                .help("Height of the video.")
                .takes_value(true)
                .default_value("2160")
                .validator(validate_u16),
        )
        .arg(
            Arg::with_name("fps")
                .short("f")
                .long("fps")
                .value_name("FPS")
                .help("Frames per second to stream. 2 should be enough for images.")
                .takes_value(true)
                .default_value("2")
                .validator(validate_u16),
        )
        .arg(
            Arg::with_name("max-bitrate")
                .short("b")
                .long("max-bitrate")
                .value_name("KILOBITS PER SECONDS")
                .help("Maximum ffmpeg bitrate for streaming.")
                .takes_value(true)
                .default_value("1000")
                .validator(validate_u16),
        )
        .arg(
            Arg::with_name("quality")
                .short("q")
                .long("quality")
                //.value_name("")
                .help("Quality value for codec. 0-63, smaller=better quality.")
                .takes_value(true)
                .default_value("18")
                .validator(validate_u16),
        )
        .arg(
            Arg::with_name("ffmpeg")
                .short("p")
                .long("ffmpeg")
                .help("Path to ffmpeg, or simple 'ffmpeg' if in your operating system PATH variable.")
                .takes_value(true)
                .default_value("ffmpeg")
        ).arg(
            Arg::with_name("image_incl")
                .value_name("PATH")
                .help("The image is sent. If you want to change the image, overwrite the image at the same position.")
                .takes_value(true)
                .required(true)
                .validator(validate_pathbuf),
        ).arg(
            Arg::with_name("url")
                .value_name("URL")
                .help(r#"Target URL for streaming. You need a custom URL. For Youtube, for example, in the format "rtmp://a.rtmps.youtube.com/live2/abcf-1234-5678-abcd-1234""#)
                .takes_value(true)
                .required(true)
                .validator(validate_url),
        )
        .get_matches();

    let width  = matches
        .value_of("width")
        .unwrap()
        .parse::<u16>()
        .unwrap();
    let height  = matches
        .value_of("height")
        .unwrap()
        .parse::<u16>()
        .unwrap();
    let fps = matches
        .value_of("fps")
        .unwrap()
        .parse::<u16>()
        .unwrap();
    let max_bitrate = matches
        .value_of("max-bitrate")
        .unwrap()
        .parse::<u16>()
        .unwrap();
    let quality =  matches
        .value_of("quality")
        .unwrap()
        .parse::<u16>()
        .unwrap();
    let ffmpeg =  matches
        .value_of("ffmpeg")
        .unwrap();
    let in_file = 
        matches
        .value_of("image_incl")
        .unwrap();
    let url = 
        matches
        .value_of("url")
        .unwrap();
    
    let resolution_para = &format!("{w}x{h}", w=width, h=height);
    let max_datarate_para = &format!("{}k", max_bitrate);
    let gop = &format!("{}", 4 * fps);
    let quality = &format!("{}", quality);
    let fps_para = &format!("{}", fps);

    let para_ffmpeg = vec![
        //"-y", 
        "-stream_loop", "-1", // repeat stream ???
        "-thread_queue_size", "16",
        "-framerate", fps_para,
        "-pix_fmt", "yuv420p", // pixel format
        
        "-f", "image2pipe",
        "-i", "-",
        "-f", "lavfi", // anullsrc needs lavfi
        "-i", "anullsrc=channel_layout=stereo:sample_rate=44100", // create silent audio
        
        "-c:v", "libx264", // video codec
        "-c:a", "aac", // aac audio codec
        "-b:a", "32k", // 64kBit/sec audio
        "-ac", "2", // 2 channel (stereo)
        "-ar", "44100", // 44100 kHz
        //"-s", "1920x1080",
        "-s", resolution_para, // video resolution
        
        "-maxrate", max_datarate_para,  // max 1MBit/s
        //"-flags", "+global_header",
        //"-x264-params", "keyint=48:min-keyint=48:scenecut=-1",
        "-preset", "superfast", // 
        //"-preset", "ultrafast", // ultrafast superfast veryfast faster fast medium slow slower veryslow placebo
        "-tune", "stillimage", // film animation grain stillimage psnr ssim fastdecode zerolatency
        
        "-bufsize", "2000k", // buffer 2MB
        "-crf", quality, // constant quality 0-63, smaller=better quality
        "-g",  gop, // GOP 4sec, Google sagt "Verwende eine Keyframe-Frequenz von vier Sekunden oder weniger. Da Keyframes derzeit nicht oft genug gesendet werden, können Zwischenspeicherungen auftreten. Die aktuelle Keyframe-Frequenz beträgt 7,9 Sekunden. Fehler bei der Datenaufnahme können falsche Bildergruppengrößen (GoP) verursachen."
        //"-threads", "2", 
        // "-shortest",
        // "/Users/akuechler/Documents/rust-workspace/video-x/target/test.mp4"
        "-hide_banner",
        "-f", "flv",
        url
    ];
    loop {
        let mut ffmpeg_child = Command::new(ffmpeg)
            .args(&para_ffmpeg)
            .stdin(Stdio::piped())
            .spawn()?;

        if let Ok(_) = run(&ffmpeg_child, in_file, fps) {
            break;
        }
        if let Err(e) = ffmpeg_child.kill() {
            println!("Error during killing ffmpeg {}", e);
        }
    }
    Ok(())
    }
    
fn run(ffmpeg_child: &Child, in_file: &str, fps: u16) -> io::Result<()> {
    
    let mut ffmpeg_stdin = ffmpeg_child.stdin.as_ref().unwrap();
    let mut writer = BufWriter::new(&mut ffmpeg_stdin);


    let time_per_frame = time::Duration::from_millis(1000 / (fps as u64));
    
    let mut last_timestamp = UNIX_EPOCH;
    let mut bytes = vec![];
    loop {
        let start = Instant::now();
        match image_timestamp(in_file) {
            Ok(new_timestamp) => {
                if last_timestamp != new_timestamp {
                    println!("Other image detected. Reload.");
                    bytes = match load_image2(in_file) {
                        Ok(b) => {
                            last_timestamp = new_timestamp;
                            b
                        },
                        Err(e) =>  {
                            println!("Error {}", e);
                            thread::sleep(Duration::from_secs(1)); // sleep one sec and try again
                            vec![]
                        }
                    };
                }
                if !bytes.is_empty() {
                    if let Err(e) = write_and_sleep(start, &mut writer, time_per_frame, &bytes) {
                        println!("Error {}", e);

                        return Err(e); // return Err -> restart ffmpeg and try again
                    }
                }
            },
            Err(e) => {
                println!("Error {}", e);
                thread::sleep(Duration::from_secs(1)); // sleep one sec and try again
            }
        };

    }
}

fn image_timestamp(file_path: &str) -> io::Result<SystemTime> {
    let metadata = fs::metadata(file_path)?;
    metadata.modified()
}

fn write_and_sleep(start: Instant, writer: &mut BufWriter<&mut &ChildStdin>, time_per_frame : Duration, bytes: &Vec<u8>) -> io::Result<()> {
    writer.write_all(bytes)?;

    let duration_write =  Instant::now().duration_since(start);
    let duration_sleep = time_per_frame.saturating_sub(duration_write).saturating_sub(Duration::from_millis(20));
    #[cfg(feature = "debug")]
    println!("Sleep {}ms", duration_sleep.as_millis());
    thread::sleep(duration_sleep);

    Ok(())
}

fn load_image2(in_file: &str) -> io::Result<Vec<u8>> {
    std::fs::read(in_file)
}

#[cfg(feature = "image_incl")]
fn _load_image(in_file:&str, width: u32, height: u32) -> io::Result<Vec<u8>> {
    let img = ImageReader::open(in_file)?.decode().unwrap();
    let byte_vec = img.into_rgb8();

    let mut buffer = ByteBuffer::new();
    let encoder = PngEncoder::new(buffer.by_ref());
    encoder.write_image(&byte_vec, width, height, ColorType::Rgb8).unwrap();
    let bytes = buffer.to_bytes();
    Ok(bytes)
}

pub fn validate_u16(value: String) -> Result<(), String> {
    match value.parse::<u16>() {
        Ok(_) => Ok(()),
        _ => Err(format!(r#"Value have to be a number, not "{}"."#, &value))
    }
}

pub fn validate_pathbuf(value: String) -> Result<(), String> {
    let path = PathBuf::from(&value);
    match path.try_exists() {
        Ok(true) => Ok(()),
        _ => Err(format!(r#"Value have to be exists path, not "{}"."#, &value))
    }
}

pub fn validate_url(value: String) -> Result<(), String> {
    if value.len() > 0 {
        Ok(())
    }else {
        Err(format!(r#"Value have to be exists path, not "{}"."#, &value))
    }
}
