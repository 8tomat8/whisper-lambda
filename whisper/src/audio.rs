use std::error::Error;
use std::io::{Read, Write};
use std::process::Command;
use tempfile::NamedTempFile;

pub fn convert_audio_to_mono_wav(input: Vec<u8>) -> Result<Vec<u8>, Box<dyn Error>> {
    let mut input_file = match NamedTempFile::new() {
        Ok(f) => f,
        Err(e) => return Err(Box::new(e)),
    };
    let output_file = match NamedTempFile::new() {
        Ok(f) => f,
        Err(e) => return Err(Box::new(e)),
    };

    match input_file.write_all(&input) {
        Ok(_) => (),
        Err(e) => return Err(Box::new(e)),
    };
    match input_file.flush() {
        Ok(_) => (),
        Err(e) => return Err(Box::new(e)),
    };

    let input_path = input_file.into_temp_path();
    let output_path = output_file.into_temp_path();

    // Construct the FFmpeg command
    let status = Command::new("ffmpeg")
        .args([
            "-y",
            "-i",
            input_path.to_str().unwrap(), // Input file
            "-ac",
            "1", // Set audio channels to 1 (mono)
            "-ar",
            "16000", // Set sample rate to 16000 Hz
            "-f",
            "wav",                         // Set format to WAV
            output_path.to_str().unwrap(), // Output file
        ])
        .status()?;

    if !status.success() {
        return Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::Other,
            "FFmpeg command failed",
        )));
    }

    let mut output_file = match std::fs::File::open(output_path) {
        Ok(f) => f,
        Err(e) => return Err(Box::new(e)),
    };

    let mut buf = Vec::new();
    match output_file.read_to_end(&mut buf) {
        Ok(_) => (),
        Err(e) => return Err(Box::new(e)),
    };

    Ok(buf)
}
