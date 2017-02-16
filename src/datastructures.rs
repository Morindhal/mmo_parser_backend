
pub mod encounter_structures
{
    extern crate regex;
    extern crate chrono;
    extern crate json;
    
    use std::cmp::Ordering;
    use std::fmt;
    use self::chrono::*;
    
    use parserfunctions::parser_functions::get_time;

    #[derive(Eq, Clone)]
    pub struct Attack
    {
        attacker: String,
        damage: u64,
        victim: String,
        pub timestamp: String,
        pub attack_name: String,
        crit: String, // "" for did not crit?
        damage_type: String
    }

    impl Attack
    {
        pub fn attack(&mut self, attack_data: &regex::Captures, attacker: &str)
        {
            self.attacker = String::from(attacker);
            self.damage = attack_data.name("damage").unwrap().as_str().parse::<u64>().unwrap();
            self.victim = String::from(attack_data.name("target").unwrap().as_str());
            self.timestamp = String::from(attack_data.name("datetime").unwrap().as_str());
            self.attack_name = String::from(match attack_data.name("attack").unwrap().as_str() { "" => "auto attack", val => val } );
            self.crit = String::from(attack_data.name("crittype").unwrap().as_str());
            self.damage_type = String::from(attack_data.name("damagetype").unwrap().as_str());
        }
        
        pub fn filter(&self, filters: &str, attacker: &String) -> bool
        {
            if !self.attacker.contains(attacker) {return false;}
            if filters.len() as i32 != 0
            {
                for filter in filters.split_whitespace()
                {
                    if !self.timestamp.contains(filter) && !self.victim.contains(filter) && !self.attack_name.contains(filter) && !self.crit.contains(filter) && !self.damage_type.contains(filter)  {return false;}
                }
            }
            true
        }
        
        pub fn new()
            -> Attack
        {
            Attack
            {
                attacker: String::from("undefined"),
                damage: 0,
                victim: String::from("undefined"),
                timestamp: String::from("undefined"),
                attack_name: String::from("undefined"),
                crit: String::from("undefined"),
                damage_type: String::from("undefined")
            }
        }
        
        pub fn jsonify(&self)
            -> json::JsonValue
        {
            object!{
                "Attacker" => self.attacker.clone(),
                "Damage" => self.damage.clone(),
                "Victim" => self.victim.clone(),
                "Time" => format!("{}", get_time(self.timestamp.as_str())),
                "AttackName" => self.attack_name.clone(),
                "Crit" => self.crit.clone(),
                "DamageType" => self.damage_type.clone()
            }
        }
    }

    impl Ord for Attack
    {
        fn cmp(&self, other: &Attack) -> Ordering
        {
            self.damage.cmp(&other.damage)
        }
    }

    impl PartialOrd for Attack
    {
        fn partial_cmp(&self, other: &Attack) -> Option<Ordering>
        {
            Some(self.cmp(other))
        }
    }

    impl PartialEq for Attack
    {
        fn eq(&self, other: &Attack) -> bool
        {
            self.damage == other.damage
        }
    }

    impl fmt::Display for Attack
    {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result
        {
            write!(f, "{:25.25}   VICTIM: {:20.20}   ATTACK: {:30.30}   DAMAGE: {:>15.15}   CRIT: {:>20.20}   TYPE: {:>10.10}", self.timestamp, self.victim, self.attack_name, self.damage, self.crit, self.damage_type)
            //write!(f, "{}", self.timestamp, self.victim, self.attack_name)
        }
    }


    #[derive(Eq)]
    pub struct AttackStats
    {
        name: String,
        attack_nmbr: usize,
        total_damage: u64
    }

    impl AttackStats
    {
        pub fn find_attackname(&mut self, attacks: &Vec<Attack>, attack_nmbr: usize)
            -> bool
        {
            if self.name == attacks[attack_nmbr].attack_name
            {
                if attacks[self.attack_nmbr].damage < attacks[attack_nmbr].damage
                {self.attack_nmbr = attack_nmbr;}
                self.total_damage += attacks[attack_nmbr].damage;
                true
            }
            else
            {false}
        }
        
        pub fn print(&self, duration: u64, all_damage: u64, attacks: &Vec<Attack>)
            -> String
        {
            format!("{:6.2} procent of parse   {}\n", (self.total_damage as f64 / all_damage as f64 * 100.0), (attacks[self.attack_nmbr]))
        }

        pub fn new(attacks: &Vec<Attack>, attack_nmbr: usize)
            -> AttackStats
        {
            AttackStats{name: attacks[attack_nmbr].attack_name.clone(), attack_nmbr: attack_nmbr, total_damage: attacks[attack_nmbr].damage}
        }

        pub fn jsonify(&self)
            -> json::JsonValue
        {
            object!{
                "Name" => format!("{}", self.name),
                "HighestHit" => self.attack_nmbr,
                "TotalDamage" => self.total_damage
            }
        }
    }

    impl Ord for AttackStats
    {
        fn cmp(&self, other: &AttackStats) -> Ordering
        {
            other.total_damage.cmp(&self.total_damage)
        }
    }

    impl PartialOrd for AttackStats
    {
        fn partial_cmp(&self, other: &AttackStats) -> Option<Ordering>
        {
            Some(self.cmp(other))
        }
    }

    impl PartialEq for AttackStats
    {
        fn eq(&self, other: &AttackStats) -> bool
        {
            other.total_damage == self.total_damage
        }
    }



    #[derive(Eq)]
    pub struct Attacker
    {
        attacks: Vec<Attack>,
        final_damage: u64,
        final_healed: u64,
        pub name: String
    }

    impl Ord for Attacker
    {
        fn cmp(&self, other: &Attacker) -> Ordering
        {
            other.final_damage.cmp(&self.final_damage)
        }
    }

    impl PartialOrd for Attacker
    {
        fn partial_cmp(&self, other: &Attacker) -> Option<Ordering>
        {
            Some(self.cmp(other))
        }
    }

    impl PartialEq for Attacker
    {
        fn eq(&self, other: &Attacker) -> bool
        {
            self.final_damage == other.final_damage
        }
    }

    impl Attacker
    {
        pub fn attack(&mut self, attack_data: &regex::Captures)
        {
            //self.attacks.push(Attack{damage: attack_data.name("damage").unwrap().parse::<u64>().unwrap(), victim: String::from(attack_data.name("target").unwrap()), timestamp: String::from(attack_data.name("datetime").unwrap()), attack_name: String::from(match attack_data.name("attack").unwrap() { "" => "auto attack", val => val } ), crit: String::from(attack_data.name("crittype").unwrap()), damage_type: String::from(attack_data.name("damagetype").unwrap())});
            self.final_damage += attack_data.name("damage").unwrap().as_str().parse::<u64>().unwrap();
        }
    }

    impl Clone for Attacker
    {
        fn clone(&self) -> Attacker{ Attacker{attacks: self.attacks.clone(), final_damage: self.final_damage, final_healed: self.final_healed, name: self.name.clone()} }
    }


    pub struct CombatantList
    {
        pub combatants: Vec<Combatant>,
        pub attacks: Vec<Attack>,
        pub attack_stats: Vec<AttackStats>,
        pub encounter_start: DateTime<UTC>,
        pub encounter_end: DateTime<UTC>,
        pub encounter_duration: u64,
        pub highest_hit: Attack,
        pub highest_heal: Attack
    }

    impl CombatantList
    {
        pub fn attack(&mut self, attack: Attack)
        {
            if self.attacks.len() == 0
            {self.encounter_start = get_time(attack.timestamp.as_str());}
            self.encounter_end = get_time(attack.timestamp.as_str());

            
            match self.find_combatant(attack.attacker.as_str())
            {
                -1 =>/*New attacker*/
                    {
                        self.combatants.push(Combatant{name: attack.attacker.clone(), highest_hit: Attack::new(), highest_heal: Attack::new(), final_healed: 0, final_damage: 0, attack_stats: Vec::new(), combatstart: get_time(attack.timestamp.as_str()), sort_by_dps: true});
                        self.attacks.push(attack);
                        self.combatants.last_mut().unwrap().attack(&self.attacks, self.attacks.len()-1);
                        self.combatants.last_mut().unwrap().final_damage += self.attacks.last().unwrap().damage;
                    },
                i =>
                {
                    self.combatants[i as usize].final_damage += attack.damage;
                    self.attacks.push(attack);
                    self.combatants[i as usize].attack(&self.attacks, self.attacks.len()-1);
                },
            };
            /*enter the attack data into a list that keeps track of specific attacks
            * This list MUST also be entered on a player-level, create one list-struct for both?
            */
            {
                let mut exists = false;
                for stats in self.attack_stats.iter_mut()
                {
                    exists = stats.find_attackname(&self.attacks, self.attacks.len()-1);
                    if exists {break;}
                }
                if !exists
                {self.attack_stats.push(AttackStats::new(&self.attacks, self.attacks.len()-1));}
            }
        }
        
        pub fn find_combatant(&mut self, attacker: &str)
            -> i32
        {
            for i in 0..self.combatants.len()
            {
                if self.combatants[i].name == attacker
                {return i as i32;}
            }
            -1
        }
        
        pub fn new(start: DateTime<UTC>)
            -> CombatantList
        {
            CombatantList{combatants: Vec::new(), attacks: Vec::new(), attack_stats: Vec::new(), encounter_start: start, encounter_end: start, encounter_duration: 0, highest_hit: Attack::new(), highest_heal: Attack::new()}
        }
        
        pub fn print_attacks(&self, filters: &str, player: &String) -> String
        {
            let mut results: String = String::from("");
            for attack in &self.attacks
            {
                if attack.filter(filters, &player)
                {
                    results.push_str(&format!("{}\n", attack));
                }
            }
            results
        }

        pub fn print_attack_stats(&self, player: &str) -> String
        {
            let mut results: String = String::from("");
            for combatant in &self.combatants
            {
                if combatant.name == player
                {
                    for stats in &combatant.attack_stats
                    {
                        results.push_str(&format!("{}", stats.print((self.encounter_end-self.encounter_start).num_seconds() as u64, combatant.final_damage, &self.attacks)));
                    }
                }
            }
            results
        }
        
        pub fn jsonify(&self)
            -> json::JsonValue
        {
            let duration = self.encounter_end-self.encounter_start;
            object!{
                "EndTime" => format!("{}", self.encounter_end),
                "StartTime" => format!("{}", self.encounter_start),
                "Name" => "Temporary name",
                "Duration" => &*format!("{}:{:02}\n", duration.num_minutes(), duration.num_seconds() % 60 )
            }
        }
    }

    #[derive(Eq)]
    pub struct Combatant
    {
        pub name: String,
        pub highest_hit: Attack,
        pub highest_heal: Attack,
        pub final_healed: u64,
        pub final_damage: u64,
        pub attack_stats: Vec<AttackStats>,
        pub combatstart: DateTime<UTC>,
        pub sort_by_dps: bool
    }

    impl Ord for Combatant
    {
        fn cmp(&self, other: &Combatant) -> Ordering
        {
            if self.sort_by_dps
            {
                other.final_damage.cmp(&self.final_damage)
            }
            else
            {
                other.final_healed.cmp(&self.final_healed)
            }
        }
    }

    impl PartialOrd for Combatant
    {
        fn partial_cmp(&self, other: &Combatant) -> Option<Ordering>
        {
            Some(self.cmp(other))
        }
    }

    impl PartialEq for Combatant
    {
        fn eq(&self, other: &Combatant) -> bool
        {
            if self.sort_by_dps
            {
                other.final_damage == self.final_damage
            }
            else
            {
                other.final_healed == self.final_healed
            }
        }
    }

    impl Combatant
    {
        pub fn attack(&mut self, attacks: &Vec<Attack>, attack_nmbr: usize)
        {
            let mut exists = false;
            for stats in self.attack_stats.iter_mut()
            {
                exists = stats.find_attackname(&attacks, attack_nmbr);
                if exists {break;}
            }
            if !exists
            {self.attack_stats.push(AttackStats::new(&attacks, attack_nmbr));}
            self.attack_stats.sort();
        }
        
        pub fn jsonify(&self)
            -> json::JsonValue
        {
            object!{
                "Name" => self.name.clone(),
                "highest_hit" => self.highest_hit.jsonify(),
                "highest_heal" => self.highest_heal.jsonify(),
                "Healed" => self.final_healed,
                "Damage" => self.final_damage,
                "CombatStart" => format!("{}", self.combatstart)
            }
        }
    }
}


