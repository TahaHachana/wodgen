use anyhow::{Context, Result};
use chrono::Local;
use chrono::{DateTime, Utc};
use clap::Parser;
use csv::{Reader, Writer};
use rand::seq::SliceRandom;
use rand::{thread_rng, Rng};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::fs::File;

// --------------------------------------------------
const EXERCISE_LIBRARY_DIR: &str = "/home/taha/Documents/training/exercise_library";
const WORKOUTS_DIR: &str = "/home/taha/Documents/training/workouts";
const COOLDOWN_FILE: &str = "cooldown.csv";
const CORE_FILE: &str = "core.csv";
const LEGS_FILE: &str = "legs.csv";
const PULL_FILE: &str = "pull.csv";
const PUSH_FILE: &str = "push.csv";
const SNOOZED_FILE: &str = "snoozed.csv";
const SNOOZE_PERIOD: i64 = 7;

// --------------------------------------------------
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
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

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
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
            name: to_title_case(exercise.name.clone()),
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
pub fn read_csv<T: DeserializeOwned>(file: &str) -> Result<Vec<T>> {
    // Open the CSV file
    let file = File::open(file).with_context(|| format!("Failed to open file: {}", file))?;

    // Create a CSV reader from the file
    let mut rdr = Reader::from_reader(file);

    // Deserialize each record into a T struct and collect them into a vector
    let mut records = Vec::new();
    for result in rdr.deserialize() {
        let record: T = result.with_context(|| "Failed to deserialize record")?;
        records.push(record);
    }
    Ok(records)
}

// --------------------------------------------------
pub fn write_csv<T: serde::Serialize>(file: &str, data: Vec<T>) -> Result<()> {
    // Create a CSV writer
    let mut wtr = Writer::from_path(file)?;

    // Serialize each record into CSV and write it to the file
    for record in data {
        wtr.serialize(record)
            .with_context(|| "Failed to serialize record")?;
    }
    wtr.flush()?;
    Ok(())
}

// --------------------------------------------------
#[derive(Debug, Parser)]
#[command(author, version, about)]
/// Workout generator based on specified types and level
struct Args {
    #[arg(short, long, value_name = "TYPES", required = true, num_args = 1..)]
    types: Vec<String>,

    #[arg(short, long, value_name = "GROUPS", default_value = "2")]
    groups: u32,

    #[arg(short, long, value_name = "LEVEL")]
    level: Option<String>,
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
fn to_title_case(input: String) -> String {
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
fn main() -> Result<()> {
    let args = Args::parse();

    let exercise_types: Vec<ExerciseType> = args
        .types
        .iter()
        .map(|t| match t.as_str() {
            "core" => ExerciseType::Core,
            "legs" => ExerciseType::Legs,
            "pull" => ExerciseType::Pull,
            "push" => ExerciseType::Push,
            _ => panic!("Invalid exercise type"),
        })
        .collect();

    let exercise_level = match args.level.as_deref() {
        Some("beginner") => Some(ExerciseLevel::Beginner),
        Some("intermediate") => Some(ExerciseLevel::Intermediate),
        Some("advanced") => Some(ExerciseLevel::Advanced),
        _ => None,
    };

    let cooldown_file_path = format!("{}/{}", EXERCISE_LIBRARY_DIR, COOLDOWN_FILE);
    let core_file_path = format!("{}/{}", EXERCISE_LIBRARY_DIR, CORE_FILE);
    let legs_file_path = format!("{}/{}", EXERCISE_LIBRARY_DIR, LEGS_FILE);
    let pull_file_path = format!("{}/{}", EXERCISE_LIBRARY_DIR, PULL_FILE);
    let push_file_path = format!("{}/{}", EXERCISE_LIBRARY_DIR, PUSH_FILE);
    let snoozed_file_path = format!("{}/{}", EXERCISE_LIBRARY_DIR, SNOOZED_FILE);

    // A cooldown exercise is always included at the end
    let mut cooldown_exercises = read_csv::<Exercise>(&cooldown_file_path)?;

    // Read snoozed exercises and filter out those that are still within the snooze period
    let mut snoozed_exercises = read_csv::<SnoozedExercise>(&snoozed_file_path)?
        .into_iter()
        .filter(|e| {
            let now = Utc::now();
            now.signed_duration_since(e.timestamp).num_days() < SNOOZE_PERIOD
        })
        .collect::<Vec<_>>();

    let mut relevant_exercises = Vec::new();

    for t in &exercise_types {
        match t {
            ExerciseType::Core => relevant_exercises.extend(read_csv::<Exercise>(&core_file_path)?),
            ExerciseType::Legs => relevant_exercises.extend(read_csv::<Exercise>(&legs_file_path)?),
            ExerciseType::Pull => relevant_exercises.extend(read_csv::<Exercise>(&pull_file_path)?),
            ExerciseType::Push => relevant_exercises.extend(read_csv::<Exercise>(&push_file_path)?),
            // A cooldown exercise is always included at the end
            _ => (),
        }
    }

    // Remove snoozed exercises from relevant_exercises
    snoozed_exercises.iter().for_each(|snoozed| {
        relevant_exercises.retain(|e| e.name != snoozed.name);
    });

    shuffle_vector(&mut relevant_exercises);

    let mut workout = Vec::<WorkoutExercise>::new();

    // Skill block placeholder
    workout.push(WorkoutExercise {
        group: 1,
        name: "Skill Block".to_string(),
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
                .filter(|&e| e.exercise_type == *t)
                // Filter further if exercise_level is some
                .filter(|&e| match &exercise_level {
                    Some(level) => match *level {
                        ExerciseLevel::Beginner => e.exercise_level == ExerciseLevel::Beginner,
                        // Intermediate includes beginner
                        ExerciseLevel::Intermediate => {
                            e.exercise_level == ExerciseLevel::Beginner
                                || e.exercise_level == ExerciseLevel::Intermediate
                        }
                        // Advanced includes intermediate and beginner
                        ExerciseLevel::Advanced => true,
                    },
                    None => true,
                })
                // Start with primary exercises then secondary and finally accessory
                .filter(|&e| match group {
                    0 => e.exercise_category == ExerciseCategory::Primary,
                    1 => {
                        e.exercise_category == ExerciseCategory::Primary
                            || e.exercise_category == ExerciseCategory::Secondary
                    }
                    2 => match t {
                        // No accessory exercises for core
                        ExerciseType::Core => e.exercise_category == ExerciseCategory::Secondary,
                        _ => {
                            e.exercise_category == ExerciseCategory::Secondary
                                || e.exercise_category == ExerciseCategory::Accessory
                        }
                    },
                    3.. => match t {
                        // No accessory exercises for core
                        ExerciseType::Core => e.exercise_category == ExerciseCategory::Secondary,
                        _ => e.exercise_category == ExerciseCategory::Accessory,
                    },
                })
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

    // Save the workout to a csv file
    let date = Local::now().format("%Y_%m_%d").to_string();
    let file_name = format!("{}/{}.csv", WORKOUTS_DIR, date);
    write_csv(&file_name, workout)?;

    // Override the snoozed exercises CSV with the updated snoozed exercises
    write_csv(&snoozed_file_path, snoozed_exercises)?;

    Ok(())
}
