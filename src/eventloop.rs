pub mod EventLoop
{
    extern crate regex;
    extern crate libc;

    use self::regex::{Regex};
    use std::collections::HashMap;
    use std::sync::mpsc::{self, RecvTimeoutError, Receiver, Sender};

    use std::{thread, time};
    

    use std::io;
    use std::io::prelude::*;
    use std::fs::File;
    use std::io::BufReader;
    use std::io::SeekFrom;
    
    use self::libc::system;
    use std::ffi::{CString, CStr};
    use std::os::raw::c_char;
    
    use json::JsonValue;

    use datastructures::EncounterStructures::*;
    use parserfunctions::ParserFunctions::getTime;
    
    /*
    * Spawns a parser for a playername on a selected file.
    * Returns a channel that should be used to recieve the parsed data.
    */
    pub fn new(filename: String, playername: String)
        -> (Sender<Box<JsonValue>>, Receiver<Box<JsonValue>>)
    {
    
        let (parse_tx, main_rx) = mpsc::channel::<Box<(bool,Vec<Attack>)>>();
        let (event_poller_tx, event_poller_rx) = mpsc::channel::<Box<JsonValue>>();
        let (event_responder_tx, event_responder_rx) = mpsc::channel::<Box<JsonValue>>();
        communication((event_responder_tx, event_poller_rx), main_rx);
        let ui = thread::spawn(move ||
        {
            /*
            * Create a call to another function here that acts as a event-parser, recieves events from all over and stores/responds
            * differently depending on what the event is.
            * 
            * The data to/from the UI should be in JSON.
            * This event-parser also stores the data parsed from the log-file, similar to how main.rs used to work.
            * 
            * parse_tx and main_rx should be used to dump the data parsed from the log-file into the event-parsers data-storage.
            * Another 2 channels should be created that handles sending data to the event-parser from the UI and the vice versa.
            */
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
            
            let mut attacks: Vec<Attack> = Vec::new();
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
                            attacks.push(Attack::new());
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
        (event_poller_tx, event_responder_rx)
    }
    
    fn communication((to_ui, from_ui):(Sender<Box<JsonValue>>, Receiver<Box<JsonValue>>), from_parser: Receiver<Box<(bool, Vec<Attack>)>>)
    {
        thread::spawn(move ||
        {
            let timeout = time::Duration::from_millis(1);
            let mut encounters: Vec<CombatantList> = Vec::new();
            encounters.push(CombatantList::new(getTime("default_time")));
            'communication: loop
            {
                match from_parser.recv_timeout(timeout)
                {
                    Ok(val) => 
                    {
                        if val.0
                        {
                            encounters.push(CombatantList::new(getTime("default_time")));
                        }
                        for attack in val.1
                        {
                            encounters.last_mut().unwrap().attack(attack);
                        }
                        if encounters.len() != 0
                        {encounters.last_mut().unwrap().encounter_duration = (encounters.last().unwrap().encounter_end-encounters.last().unwrap().encounter_start).num_seconds() as u64;}
                    },
                    Err(e) => {}
                }
                /*
                * from_ui asks for what type of data it wants by sending a json object with the request
                * to_ui responds with a json-encoded object containing all relevant data
                * this is sent only once. ?
                */
                match from_ui.recv_timeout(timeout)
                {
                    Ok(json) =>
                    {
                        println!("{}", (*json).dump());
                        to_ui.send( Box::new( object!{"test" => true} ) );
                    },
                    Err(e) => {}
                }
            }
        
        });
    }
    
    fn speak(data: &CStr)
    {
        extern { fn system(data: *const c_char); }

        unsafe { system(data.as_ptr()) }
    }

}