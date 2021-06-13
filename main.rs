extern crate nbt;
extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate failure;

use std::cmp::Ordering;
use std::fmt;
use std::fs::File;
use std::fs;
use std::io::prelude::*;
use std::path::PathBuf;

use serde_json::Value;

use failure::ResultExt;

const AMOUNT: usize = 300;

type Result<T> = std::result::Result<T, failure::Error>;

fn main() -> Result<()> {
    let files = list_stats_files("./stats").unwrap();

    let mut stats = Vec::with_capacity(files.len());

    for file in files {
        stats.push(Player::new(&file)
                   .with_context(|_| format!("while handling player from file {}", file.to_string_lossy()))?);
    }

    stats.sort();

    let take = if stats.len() > AMOUNT {
        AMOUNT
    } else {
        stats.len()
    };
    let stats = &stats[..take];

    /* The static numbers column commented out because it doesn't align properly
     * https://meta.wikimedia.org/wiki/Help:Sorting#Static_column
    println!(r#"{{|"#);
    println!("|-");
    println!("|");
    println!(r#"{{| class="wikitable" style="margin-right:0""#);
    println!("! Number");
    for i in 1..AMOUNT+1 {
        println!("|-");
        println!("! {}", i);
    }
    println!("|}}");
    println!("|");
    */

    println!(r#"{{| class="wikitable sortable" style="margin-left:0""#);
    println!("|-");
    println!(r#"! Player !! Play time (hours) !! Games quit !! Jumps !! Deaths !! Damage taken (half hearts) !! Damage dealt (half hearts) !! Mob kills !! Player kills !! Traveled (km) !! Cake slices eaten !!data-sort-type="number" | Advancements !! Villagers Traded With !! Stone Mined !! Obsidian Mined !! Cobblestone mined !! Netherrack mined !! Spawners mined !! Ender Dragons Killed !! Withers Killed !! Elder Guardians Killed !! Evokers Killed !! Skeleton Horses Killed !! Piglin Brutes Killed !! Dirt Mined !! Sand Mined"#);
    for stat in stats {
        println!("|-");
        println!("{}", &stat);
    }
    println!("|}}");

    Ok(())
}

/// Returns a list of paths to each of the json files in the stats directory
fn list_stats_files(dir: &str) -> Result<Vec<PathBuf>> {
    Ok(fs::read_dir(dir)?
       .map(|x| x.unwrap().path().to_path_buf())
       .filter(|x| {
           match x.file_name() {
               /* json stats file names have exactly 41 characters */
               Some(name) if name.len() == 41 => (),
               _ => return false,
           }
           match x.extension() {
               Some(extension) if extension == "json" => true,
               _ => false,
           }
       })
       .collect())
}

/// Read the given advancements file, returning the number of gained achievements
fn count_advancements(path: &PathBuf) -> Result<u64> {
    let mut f = match File::open(path) {
        Ok(f) => f,
        /* If file is not found, the player has 0 advancements.
         * This happens when the player has not logged in since advancements
         * were added to the game */
        Err(ref e) if e.kind() == std::io::ErrorKind::NotFound => {
            return Ok(0);
        },
        Err(e) => {
            panic!(e);
        },
    };
    let mut tmp = String::new();
    f.read_to_string(&mut tmp)?;
    let json: Value = serde_json::from_str(&tmp)?;
    let json = json.as_object().unwrap();
    /* json is now a map with the keys being the names of the advancements,
     * and the values being a json object of details over the advancement */

    let mut count = 0;

    for (name, value) in json {
        if name.starts_with("minecraft:story") ||
            name.starts_with("minecraft:nether") ||
            name.starts_with("minecraft:end") ||
            name.starts_with("minecraft:adventure") ||
            name.starts_with("minecraft:husbandry")
        { }
        else {
            continue;
        }

        match value.get("done") {
            Some(x) if x.is_boolean() => {
                if x.as_bool().expect("wasn't really bool") {
                    count += 1;
                }
            },
            _ => (),
        }
    }

    Ok(count)
}

#[derive(Deserialize, Debug, PartialEq, Eq)]
struct Player {
    #[serde(skip)]
    playername: String,
    #[serde(skip)]
    advancements_count: u64,
    #[serde(skip)]
    uuid: String,
    #[serde(default)]
    stats: Stats,
    #[serde(flatten)]
    oldstats: OldStats,
}

#[derive(Deserialize, Default, Debug, PartialEq, Eq)]
struct Stats {
    /* May add other fields here, such as minecraft:dropped */
    #[serde(rename = "minecraft:custom")]
    custom: Custom,
    #[serde(rename = "minecraft:killed", default)]
    killed: Killed,
    #[serde(rename = "minecraft:mined", default)]
    mined: Mined,
}

#[derive(Deserialize, Default, Debug, PartialEq, Eq)]
struct Custom {
    #[serde(rename = "minecraft:play_one_minute")]
    play_time: u64,
    #[serde(rename = "minecraft:jump", default)]
    jumps: u64,
    #[serde(rename = "minecraft:deaths", default)]
    deaths: u64,
    #[serde(rename = "minecraft:damage_taken", default)]
    damage_taken: u64,
    #[serde(rename = "minecraft:damage_dealt", default)]
    damage_dealt: u64,
    #[serde(rename = "minecraft:mob_kills", default)]
    mob_kills: u64,
    #[serde(rename = "minecraft:player_kills", default)]
    player_kills: u64,
    #[serde(rename = "minecraft:eat_cake_slice", default)]
    cake_slices: u64,
    #[serde(rename = "minecraft:leave_game", default)]
    leave_game: u64,
    #[serde(rename = "minecraft:walk_one_cm", default)]
    walk: u64,
    #[serde(rename = "minecraft:crouch_one_cm", default)]
    crouch: u64,
    #[serde(rename = "minecraft:sprint_one_cm", default)]
    sprint: u64,
    #[serde(rename = "minecraft:swim_one_cm", default)]
    swim: u64,
    #[serde(rename = "minecraft:fall_one_cm", default)]
    fall: u64,
    #[serde(rename = "minecraft:climb_one_cm", default)]
    climb: u64,
    #[serde(rename = "minecraft:fly_one_cm", default)]
    fly: u64,
    #[serde(rename = "minecraft:walk_on_water_one_cm", default)]
    walk_on_water: u64,
    #[serde(rename = "minecraft:minecart_one_cm", default)]
    minecart: u64,
    #[serde(rename = "minecraft:boat_one_cm", default)]
    boat: u64,
    #[serde(rename = "minecraft:pig_one_cm", default)]
    pig: u64,
    #[serde(rename = "minecraft:horse_one_cm", default)]
    horse: u64,
    #[serde(rename = "minecraft:aviate_one_cm", default)]
    aviate: u64,
    #[serde(rename = "minecraft:traded_with_villager", default)]
    traded_with_villager: u64,
}

#[derive(Deserialize, Default, Debug, PartialEq, Eq)]
struct Mined {
    #[serde(rename = "minecraft:stone", default)]
    stone: u64,
    #[serde(rename = "minecraft:obsidian", default)]
    obsidian: u64,
    #[serde(rename = "minecraft:cobblestone", default)]
    cobblestone: u64,
    #[serde(rename = "minecraft:netherrack", default)]
    netherrack: u64,
    #[serde(rename = "minecraft:spawner", default)]
    spawner: u64,
    #[serde(rename = "minecraft:dirt", default)]
    dirt: u64,
    #[serde(rename = "minecraft:sand", default)]
    sand: u64,
}

#[derive(Deserialize, Default, Debug, PartialEq, Eq)]
struct Killed {
    #[serde(rename = "minecraft:ender_dragon", default)]
    ender_dragon: u64,
    #[serde(rename = "minecraft:wither", default)]
    wither: u64,
    #[serde(rename = "minecraft:elder_guardian", default)]
    elder_guardian: u64,
    #[serde(rename = "minecraft:evoker", default)]
    evoker: u64,
    #[serde(rename = "minecraft:skeleton_horse", default)]
    skeleton_horse: u64,
    #[serde(rename = "minecraft:piglin_brute", default)]
    piglin_brute: u64,
}

/// Represents stats files in the old pre 1.13 format
#[derive(Deserialize, Default, Debug, PartialEq, Eq)]
struct OldStats {
    #[serde(rename = "stat.playOneMinute", default)]
    play_time: u64,
    #[serde(rename = "stat.jump", default)]
    jumps: u64,
    #[serde(rename = "stat.deaths", default)]
    deaths: u64,
    #[serde(rename = "stat.damageTaken", default)]
    damage_taken: u64,
    #[serde(rename = "stat.damageDealt", default)]
    damage_dealt: u64,
    #[serde(rename = "stat.mobKills", default)]
    mob_kills: u64,
    #[serde(rename = "stat.playerKills", default)]
    player_kills: u64,
    #[serde(rename = "stat.cakeSlicesEaten", default)]
    cake_slices: u64,
    #[serde(rename = "stat.leaveGame", default)]
    leave_game: u64,
    #[serde(rename = "stat.walkOneCm", default)]
    walk: u64,
    #[serde(rename = "stat.crouchOneCm", default)]
    crouch: u64,
    #[serde(rename = "stat.sprintOneCm", default)]
    sprint: u64,
    #[serde(rename = "stat.swimOneCm", default)]
    swim: u64,
    #[serde(rename = "stat.fallOneCm", default)]
    fall: u64,
    #[serde(rename = "stat.climbOneCm", default)]
    climb: u64,
    #[serde(rename = "stat.flyOneCm", default)]
    fly: u64,
    #[serde(rename = "stat.diveOneCm", default)]
    dive: u64,
    #[serde(rename = "stat.minecartOneCm", default)]
    minecart: u64,
    #[serde(rename = "stat.boatOneCm", default)]
    boat: u64,
    #[serde(rename = "stat.pigOneCm", default)]
    pig: u64,
    #[serde(rename = "stat.horseOneCm", default)]
    horse: u64,
    #[serde(rename = "stat.aviateOneCm", default)]
    aviate: u64,
}

impl Player {
    /// Create a Player struct using the path to the player's stats file as input
    fn new(path: &PathBuf) -> Result<Self> {
        let uuid = match path.file_stem() {
            Some(x) => x.to_str().expect("Invalid player uuid").to_string(),
            None => bail!("File name did not contain a uuid"),
        };

        let mut f = File::open(path)?;
        let mut tmp = String::new();
        f.read_to_string(&mut tmp)?;
        let mut ret: Player = serde_json::from_str(&tmp)?;

        let advancements_path = {
            let mut path = path.clone();
            assert!(path.pop()); /* assert so that we unwind if path is empty */
            assert!(path.pop()); /* and we pop twice to remove both uuid.json and stats */
            path.push("advancements");
            path.push(format!("{}.json", uuid));
            path
        };
        ret.advancements_count = count_advancements(&advancements_path)?;
        ret.uuid = uuid;
        ret.set_player_name()?;

        Ok(ret)
    }

    /// Set the player's name
    fn set_player_name(&mut self) -> Result<()> {
        let mut f = File::open(format!("./playerdata/{}.dat", self.uuid))
            .with_context(|_| format!("while trying to open playerdata file for player with uuid {}", self.uuid))?;
        let nbt = nbt::Blob::from_gzip(&mut f)
            .with_context(|_| format!("while trying to parse playerdata file for player with uuid {}", self.uuid))?;

        let nbt = match nbt["bukkit"] {
            nbt::Value::Compound(ref x) => x.clone(),
            _ => bail!("Could not find bukkit compound in NBT"),
        };
        let name = match nbt.get("lastKnownName") {
            Some(x) => x.clone(),
            None => bail!("lastKnownName not found in NBT"),
        };
        let name = match name {
            nbt::Value::String(ref x) => x.clone(),
            _ => bail!("lastKnownName had invalid type in NBT"),
        };

        self.playername = name;

        Ok(())
    }

    /// Sum all the travel stats to get the total distance traveled (in km)
    fn get_traveled_distance(&self) -> u64 {
        let a = &self.stats.custom;
        let b = &self.oldstats;
        (a.walk
            + a.crouch
            + a.sprint
            + a.swim
            + a.fall
            + a.climb
            + a.fly
            + a.walk_on_water
            + a.minecart
            + a.boat
            + a.pig
            + a.horse
            + a.aviate
            + b.walk
            + b.crouch
            + b.sprint
            + b.swim
            + b.fall
            + b.climb
            + b.fly
            + b.dive
            + b.minecart
            + b.boat
            + b.pig
            + b.horse
            + b.aviate
            ) / (100 * 1000)
    }
}

impl fmt::Display for Player {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,
               "| [[{playername}]] || {playtime} || {leavegame} || {jump} || {deaths} || {damagetaken} || {damagedealt} || {mobkills} || {playerkills} || {distance} || {cakeslices} || {advancements}/81 || {traded_with_villager} || {stonemined} || {obsidianmined} || {cobblestonemined} || {netherrackmined} || {spawnermined} || {ender_dragon} || {wither} || {elder_guardian} || {evoker} || {skeleton_horse} || {piglin_brute} || {dirtmined} || {sandmined}",
               playername=self.playername,
               playtime=(self.stats.custom.play_time + self.oldstats.play_time) / (20 * 60 * 60),
               leavegame=self.stats.custom.leave_game + self.oldstats.leave_game,
               jump=self.stats.custom.jumps + self.oldstats.jumps,
               deaths=self.stats.custom.deaths + self.oldstats.deaths,
               damagetaken=(self.stats.custom.damage_taken + self.oldstats.damage_taken) / 10,
               damagedealt=(self.stats.custom.damage_dealt + self.oldstats.damage_dealt) / 10,
               mobkills=self.stats.custom.mob_kills + self.oldstats.mob_kills,
               playerkills=self.stats.custom.player_kills + self.oldstats.player_kills,
               distance=self.get_traveled_distance(),
               cakeslices=self.stats.custom.cake_slices + self.oldstats.cake_slices,
               advancements=self.advancements_count,
               traded_with_villager=self.stats.custom.traded_with_villager,
               stonemined=self.stats.mined.stone,
               obsidianmined=self.stats.mined.obsidian,
               cobblestonemined=self.stats.mined.cobblestone,
               netherrackmined=self.stats.mined.netherrack,
               spawnermined=self.stats.mined.spawner,
               ender_dragon=self.stats.killed.ender_dragon,
               wither=self.stats.killed.wither,
               elder_guardian=self.stats.killed.elder_guardian,
               evoker=self.stats.killed.evoker,
               skeleton_horse=self.stats.killed.skeleton_horse,
               piglin_brute=self.stats.killed.piglin_brute
               dirtmined=self.stats.mined.dirt,
               sandmined=self.stats.mined.sand)
    }
}

impl Ord for Player {
    fn cmp(&self, other: &Player) -> Ordering {
        let a_playtime = self.stats.custom.play_time + self.oldstats.play_time;
        let b_playtime = other.stats.custom.play_time + other.oldstats.play_time;
        match b_playtime.cmp(&a_playtime) {
            Ordering::Equal => {
                self.uuid.cmp(&other.uuid)
            }
            x => x,
        }
    }
}
impl PartialOrd for Player {
    fn partial_cmp(&self, other: &Player) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
