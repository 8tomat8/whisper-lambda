use axum::{extract, response::Json, routing::post, Router};
use hound;
use lambda_http::{run, Error};
use log;
use serde_json::{json, Value};
use std::io::Cursor;
use std::str;
use whisper_rs::{FullParams, SamplingStrategy, WhisperContext};

mod audio;
mod models;
use models::Model;

#[derive(serde::Deserialize)]
struct Req {
    model: Model,
    file: String,
}

async fn transcribe<R: std::io::Read>(audio: R, model: Model) -> Result<Vec<Segment>, Error> {
    let mut reader = hound::WavReader::new(audio).expect("failed to read wav file");
    let hound::WavSpec {
        channels,
        sample_rate,
        ..
    } = reader.spec();

    // Convert the audio to floating point samples.
    let mut audio = whisper_rs::convert_integer_to_float_audio(
        &reader
            .samples::<i16>()
            .map(|s| s.expect("invalid sample"))
            .collect::<Vec<_>>(),
    );

    if channels == 2 {
        audio = whisper_rs::convert_stereo_to_mono_audio(&audio)
            .expect("failed to convert stereo to mono");
    } else if channels != 1 {
        return Err(format!(">2 channels unsupported").into());
    }

    if sample_rate != 16000 {
        return Err(format!("sample rate must be 16KHz").into());
    }

    // Run the model.
    // Load a context and model.
    let ctx = match WhisperContext::new(model.path().as_str()) {
        Ok(ctx) => ctx,
        Err(e) => return Err(format!("failed to load model: {}", e).into()),
    };

    // Create a state
    let mut state = match ctx.create_state() {
        Ok(state) => state,
        Err(e) => {
            return Err(format!("failed to create key: {}", e).into());
        }
    };

    let params = FullParams::new(SamplingStrategy::Greedy { best_of: 0 });
    match state.full(params, &audio[..]) {
        Ok(_) => (),
        Err(e) => return Err(format!("failed to run model: {}", e).into()),
    };

    // Iterate through the segments of the transcript.
    let num_segments = match state.full_n_segments() {
        Ok(num_segments) => num_segments,
        Err(e) => return Err(format!("failed to get number of segments: {}", e).into()),
    };

    let mut segments = Vec::with_capacity(num_segments.try_into().unwrap());

    for i in 0..num_segments {
        // Get the transcribed text and timestamps for the current segment.
        let segment = match state.full_get_segment_text(i) {
            Ok(segment) => segment,
            Err(e) => format!("[{}] failed to get segment: {}", i, e),
        };

        let start_timestamp = match state.full_get_segment_t0(i) {
            Ok(start_timestamp) => start_timestamp,
            Err(e) => {
                log::warn!("[{}] failed to get start timestamp: {}", i, e);
                continue;
            }
        };
        let end_timestamp = match state.full_get_segment_t1(i) {
            Ok(end_timestamp) => end_timestamp,
            Err(e) => {
                log::warn!("[{}] failed to get start timestamp: {}", i, e);
                continue;
            }
        };

        segments.push(Segment {
            start: start_timestamp,
            end: end_timestamp,
            text: segment,
        });
    }
    return Ok(segments);
}

#[derive(serde::Serialize)]
struct Segment {
    start: i64,
    end: i64,
    text: String,
}

use base64::{engine::general_purpose, Engine as _};
async fn call_handler(extract::Json(payload): extract::Json<Req>) -> Json<Value> {
    let file = match general_purpose::STANDARD.decode(payload.file) {
        Ok(x) => x,
        Err(e) => {
            return Json(json!({
                "error": format!("{}", e),
            }))
        }
    };

    println!("model: {:?}", payload.model);

    let wav_audio = match audio::convert_audio_to_mono_wav(file) {
        Ok(x) => x,
        Err(e) => {
            return Json(json!({
                "error": format!("{}", e),
            }))
        }
    };

    match transcribe(Cursor::new(wav_audio), payload.model).await {
        Ok(segments) => {
            return Json(json!({
                "segments": segments,
            }))
        }
        Err(e) => {
            return Json(json!({
                "error": format!("{}", e),
            }))
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    // required to enable CloudWatch error logging by the runtime
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        // disable printing the name of the module in every log line.
        .with_target(false)
        // disabling time is handy because CloudWatch will add the ingestion time.
        .without_time()
        .init();

    let app = Router::new()
        .route("/", post(call_handler))
        // TODO: make it configurable
        .layer(extract::DefaultBodyLimit::disable());

    run(app).await
}
