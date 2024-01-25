/*TODO: IMPLEMENT
"anyway uhh for my server, to run different modpacks I just have different sbinit.config files
you can then just pass that to the client / server executable by doing like run-client.sh -bootconfig <some path to an sbinit.config>
so for a modpack launcher I'd just have it create a mods and storage folder and an sbinit file
and then just pass that sbinit file to the client
there are some other options you could take a look at by just doing run-client.sh -help for example"*/


use copy_dir::copy_dir;
#[cfg(target_family = "windows")]
use is_elevated::is_elevated;
#[cfg(target_family = "windows")]
use std::process;
use symlink::*;
use std::path::Path;
use std::{env, fs, io};
use steam_workshop_api::{Workshop, WorkshopItem};
use serde::{Serialize, Deserialize};


#[derive(Serialize, Deserialize)]
struct MainConfig {
    game_location: Option<String>,
    workshop_location: Option<String>
}

impl MainConfig {
    fn empty () -> Self {
        MainConfig{game_location: None, workshop_location: None}
    }
    fn from_toml (main_config_contents: &str) -> Self {
        toml::from_str(main_config_contents).unwrap()
    }
}

#[derive(Serialize, Deserialize)]
struct PackConfig {
    pack_name: String,
    workshop_id: String
}


static DEFAULT_PACK: &str = "defaultpack";

fn main() {
    #[cfg(target_family = "windows")] {
        if !is_elevated() {
            println!("This program must be run as Administator on Windows, as it makes heavy use of symlinks. Hit enter to exit."); //I cannot stand this OS.
            pause();
            process::exit(1);
        }
    }

    //prep work
    let current_dir = env::current_dir().unwrap();
    let mut main_config_filepath = current_dir.clone();
    main_config_filepath.push("starpacksconfig.toml");

    let mut main_config = MainConfig::empty();
    let main_config_filepath = Path::new(&main_config_filepath);
    let main_config_file_exists = main_config_filepath.exists();
    if !main_config_file_exists {
        let game_location = sbpath_input();
        main_config.game_location = Some(game_location);
        let workshop_location = wspath_input();
        main_config.workshop_location = Some(workshop_location);
        let main_config_toml = toml::to_string_pretty(&main_config).unwrap();
        fs::write(main_config_filepath, main_config_toml).unwrap();

    } else {
        let mut game_path_correct = false;
        let main_config_toml = fs::read_to_string(main_config_filepath).unwrap();
        main_config = MainConfig::from_toml(&main_config_toml);
        if main_config.game_location.is_some() {
            game_path_correct = sbpath_check(main_config.game_location.unwrap())
        }
        if !game_path_correct {
            let game_location = sbpath_input();
            main_config.game_location = Some(game_location);
        }
        if !main_config.workshop_location.is_some() {
            let workshop_location = wspath_input();
            main_config.workshop_location = Some(workshop_location)
        }
    }

    let game_location = main_config.game_location.unwrap();

    println!("Game path: {}", game_location);

    //pre-workshop setup
    let pack_name = input("Please enter modpack name.");

    let mut pack_dir = env::current_dir().unwrap();
    pack_dir.push(pack_name);
    println!("Pack location: {}", pack_dir.display());
    let link_dir_list = ["assets", "doc", "tiled"];
    if !pack_dir.exists() {
        println!("Pack doesn't exist! Creating from game files.");
        fs::create_dir(&pack_dir).expect("Could not create main pack folder!");
        #[cfg(target_family = "unix")]
        fs::create_dir(&format!("{}{}", &pack_dir.display(), "/linux")).expect("Could not create linux folder!");
        #[cfg(target_family = "windows")]
        fs::create_dir(&format!("{}{}", &pack_dir.display(), "/win64")).expect("Could not create win64 folder!");
        fs::create_dir(&format!("{}{}", &pack_dir.display(), "/mods")).expect("Could not create mods folder!");
        fs::create_dir(&format!("{}{}", &pack_dir.display(), "/storage")).expect("Could not create storage folder!");
        #[cfg(target_family = "unix")]
        let copy_list = [
            "linux/sbinit.config",
            "linux/run-client.sh",
            "linux/run-server.sh",
            "linux/starbound_server",
            "linux/starbound",
        ];
        #[cfg(target_family = "windows")]
        let copy_list = [
            "win64/sbinit.config",
            "win64/starbound_server.exe",
            "win64/starbound.exe",
        ];
        let mut link_list_search_directory = game_location.clone() + "/";
        #[cfg(target_family = "unix")]
        let platstring = "linux/";
        #[cfg(target_family = "windows")]
        let platstring = "win64/";
        link_list_search_directory.push_str(platstring);
        let link_file_list2 = fs::read_dir(&link_list_search_directory).unwrap();
        let mut link_file_list = Vec::new();
        for index in link_file_list2.into_iter() {
            let new_val = index.unwrap().file_name();
            link_file_list.push(new_val);
        }

        link_dir_list.iter().for_each(|thing| {
            let mut pack_subdir = pack_dir.clone();
            pack_subdir.push(thing);
            symlink_dir(game_location.to_owned() + "/" + thing, pack_subdir).unwrap()
        });
        link_file_list.into_iter().for_each(|thing| {
            let mut pack_subdir = pack_dir.clone();
            pack_subdir.push(platstring);
            pack_subdir.push(&thing);
            let mut should_link = true;

            let symlink_source = game_location.to_owned() + "/" + platstring + &thing.clone().to_str().unwrap();
            for i in copy_list.iter() {
                if symlink_source.clone().contains(i) {
                    should_link = false;
                }
            }
            if should_link {
                symlink_auto(Path::new(&symlink_source), pack_subdir).unwrap();
            }
        });
        copy_list.iter().for_each(|thing| {
            let mut pack_subdir = pack_dir.clone();
            pack_subdir.push(thing);
            copy_dir(game_location.to_owned() + "/" + thing, pack_subdir).unwrap();
        });
    } else {
        println!("Pack folder exists! Relinking game files.");
        link_dir_list.iter().for_each(|thing| {
            let mut pack_subdir = pack_dir.clone();
            pack_subdir.push(thing);
            if pack_subdir.exists() {
                remove_symlink_dir(&pack_subdir).unwrap();
            }
          symlink_dir(game_location.to_owned() + "/" + thing, &pack_subdir).unwrap()
        });
    }

    let workshop_location = main_config.workshop_location.unwrap();

    println!("Workshop path: {}", workshop_location);

    let collection_id =
        input("Please enter the Workshop collection ID of the pack you want to use.");
    println!("Starting workshop API!");
    let wsclient: Workshop = Workshop::new(None);
    let wscollection: Vec<String> = wsclient
        .get_collection_details(&collection_id)
        .unwrap()
        .unwrap();
    let wsmods: Vec<WorkshopItem> = wsclient.get_published_file_details(&wscollection).unwrap();
    wsmods.iter().for_each(|ws_item| {
        println!();
        let mut ws_sublocation = workshop_location.clone();
        ws_sublocation.push_str(&format!(
            "{}{}",
            "/", &ws_item.publishedfileid
        ));
        let ws_file_list2 = fs::read_dir(&ws_sublocation).unwrap();
        let mut ws_file_list = Vec::new();
        for index in ws_file_list2.into_iter() {
            let new_val = index.unwrap().file_name();
            println!("{}", new_val.clone().into_string().unwrap());
            ws_file_list.push(new_val);
        }
        ws_file_list.into_iter().for_each(|thing| {
            let mut pack_subdir = pack_dir.clone();
            let thing_string = thing.to_str().unwrap();
            let mut mod_location = ws_sublocation.clone().to_string();
            mod_location.push_str("/");
            mod_location.push_str(thing_string);
            let mod_location_path = Path::new(&mod_location);
            pack_subdir.push("mods/".to_owned() + &ws_item.publishedfileid + thing_string);
            println!("{}", mod_location);
            let mod_found = mod_location_path.exists();
            if !mod_found {
                println!("Mod file not found! This should never print!");
            } else {
                println!("{}", pack_subdir.display());
            }
            if mod_found {
                if pack_subdir.exists() {
                    remove_symlink_file(&pack_subdir).unwrap();
                }
                symlink_file(&mod_location_path, &pack_subdir).unwrap()
            }
        });
    });

    println!("Complete! Take another look at the above logging to check if anything went wrong.");
    pause();
}

fn pause() {
        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read input");
}

fn sbpath_input() -> String {
    loop {
        println!("Please enter Starbound path.");

        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read input");
        let mut input = input.trim().to_string();
        if input.chars().last() == Some('/') {
            input.pop();
        }
        let input_check = input.clone();
        let path_file_exists = sbpath_check(input_check);
        if path_file_exists {
            println!("Starbound found! Continuing.");
            return input;
        }
        println!("Starbound not found! Please try again.")
    }
}

fn sbpath_check(mut input_check: String) -> bool {
    input_check.push_str("/assets/packed.pak");
    println!("{}", &input_check);
    let input_check_file = Path::new(&input_check);
    let path_file_exists = input_check_file.exists();
    return path_file_exists;
}

fn wspath_input() -> String {
    loop {
        println!("Please enter Workshop path.");

        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read input");
        let mut input = input.trim().to_string();
        if input.chars().last() == Some('/') {
            input.pop();
        }
        println!("TODO: Need a path check for Workshop."); //TODO: Probably take whatever folder is easy to reach and check for the contents.pak? TODO2: or any pak
        return input;
    }
}

fn input(prompt: &str) -> String {
    println!("{}", prompt);

    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .expect("Failed to read input");
    let mut input = input.trim().to_string();
    if input.is_empty() {
        input = String::from(DEFAULT_PACK)
    }
    input
}
