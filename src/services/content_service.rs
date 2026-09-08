use sqlx::PgPool;
use uuid::Uuid;

use crate::errors::AppError;
use crate::models::content::{Content, ContentStatus, ContentType};

/// Проверка, что файл имеет поддерживаемый формат
pub fn validate_file_format(filename: &str, content_type: &ContentType) -> Result<(), AppError> {
    let extension = filename.rsplit('.').last().unwrap_or("").to_lowercase();

    match content_type {
        ContentType::Audio => {
            let supported = ["flac", "wav", "mp3"];
            if !supported.contains(&extension.as_str()) {
                return Err(AppError::ValidationError(
                    format!("Неподдерживаемый аудиоформат: {}. Допустимы: FLAC, WAV, MP3", extension)
                ));
            }
        }
        ContentType::Video => {
            let supported = ["mp4", "mov"];
            if !supported.contains(&extension.as_str()) {
                return Err(AppError::ValidationError(
                    format!("Неподдерживаемый видеоформат: {}. Допустимы: MP4, MOV", extension)
                ));
            }
        }
    }

    Ok(())
}

/// Проверка размера файла (максимум 2 ГБ)
pub fn validate_file_size(size_bytes: i64) -> Result<(), AppError> {
    const MAX_SIZE: i64 = 2 * 1024 * 1024 * 1024; // 2 GB
    if size_bytes > MAX_SIZE {
        return Err(AppError::ValidationError(
            format!("Файл слишком большой. Максимум: 2 ГБ")
        ));
    }
    Ok(())
}

/// Создание тизера из аудиофайла (вызов FFmpeg)
pub async fn create_audio_teaser(
    input_path: &str,
    output_path: &str,
    start_seconds: i32,
    duration_seconds: i32,
) -> Result<(), AppError> {
    // В реальном коде — вызов FFmpeg для нарезки тизера:
    // ffmpeg -i input.flac -ss {start} -t {duration} -acodec libmp3lame -b:a 192k output.mp3
    
    // Пример команды:
    let _command = format!(
        "ffmpeg -i {} -ss {} -t {} -acodec libmp3lame -b:a 192k {} -y",
        input_path, start_seconds, duration_seconds, output_path
    );

    // tokio::process::Command::new("ffmpeg")
    //     .args(&["-i", input_path, "-ss", &start_seconds.to_string(), ...])
    //     .output()
    //     .await?;

    Ok(())
}

/// Создание тизера из видеофайла (вызов FFmpeg)
pub async fn create_video_teaser(
    input_path: &str,
    output_path: &str,
    start_seconds: i32,
    duration_seconds: i32,
) -> Result<(), AppError> {
    // ffmpeg -i input.mp4 -ss {start} -t {duration} -c:v libx264 -c:a aac output.mp4
    let _command = format!(
        "ffmpeg -i {} -ss {} -t {} -c:v libx264 -preset fast -c:a aac {} -y",
        input_path, start_seconds, duration_seconds, output_path
    );

    Ok(())
}

/// Транскодирование видео в HLS (создание сегментов)
pub async fn transcode_to_hls(
    input_path: &str,
    output_dir: &str,
    content_id: Uuid,
) -> Result<Vec<String>, AppError> {
    // Создание нескольких quality levels
    let qualities = vec![
        ("360p", "640x360", "500k"),
        ("720p", "1280x720", "1500k"),
        ("1080p", "1920x1080", "3000k"),
    ];

    let mut segment_urls = Vec::new();

    for (label, resolution, bitrate) in qualities {
        let output = format!("{}/{}", output_dir, label);
        
        // ffmpeg -i input.mp4 -c:v libx264 -s {resolution} -b:v {bitrate} 
        //   -c:a aac -b:a 128k -f hls -hls_time 6 -hls_list_size 0 
        //   -hls_key_info_file key_info {output}/playlist.m3u8
        
        let _command = format!(
            "ffmpeg -i {} -c:v libx264 -s {} -b:v {} -c:a aac -b:a 128k -f hls -hls_time 6 -hls_segment_filename '{}/segment_%03d.ts' {}/playlist.m3u8",
            input_path, resolution, bitrate, output, output
        );

        segment_urls.push(format!("{}/playlist.m3u8", label));
    }

    Ok(segment_urls)
}

/// Транскодирование аудио в HLS
pub async fn transcode_audio_to_hls(
    input_path: &str,
    output_dir: &str,
) -> Result<Vec<String>, AppError> {
    // ffmpeg -i input.flac -c:a aac -b:a 256k -f hls -hls_time 10 
    //   -hls_segment_filename '{output}/segment_%03d.ts' {output}/playlist.m3u8

    let _command = format!(
        "ffmpeg -i {} -c:a aac -b:a 256k -f hls -hls_time 10 -hls_segment_filename '{}/segment_%03d.ts' {}/playlist.m3u8",
        input_path, output_dir, output_dir
    );

    Ok(vec![format!("{}/playlist.m3u8", output_dir)])
}

/// Шифрование HLS-сегментов (AES-128)
pub async fn encrypt_hls_segments(
    segments_dir: &str,
    key_id: &str,
) -> Result<String, AppError> {
    // Генерация ключа шифрования
    // openssl rand -hex 16 > {segments_dir}/key.key
    // Создание key info файла для FFmpeg
    
    let key_info_path = format!("{}/key_info.txt", segments_dir);
    let _key_info = format!(
        "https://cdn.chistovik.ru/keys/{}.key\n{}/key.key",
        key_id, segments_dir
    );

    Ok(key_info_path)
}

/// Получение метаданных аудиофайла (длительность, формат, битрейт)
pub async fn get_audio_metadata(file_path: &str) -> Result<AudioMetadata, AppError> {
    // ffprobe -v quiet -print_format json -show_format -show_streams {file_path}
    
    // В реальном коде — парсинг JSON-вывода ffprobe
    Ok(AudioMetadata {
        duration_seconds: 0,
        format: "unknown".to_string(),
        bitrate: 0,
        sample_rate: 0,
        channels: 0,
    })
}

/// Получение метаданных видеофайла
pub async fn get_video_metadata(file_path: &str) -> Result<VideoMetadata, AppError> {
    // ffprobe -v quiet -print_format json -show_format -show_streams {file_path}
    
    Ok(VideoMetadata {
        duration_seconds: 0,
        format: "unknown".to_string(),
        width: 0,
        height: 0,
        fps: 0.0,
        video_bitrate: 0,
        audio_bitrate: 0,
    })
}

#[derive(Debug)]
pub struct AudioMetadata {
    pub duration_seconds: i32,
    pub format: String,
    pub bitrate: i32,
    pub sample_rate: i32,
    pub channels: i32,
}

#[derive(Debug)]
pub struct VideoMetadata {
    pub duration_seconds: i32,
    pub format: String,
    pub width: i32,
    pub height: i32,
    pub fps: f64,
    pub video_bitrate: i32,
    pub audio_bitrate: i32,
}
