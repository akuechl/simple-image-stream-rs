# simple-image-stream-rs

 ```
Youtube Still Image Streamer 0.2.0

Using images to stream to Youtube.
            To change the streamed image you can replace the image in the file system.
            
            You needs an installed ffmpeg.

USAGE:
    video-x [OPTIONS] <PATH> <URL>

FLAGS:
        --help       
            Prints help information

    -V, --version    
            Prints version information


OPTIONS:
    -p, --ffmpeg <ffmpeg>                       
            Path to ffmpeg, or simple 'ffmpeg' if in your operating system PATH variable. [default: ffmpeg]

    -f, --fps <FPS>                             
            Frames per second to stream. 2 should be enough for images. [default: 2]

    -h, --height <PIXEL>                        
            Height of the video. [default: 2160]

    -b, --max-bitrate <KILOBITS PER SECONDS>    
            Maximum ffmpeg bitrate for streaming. [default: 1000]

    -q, --quality <quality>                     
            Quality value for codec. 0-63, smaller=better quality. [default: 18]

    -w, --width <PIXEL>                         
            Width of the video. [default: 3840]


ARGS:
    <PATH>    
            The image is sent. If you want to change the image, overwrite the image at the same position.

    <URL>     
            Target URL for streaming. You need a custom URL. For Youtube, for example, in the format
            "rtmp://a.rtmps.youtube.com/live2/abcf-1234-5678-abcd-1234"
```
