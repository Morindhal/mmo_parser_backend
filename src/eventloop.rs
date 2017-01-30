pub mod EventLoop
{
    extern crate regex;
    extern crate libc;

    use self::regex::{Regex};
    use std::collections::HashMap;
    use std::sync::mpsc::{self, RecvTimeoutError, Receiver};

    use std::{thread, time};
    

    use std::io;
    use std::io::prelude::*;
    use std::fs::File;
    use std::io::BufReader;
    use std::io::SeekFrom;
    
    use self::libc::system;
    use std::ffi::{CString, CStr};
    use std::os::raw::c_char;

    use datastructures::EncounterStructures;
    
    /*
    * Spawns a parser for a playername on a selected file.
    * Returns a channel that should be used to recieve the parsed data.
    */
    pub fn spawn_parser(filename: String, playername: String)
        -> Receiver<Box<(bool, Vec<EncounterStructures::Attack>)>>
    {
    
        let (parse_tx, main_rx) = mpsc::channel::<Box<(bool,Vec<EncounterStructures::Attack>)>>();
        let ui = thread::spawn(move ||
        {
        
            let mut buffer = String::new();
            let mut battle_timer = time::Instant::now();
            let mut ui_update_timer = time::Instant::now();
            let mut fightdone = true;
            
            let player = playername.as_str();
            let f = File::open(filename).unwrap();
            let mut file = BufReader::new(&f);
            
            /*jump to the end of the file, negative value here will go to the nth character before the end of file.. Positive values are not encouraged.*/
            file.seek(SeekFrom::End(0));
        
            let re = Regex::new(r"\((?P<time>\d+)\)\[(?P<datetime>(\D|\d)+)\] (?P<attacker>\D*?)(' |'s |YOUR |YOU )(?P<attack>\D*)(((multi attack)|hits|hit|flurry|(aoe attack)|flurries|(multi attacks)|(aoe attacks))|(( multi attacks)| hits| hit)) (?P<target>\D+) for(?P<crittype>\D*)( of | )(?P<damage>\d+) (?P<damagetype>[A-Za-z]+) damage").unwrap();
            
            let mut attacks: Vec<EncounterStructures::Attack> = Vec::new();
            'parser: loop/*Parse file, send results to main every X secs*/
            {
                'encounter_loop: loop
                {
                    buffer.clear();
                    if file.read_line(&mut buffer).unwrap() > 0
                    {
                        /*Spawn a seperate thread to deal with the triggers*/
                        let triggerbuffer = buffer.clone();
                        thread::spawn( move || 
                        {
                            /*The container for the triggers, the key is what the tts should say, the value is the regex that is matched.*/
                            let mut triggers: HashMap<&str, Regex> = HashMap::new();
                                triggers.insert("Ruling I am", Regex::new(r".*I rule.*").unwrap());
                                triggers.insert("Verily", Regex::new(r".*i also rule.*").unwrap());
                                triggers.insert("Madness!", Regex::new(r".*Madness heals.*").unwrap());
                            for (trigger, trigged) in triggers.iter()
                            {
                                match trigged.captures(triggerbuffer.as_str()) {None => {}, Some(cap) =>
                                {
                                    speak(&CString::new(format!("espeak \"{}\"", trigger)).unwrap());
                                }};
                            }
                        });
                        match re.captures(buffer.as_str()) {None => {/*println!("{}",buffer);*/}, Some(cap) =>
                        {
                            fightdone = false;
                            attacks.push(EncounterStructures::Attack::new());
                            attacks.last_mut().unwrap().attack(&cap, match cap.name("attacker").unwrap().as_str() { "" => player, var => var});
                            //encounter.encounter_end = parsed_time; //assume every line ends the encounter, likely not optimal, needs to be overhauled
                            battle_timer = time::Instant::now();
                        }};
                    }
                    else /*Sleep for 0.1 sec if nothing has happened in the log-file*/
                    {
                        thread::sleep(time::Duration::from_millis(100));
                    }
                    /*update the UI, once every 1 sec*/
                    if ui_update_timer.elapsed() >= time::Duration::from_millis(1000) && attacks.len() != 0 && !fightdone
                    {
                        ui_update_timer = time::Instant::now();
                        parse_tx.send(Box::new((false, attacks.drain(0..).collect())));
                    }
                    /*End current encounter if nothing has been parsed in combat within the last 3 secs*/
                    if battle_timer.elapsed() >= time::Duration::from_millis(3200)
                    {
                        if !fightdone
                        {
                            attacks.clear();
                            fightdone = true;
                            parse_tx.send(Box::new((fightdone, attacks.drain(0..).collect())));
                            break 'encounter_loop;
                        }
                    }
                }
            }
        });
        main_rx
    }
    
    
    fn speak(data: &CStr)
    {
        extern { fn system(data: *const c_char); }

        unsafe { system(data.as_ptr()) }
    }

}