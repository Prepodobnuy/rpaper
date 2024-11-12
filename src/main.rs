mod argparser;
mod displays;
mod rwal;
mod templates;
mod wallpaper;
mod utils;

use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::thread;

use rand::seq::SliceRandom;
use rand::thread_rng;

use crate::utils::config::Config;
use crate::utils::directory::expand_user;
use crate::utils::name::ImageMeta;
use crate::utils::logger::log;

fn run(image_path: &str, verbose: bool) {
    let argv: Vec<String> = env::args().collect();

    let default_config_path: String = expand_user("~/.config/rpaper/config.json");
    let config_path = argv
        .iter()
        .position(|s| s == "--conf")
        .and_then(|i| argv.get(i + 1))
        .unwrap_or(&default_config_path)
        .to_string();
    let config = Config::new(&config_path, argv.iter().any(|s| s == "--cache"));
    let image_meta = ImageMeta::new(
        &image_path,
        &config.cache_dir,
        Some(&config.displays),
        Some(&config.image_operations),
        None
    );

    let mut threads = Vec::new();

    let cache_colorscheme = config.cache_scheme;
    let cache_wallpaper = config.cache_walls;
    let apply_templates = config.set_templates;
    let set_wallpaper = config.set_walls;
    let image_name = image_meta.shasum();
    let image_path = image_meta.image_path();

    if cache_colorscheme || apply_templates {
        // colorscheme & templates processing
        let image_ops = config.image_operations.clone();
        let rwal = rwal::Rwal::new(
            &image_name,
            &config.rwal_params.cache_dir,
            (config.rwal_params.thumb_w, config.rwal_params.thumb_h),
            config.rwal_params.accent,
            config.rwal_params.clamp_min,
            config.rwal_params.clamp_max,
        );
        let _colorscheme_thread = thread::spawn(move || {
            if cache_colorscheme {
                if !rwal.is_cached() {
                    if verbose {log("caching colorscheme...")};
                    let img = wallpaper::get_thumbed_image(
                        &image_path,
                        &image_ops,
                        rwal.thumb_size.0,
                        rwal.thumb_size.1,
                    );
                    rwal.uncached_run(&img.clone());
                } else {
                    rwal.cached_run();
                }
            }
            if apply_templates {
                let templates = config.templates;
                let variables =
                    templates::fill_color_variables(&config.vars_path, &config.scheme_file);
                    if verbose {log("applying templates...")};
                templates::apply_templates(templates, variables);
            }
        });
        threads.push(_colorscheme_thread);
    }

    if cache_wallpaper {
        // wallpapers processing
        let image_name = image_meta.image();
        let image_path = image_meta.image_path();
        let cache_paths = image_meta.cache_paths();
        let image_ops = config.image_operations.clone();
        let displays = config.displays.clone();
        let image_resize_algorithm = config.resize_algorithm.clone();

        let _wallpaper_thread = thread::spawn(move || {
            if cache_wallpaper {
                wallpaper::cache(
                    &image_name,
                    &image_path,
                    &cache_paths,
                    &image_ops,
                    &image_resize_algorithm,
                    &displays,
                    verbose,
                );

                if set_wallpaper {
                    if verbose {log("setting wallpapers...")};
                    wallpaper::set(
                        &displays,
                        &image_meta.cache_paths(),
                        &image_meta.image_path(),
                        &config.wall_command,
                    );
                }
            };
        });
        threads.push(_wallpaper_thread);
    }
    for thread in threads {
        thread.join().unwrap()
    }
}

fn get_absolute_path(path: PathBuf) -> PathBuf {
    path.canonicalize().unwrap_or_else(|_| path)
}

fn is_file_image(extension: &str) -> bool {
    matches!(
        extension.to_lowercase().as_str(),
        "jpg" | "jpeg" | "webp" | "png" | "gif" | "bmp" | "tiff"
    )
}

fn get_images_from_dir(dir: &str) -> Vec<String> {
    let path = Path::new(dir);
    let files = fs::read_dir(path).unwrap();
    let mut res: Vec<String> = Vec::new();

    for entry in files {
        let entry = entry.unwrap();
        let file_type = entry.file_type().unwrap();
        if file_type.is_dir() {
            res.extend(get_images_from_dir(
                &get_absolute_path(entry.path())
                    .to_string_lossy()
                    .to_string(),
            ))
        } else if file_type.is_file() {
            if let Some(extension) = entry.path().extension() {
                if is_file_image(extension.to_str().unwrap_or("")) {
                    res.push(
                        get_absolute_path(entry.path())
                            .to_string_lossy()
                            .to_string(),
                    )
                }
            }
        }
    }
    res
}

fn main() {
    let argv: Vec<String> = env::args().collect();
    if argv.len() == 1 || argv.iter().any(|arg| arg == "--help") {
        utils::helpmessages::help_message();
        return;
    }

    if argv.contains(&String::from("-V")) {
        utils::helpmessages::version();
        return;
    }
    
    let verbose = argv.contains(&String::from("-v"));

    let path = Path::new(&argv[1]);
    if path.is_dir() {
        if verbose {log("looking for images...")};
        let images = get_images_from_dir(&argv[1]);
        if argv.contains(&"--cache".to_string()) {
            if verbose {log("caching images...")};
            for chunk in images.chunks(6) {
                let mut threads = Vec::new();

                for image in chunk {
                    let image = image.clone();
                    let thread = thread::spawn(move || run(&image, verbose));
                    threads.push(thread);
                }

                for thread in threads {
                    thread.join().unwrap();
                }
            }
        } else {
            let mut rng = thread_rng();
            let random_image = images.choose(&mut rng).cloned();

            if let Some(ref random_image) = random_image {
                if verbose {log(&format!("selected image: {}", &random_image))};
                run(random_image, verbose)
            }
        }
    } else if path.is_file() {
        run(&argv[1], verbose)
    }
}
