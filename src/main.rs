use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::thread;

use rand::seq::SliceRandom;
use rand::thread_rng;

use crate::config::{ArgvParser, Config};

mod config;
mod displays;
mod rwal;
mod templates;
mod utils;
mod wallpaper;

fn run(image_path: &str) {
    let argv_parser = ArgvParser::new();

    let default_config_path: String = utils::parse_path("~/.config/rpaper/config.json");
    let config_path = argv_parser.get_config_path(default_config_path);
    let image_path = String::from(image_path);
    let image_name = utils::get_image_name(&image_path);

    let config = Config::new(&config_path, argv_parser);

    let mut threads = Vec::new();

    let cache_colorscheme = config.cache_colorscheme;
    let apply_templates = config.apply_templates;
    if cache_colorscheme || apply_templates {
        // colorscheme & templates processing
        let image_ops = config.image_operations.clone();
        let img_path = image_path.clone();
        let color_scheme_file = config.color_scheme_file;
        let templates = config.templates;
        let variables = config.variables;
        let rwal = rwal::Rwal::new(
            &utils::get_img_ops_affected_name(&image_name, &image_ops),
            &config.rwal_cache_dir,
            config.rwal_thumb,
            config.rwal_accent_color,
            config.rwal_clamp_min_v,
            config.rwal_clamp_max_v,
        );
        let _colorscheme_thread = thread::spawn(move || {
            if cache_colorscheme {
                if !rwal.is_cached() {
                    let img = wallpaper::get_thumbed_image(
                        &img_path,
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
                templates::apply_templates(templates, variables, color_scheme_file);
            }
        });
        threads.push(_colorscheme_thread);
    }

    let cache_wallpaper = config.cache_wallpaper;
    let set_wallpaper = config.set_wallpaper;
    if cache_wallpaper {
        // wallpapers processing
        let image_path = image_path.clone();
        let image_ops = config.image_operations.clone();
        let displays = config.displays.clone();
        let cached_wallpapers_names =
            wallpaper::get_cached_images_names(&displays, &image_name, &image_ops);
        let cached_wallpapers_paths = wallpaper::get_cached_images_paths(
            &cached_wallpapers_names,
            &config.cached_images_path,
        );
        let image_resize_algorithm = config.wallpaper_resize_backend.clone();

        let _wallpaper_thread = thread::spawn(move || {
            if cache_wallpaper {
                wallpaper::cache(
                    &image_path,
                    &image_name,
                    &image_ops,
                    &image_resize_algorithm,
                    &displays,
                    &cached_wallpapers_paths,
                );

                if set_wallpaper {
                    wallpaper::set(
                        &displays,
                        &cached_wallpapers_paths,
                        &config.set_wallpaper_command,
                    );
                }
            }
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
            res.push(
                get_absolute_path(entry.path())
                    .to_string_lossy()
                    .to_string(),
            )
        }
    }
    res
}

fn main() {
    let argv: Vec<String> = env::args().collect();
    if argv.len() == 1 {
        return;
    }
    let path = Path::new(&argv[1]);
    if path.is_dir() {
        let images = get_images_from_dir(&argv[1]);
        if argv.contains(&String::from("--cache")) {
            for chunk in images.chunks(6) {
                let mut threads = Vec::new();

                for image in chunk {
                    let image = image.clone();
                    let thread = thread::spawn(move || run(&image));
                    threads.push(thread);
                }

                for thread in threads {
                    thread.join().unwrap();
                }
            }
        } else {
            let mut rng = thread_rng();
            let random_image = images.choose(&mut rng).cloned();

            match random_image {
                Some(random_image) => run(&random_image),
                _ => {}
            }
        }
    } else if path.is_file() {
        run(&argv[1])
    }
}
