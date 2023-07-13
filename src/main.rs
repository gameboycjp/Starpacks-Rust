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
    let mut game_location_filepath = current_dir.clone();
    game_location_filepath.push("sbpath.txt");

    let game_location_filepath = Path::new(&game_location_filepath);
    let mut game_location = String::new();
    let path_file_exists = game_location_filepath.exists();
    let mut path_file_correct = false;

    if path_file_exists {
        println!("Starbound path file found.");
        game_location = fs::read_to_string(game_location_filepath).unwrap();

        let mut input_check = game_location.clone();
        input_check.push_str("/assets/packed.pak");

        let input_check_file = Path::new(&input_check);
        let path_file_exists = input_check_file.exists();
        if path_file_exists {
            println!("Starbound asset found! Continuing.");
            path_file_correct = true;
        }
    } else {
        println!("Starbound path file missing.");
    }

    if !path_file_correct {
        println!("Path invalid!");
        game_location = sbpath_input();
        fs::write(game_location_filepath, &game_location)
            .expect("TODO: panic message, file not written...");
    }

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

    //workshop setup
    let mut workshop_location_filepath = current_dir.clone();
    workshop_location_filepath.push("wspath.txt");

    let workshop_location_filepath = Path::new(&workshop_location_filepath);
    let mut workshop_location = String::new();
    let wspath_file_exists = workshop_location_filepath.exists();
    let mut wspath_file_correct = false;

    if wspath_file_exists {
        println!("Workshop path file found.");
        workshop_location = fs::read_to_string(workshop_location_filepath).unwrap();

        let input_check = workshop_location.clone();

        let input_check_file = Path::new(&input_check);
        let wspath_file_exists = input_check_file.exists();
        if wspath_file_exists {
            println!("Workshop folder exists! Continuing.");
            wspath_file_correct = true;
        }
    } else {
        println!("Workshop path file missing.");
    }

    if !wspath_file_correct {
        workshop_location = wspath_input();
        fs::write(workshop_location_filepath, &workshop_location)
            .expect("TODO: panic message, file not written...");
    }

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
    wsmods.iter().for_each(|thing| {
        println!();
        let mut pack_subdir = pack_dir.clone();
        let mut game_sublocation = game_location.clone();
        pack_subdir.push("mods/".to_owned() + &thing.publishedfileid + ".pak");
        game_sublocation.push_str(&format!(
            "{}{}{}",
            "/../../workshop/content/211820/", &thing.publishedfileid, "/contents.pak"
        ));
        let mod_location = Path::new(&game_sublocation);
        println!("{}", mod_location.display());
        let mod_found = mod_location.exists();
        if !mod_found {
            println!("Mod not found! Perhaps it's not in a recognised format? Must be a singular file named contents.pak."); //TODO, FIX THIS! FFS! CHECK FOR ANY .PAK! IDIOT! DUNCE!
        } else {
            println!("{}", pack_subdir.display());
        }
        if mod_found {
            if pack_subdir.exists() {
                remove_symlink_file(&pack_subdir).unwrap();
            }
            symlink_file(game_sublocation, &pack_subdir).unwrap()
        }
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
        let mut input_check = input.clone();
        input_check.push_str("/assets/packed.pak");
        println!("{}", &input_check);
        let input_check_file = Path::new(&input_check);
        let path_file_exists = input_check_file.exists();
        if path_file_exists {
            println!("Starbound found! Continuing.");
            return input;
        }
        println!("Starbound not found! Please try again.")
    }
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
