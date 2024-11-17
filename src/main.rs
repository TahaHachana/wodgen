mod csv_utils;

use crate::csv_utils::{read_csv, write_csv};
use anyhow::{Context, Result};
use chrono::Local;
use chrono::{DateTime, Utc};
use clap::Parser;
use csv::{Reader, Writer};
use log::info;
use rand::seq::SliceRandom;
use rand::{thread_rng, Rng};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use simplelog::*;
use std::collections::HashMap;
use std::fs::File;
use std::path::PathBuf;

// --------------------------------------------------
// const EXERCISE_LIBRARY_DIR: &str = "/home/taha/Documents/training/exercise_library";
// const WORKOUTS_DIR: &str = "/home/taha/Documents/training/workouts";

const COOLDOWN_FILE: &str = "cooldown.csv";
const CORE_FILE: &str = "core.csv";
const LEGS_FILE: &str = "legs.csv";
const PULL_FILE: &str = "pull.csv";
const PUSH_FILE: &str = "push.csv";
const SNOOZED_FILE: &str = "snoozed.csv";

const SNOOZE_PERIOD: i64 = 7;

// --------------------------------------------------
#[derive(Debug, PartialEq, Eq, Hash, Clone, Serialize, Deserialize, clap::ValueEnum)]
enum ExerciseType {
    Cooldown,
    Core,
    Legs,
    Pull,
    Push,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
enum ExerciseCategory {
    Primary,
    Secondary,
    Accessory,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize, clap::ValueEnum)]
enum ExerciseLevel {
    Beginner,
    Intermediate,
    Advanced,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
enum ExerciseProgramming {
    Distance,
    Reps,
    Time,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
struct Exercise {
    name: String,
    exercise_type: ExerciseType,
    exercise_category: ExerciseCategory,
    exercise_level: ExerciseLevel,
    exercise_programming: ExerciseProgramming,
    bodyweight: bool,
    goal: Option<String>,
    video: String,
}

#[derive(Debug, Serialize)]
#[allow(dead_code)]
struct WorkoutExercise {
    group: u32,
    name: String,
    sets: String,
    distance: String,
    time: String,
    reps: String,
    goal: String,
    video: String,
}

impl WorkoutExercise {
    fn from_exercise(group: u32, exercise: &Exercise) -> WorkoutExercise {
        let (distance, time, reps) = match exercise.exercise_programming {
            ExerciseProgramming::Distance => (String::from("X"), String::new(), String::new()),
            ExerciseProgramming::Reps => (String::new(), String::new(), String::from("X")),
            ExerciseProgramming::Time => (String::new(), String::from("X"), String::new()),
        };

        WorkoutExercise {
            group,
            name: to_title_case(&exercise.name),
            sets: String::new(),
            distance,
            time,
            reps,
            goal: exercise.goal.clone().unwrap_or_default(),
            video: exercise.video.clone(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct SnoozedExercise {
    name: String,
    #[serde(with = "chrono::serde::ts_seconds")]
    timestamp: DateTime<Utc>,
}

// --------------------------------------------------
#[derive(Debug, Parser)]
#[command(author, version, about)]
/// Workout generator based on specified types and level
struct Args {
    /// Exercise types to include in the workout, e.g., core, legs, pull, push
    #[arg(
        short,
        long,
        value_name = "TYPES",
        required = true,
        num_args = 1..,
        value_parser = clap::builder::EnumValueParser::<ExerciseType>::new(),
    )]
    types: Vec<ExerciseType>,

    /// Number of super-sets to include in the workout
    #[arg(short, long, value_name = "GROUPS", default_value = "2")]
    groups: u32,

    /// Level of difficulty for the workout
    #[arg(
        short,
        long,
        value_name = "LEVEL",
        default_value = "intermediate",
        value_parser = clap::builder::EnumValueParser::<ExerciseLevel>::new(),
    )]
    level: ExerciseLevel,

    /// Path to the exercise library directory
    #[arg(
        short,
        long,
        value_name = "EXERCISE_LIBRARY_DIR",
        default_value = "/home/taha/Documents/training/exercise_library"
    )]
    exercise_library_dir: PathBuf,

    /// Path to the workouts directory
    #[arg(
        short,
        long,
        value_name = "WORKOUTS_DIR",
        default_value = "/home/taha/Documents/training/workouts"
    )]
    workouts_dir: PathBuf,
}

// --------------------------------------------------
// Shuffle a vector in place
fn shuffle_vector<T>(vec: &mut Vec<T>) {
    let mut rng = thread_rng();
    vec.shuffle(&mut rng);
}

// --------------------------------------------------
// Remove a random element from a vector
fn remove_random<T>(vec: &mut Vec<T>) -> Option<T> {
    if vec.is_empty() {
        None
    } else {
        let index = thread_rng().gen_range(0..vec.len());
        Some(vec.swap_remove(index))
    }
}

// --------------------------------------------------
// For pretty printing the exercise names
fn to_title_case(input: &str) -> String {
    input
        .replace("__", " - ")
        .replace('_', " ")
        .split_whitespace()
        .map(|word| {
            let mut c = word.chars();
            match c.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().collect::<String>() + c.as_str(),
            }
        })
        .collect::<Vec<String>>()
        .join(" ")
}

// --------------------------------------------------
// Filter exercises by type
fn filter_by_type(e: &Exercise, t: &ExerciseType) -> bool {
    e.exercise_type == *t
}

// Filter exercises by level
fn filter_by_level(e: &Exercise, l: &ExerciseLevel) -> bool {
    match l {
        ExerciseLevel::Beginner => e.exercise_level == ExerciseLevel::Beginner,
        ExerciseLevel::Intermediate => {
            e.exercise_level == ExerciseLevel::Beginner
                || e.exercise_level == ExerciseLevel::Intermediate
        }
        ExerciseLevel::Advanced => true,
    }
}

// Filter exercises by category
fn filter_by_category(e: &Exercise, g: u32, l: &ExerciseLevel, t: &ExerciseType) -> bool {
    match g {
        0 => match l {
            ExerciseLevel::Beginner => e.exercise_category == ExerciseCategory::Secondary,
            _ => e.exercise_category == ExerciseCategory::Primary,
        },
        1 => {
            e.exercise_category == ExerciseCategory::Primary
                || e.exercise_category == ExerciseCategory::Secondary
        }
        2 => match t {
            ExerciseType::Core => e.exercise_category == ExerciseCategory::Secondary,
            _ => {
                e.exercise_category == ExerciseCategory::Secondary
                    || e.exercise_category == ExerciseCategory::Accessory
            }
        },
        3.. => match t {
            ExerciseType::Core => e.exercise_category == ExerciseCategory::Secondary,
            _ => e.exercise_category == ExerciseCategory::Accessory,
        },
    }
}

// --------------------------------------------------
fn main() -> Result<()> {
    // Initialize the logger
    CombinedLogger::init(vec![
        TermLogger::new(LevelFilter::Info, Config::default(), TerminalMode::Mixed, ColorChoice::Auto),
        WriteLogger::new(LevelFilter::Info, Config::default(), File::create("app.log").unwrap()),
    ]).unwrap();

    let args = Args::parse();

    let exercise_types = args.types;
    info!("Exercise types: {:?}", exercise_types);
    let exercise_level = args.level;
    info!("Exercise level: {:?}", exercise_level);

    let file_paths = [
        (
            ExerciseType::Cooldown,
            args.exercise_library_dir.join(COOLDOWN_FILE),
        ),
        (
            ExerciseType::Core,
            args.exercise_library_dir.join(CORE_FILE),
        ),
        (
            ExerciseType::Legs,
            args.exercise_library_dir.join(LEGS_FILE),
        ),
        (
            ExerciseType::Pull,
            args.exercise_library_dir.join(PULL_FILE),
        ),
        (
            ExerciseType::Push,
            args.exercise_library_dir.join(PUSH_FILE),
        ),
    ]
    .iter()
    .cloned()
    .collect::<HashMap<_, _>>();

    let cooldown_file_path = file_paths.get(&ExerciseType::Cooldown).unwrap();
    let snoozed_file_path = args.exercise_library_dir.join(SNOOZED_FILE);

    // A cooldown exercise is always included at the end
    let mut cooldown_exercises = read_csv::<Exercise>(cooldown_file_path.to_str().unwrap())?;
    info!("Loaded {} cooldown exercises", cooldown_exercises.len());

    // Read snoozed exercises and filter out those that are still within the snooze period
    let now = Utc::now();
    let mut snoozed_exercises: Vec<SnoozedExercise> =
        read_csv::<SnoozedExercise>(snoozed_file_path.to_str().unwrap())?
            .into_iter()
            .filter(|e| now.signed_duration_since(e.timestamp).num_days() < SNOOZE_PERIOD)
            .collect();
    info!("Loaded {} snoozed exercises", snoozed_exercises.len());

    let mut relevant_exercises = Vec::new();

    for t in &exercise_types {
        if let Some(file_path) = file_paths.get(t) {
            let exercises = read_csv::<Exercise>(file_path.to_str().unwrap())?;
            info!("Loaded {} exercises for type {:?}", exercises.len(), t);
            relevant_exercises.extend(exercises);
        }
    }

    // Remove snoozed exercises from relevant_exercises
    snoozed_exercises.iter().for_each(|snoozed| {
        relevant_exercises.retain(|e| e.name != snoozed.name);
    });
    info!("Filtered out snoozed exercises, {} exercises remaining", relevant_exercises.len());

    shuffle_vector(&mut relevant_exercises);
    info!("Shuffled relevant exercises");

    let mut workout = Vec::<WorkoutExercise>::new();

    // Skill block placeholder
    workout.push(WorkoutExercise {
        group: 1,
        name: String::from("Skill Block"),
        sets: String::new(),
        distance: String::new(),
        time: String::new(),
        reps: String::new(),
        goal: String::new(),
        video: String::new(),
    });

    // Strength training block
    for group in 0..args.groups {
        let mut exercises_to_remove = Vec::new();
        for t in &exercise_types {
            let mut exercises_subset: Vec<&Exercise> = relevant_exercises
                .iter()
                // Filter by exercise type
                .filter(|e| filter_by_type(e, t))
                // Filter by exercise level
                .filter(|&e| filter_by_level(e, &exercise_level))
                // Start with primary exercises then secondary and finally accessory
                .filter(|&e| filter_by_category(e, group, &exercise_level, t))
                .collect();

            if let Some(exercise) = remove_random(&mut exercises_subset) {
                // Mark the exercise for removal from the relevant_exercises vector
                exercises_to_remove.push(exercise.name.clone());
                // Add the exercise to the snoozed_exercises vector
                snoozed_exercises.push(SnoozedExercise {
                    name: exercise.name.clone(),
                    timestamp: Utc::now(),
                });
                let workout_exercise = WorkoutExercise::from_exercise(group + 2, exercise);
                workout.push(workout_exercise);
                info!("Added exercise {} to workout", exercise.name);
            }
        }
        // To not select the same exercise twice
        relevant_exercises.retain(|e| !exercises_to_remove.contains(&e.name));
    }

    // Cooldown block
    let cooldown_exercise = remove_random(&mut cooldown_exercises).unwrap();
    // Snooze the cooldown exercise
    snoozed_exercises.push(SnoozedExercise {
        name: cooldown_exercise.name.clone(),
        timestamp: Utc::now(),
    });
    let workout_exercise = WorkoutExercise::from_exercise(args.groups + 2, &cooldown_exercise);
    workout.push(workout_exercise);
    info!("Added cooldown exercise {} to workout", cooldown_exercise.name);
    
    println!("{:?}", workout);
    
    // Save the workout to a csv file
    let date = Local::now().format("%Y_%m_%d").to_string();
    let file_name = args.workouts_dir.join(format!("{}.csv", date));
    // write_csv(file_name.to_str().unwrap(), workout)?;
    info!("Saved workout to {}", file_name.to_str().unwrap());

    // Override the snoozed exercises CSV with the updated snoozed exercises
    // write_csv(snoozed_file_path.to_str().unwrap(), snoozed_exercises)?;
    info!("Updated snoozed exercises");

    Ok(())
}

// todo: add unit tests for exercise filtering logic
// toco: document the code
