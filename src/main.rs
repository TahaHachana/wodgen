mod csv_utils;

use crate::csv_utils::{read_csv, write_csv};
use anyhow::Result;
use chrono::Local;
use chrono::{DateTime, Utc};
use clap::Parser;
use log::info;
use rand::seq::SliceRandom;
use rand::{thread_rng, Rng};
use serde::{Deserialize, Serialize};
use simplelog::*;
use std::collections::HashMap;
use std::path::PathBuf;

// --------------------------------------------------

// Constants for file names and snooze period
const COOLDOWN_FILE: &str = "cooldown.csv";
const CORE_FILE: &str = "core.csv";
const LEGS_FILE: &str = "legs.csv";
const PULL_FILE: &str = "pull.csv";
const PUSH_FILE: &str = "push.csv";
const SNOOZED_FILE: &str = "snoozed.csv";

const SNOOZE_PERIOD: i64 = 7; // Snooze period in days

// --------------------------------------------------

// Enum for different exercise types
#[derive(Debug, PartialEq, Eq, Hash, Clone, Serialize, Deserialize, clap::ValueEnum)]
enum ExerciseType {
    Cooldown,
    Core,
    Legs,
    Pull,
    Push,
}

// Enum for different exercise categories
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
enum ExerciseCategory {
    Primary,
    Secondary,
    Accessory,
}

// Enum for different exercise levels
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize, clap::ValueEnum)]
enum ExerciseLevel {
    Beginner,
    Intermediate,
    Advanced,
}

// Enum for different exercise programming types
#[derive(Debug, Clone, Serialize, Deserialize)]
enum ExerciseProgramming {
    Distance,
    Reps,
    Time,
}

// Enum for rep schemes
// #[derive(Debug, Clone, Serialize, Deserialize)]
// enum RepScheme {
//     // 2 - 4 - 6 - 8 - 6 - 4 - 2
//     Pyramid,
//     // 8 - 6 - 4 - 2
//     ReversePyramid,
//     // 8 - 8 - 8
//     Straight,
//     // 1 - 2 - 3 - 4 - 5 - 4 - 3 - 2 - 1
//     Ladder,
//     // 5 - 4 - 3 - 2 - 1
//     DescendingLadder,
//     // 1 - 2 - 3 - 4 - 5
//     AscendingLadder,
//     // Instead of counting reps, you can base the ladder on time. For example, start with 20 seconds of an exercise, then rest, then 30 seconds, then 40 seconds, and so on
//     TimeBasedLadder,
//     // This involves performing two exercises back to back with no rest in between
//     Superset,
//     // This involves performing a set to failure, then reducing the weight and performing another set to failure
//     Dropset,
//     // This involves performing a set to failure, then resting for a short period before performing another set to failure
//     RestPause,
//     // This involves performing three different exercises back-to-back with no rest in between
//     TriSet,
//     // This involves performing four or more exercises back-to-back with no rest in between
//     GiantSet,
//     // perform as many reps as you can in a set period
//     AMRAP,
//     // Perform a set number of reps at the start of every minute
//     EMOM,
// }

// Struct to represent an exercise
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

// --------------------------------------------------

// fn random_rep_scheme() -> RepScheme {
//     let mut rng = thread_rng();
//     let schemes = vec![
//         RepScheme::Pyramid,
//         RepScheme::ReversePyramid,
//         RepScheme::Straight,
//         RepScheme::Ladder,
//         RepScheme::DescendingLadder,
//         RepScheme::AscendingLadder,
//         RepScheme::TimeBasedLadder,
//         RepScheme::Superset,
//         RepScheme::Dropset,
//         RepScheme::RestPause,
//         RepScheme::TriSet,
//         RepScheme::GiantSet,
//         RepScheme::AMRAP,
//         RepScheme::EMOM,
//     ];
//     schemes.choose(&mut rng).unwrap().clone()
// }

// Struct to represent a workout exercise
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
    // Create a WorkoutExercise from an Exercise
    fn from_exercise(group: u32, exercise: &Exercise) -> WorkoutExercise {
        let (distance, time, reps, sets) = match exercise.exercise_programming {
            ExerciseProgramming::Distance => (
                String::from("X"),
                String::new(),
                String::new(),
                String::new(),
            ),
            ExerciseProgramming::Reps => (
                String::new(),
                String::new(),
                String::from("X"),
                String::new(),
                // format!("{:?}", random_rep_scheme()),
            ),
            ExerciseProgramming::Time => (
                String::new(),
                String::from("X"),
                String::new(),
                String::new(),
            ),
        };

        WorkoutExercise {
            group,
            name: to_title_case(&exercise.name),
            sets,
            distance,
            time,
            reps,
            goal: exercise.goal.clone().unwrap_or_default(),
            video: exercise.video.clone(),
        }
    }
}

// Struct to represent a snoozed exercise
#[derive(Debug, Serialize, Deserialize)]
struct SnoozedExercise {
    name: String,
    #[serde(with = "chrono::serde::ts_seconds")]
    timestamp: DateTime<Utc>,
}

// --------------------------------------------------

// Command line arguments struct
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
        default_value = "./exercise_library"
    )]
    exercise_library_dir: PathBuf,

    /// Path to the workouts directory
    #[arg(short, long, value_name = "WORKOUTS_DIR", default_value = "./workouts")]
    workouts_dir: PathBuf,

    /// Whether to include only bodyweight exercises in the workout
    #[arg(short, long, value_name = "BODYWEIGHT", default_value = "true")]
    bodyweight: bool,
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
            e.exercise_level == ExerciseLevel::Intermediate
            // e.exercise_level == ExerciseLevel::Beginner
            //     || e.exercise_level == ExerciseLevel::Intermediate
        }
        ExerciseLevel::Advanced => {
            e.exercise_level == ExerciseLevel::Intermediate
                || e.exercise_level == ExerciseLevel::Advanced
        }
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

// Initialize the simplelog logger
fn init_logger() {
    CombinedLogger::init(vec![TermLogger::new(
        LevelFilter::Info,
        Config::default(),
        TerminalMode::Mixed,
        ColorChoice::Auto,
    )])
    .unwrap();
}

// --------------------------------------------------

// Map exercise types to their corresponding file paths
fn map_file_paths(exercise_library_dir: &PathBuf) -> HashMap<ExerciseType, PathBuf> {
    [
        (
            ExerciseType::Cooldown,
            exercise_library_dir.join(COOLDOWN_FILE),
        ),
        (ExerciseType::Core, exercise_library_dir.join(CORE_FILE)),
        (ExerciseType::Legs, exercise_library_dir.join(LEGS_FILE)),
        (ExerciseType::Pull, exercise_library_dir.join(PULL_FILE)),
        (ExerciseType::Push, exercise_library_dir.join(PUSH_FILE)),
    ]
    .iter()
    .cloned()
    .collect::<HashMap<_, _>>()
}

// --------------------------------------------------

// Load exercises from a CSV file
fn load_exercises(file_path: &PathBuf) -> Result<Vec<Exercise>> {
    let exercises = read_csv::<Exercise>(file_path.to_str().unwrap())?;
    info!("Loaded {} exercises from {:?}", exercises.len(), file_path);
    Ok(exercises)
}

// --------------------------------------------------

// Load snoozed exercises from a CSV file
fn load_snoozed_exercises(snoozed_file_path: &PathBuf) -> Result<Vec<SnoozedExercise>> {
    let now = Utc::now();
    let snoozed_exercises: Vec<SnoozedExercise> =
        read_csv::<SnoozedExercise>(snoozed_file_path.to_str().unwrap())?
            .into_iter()
            .filter(|e| now.signed_duration_since(e.timestamp).num_days() < SNOOZE_PERIOD)
            .collect();
    info!("Loaded {} snoozed exercises", snoozed_exercises.len());
    Ok(snoozed_exercises)
}

// --------------------------------------------------

// Load relevant exercises for the specified exercise types
fn load_relevant_exercises(
    exercise_types: &[ExerciseType],
    file_paths: &HashMap<ExerciseType, PathBuf>,
) -> Result<Vec<Exercise>> {
    let mut relevant_exercises = Vec::new();
    for t in exercise_types {
        if let Some(file_path) = file_paths.get(t) {
            let exercises = read_csv::<Exercise>(file_path.to_str().unwrap())?;
            info!("Loaded {} exercises for type {:?}", exercises.len(), t);
            relevant_exercises.extend(exercises);
        }
    }
    info!("Loaded {} exercises", relevant_exercises.len());
    Ok(relevant_exercises)
}

// --------------------------------------------------

// Filter exercises based on bodyweight flag and snoozed exercises
fn filter_exercises(
    relevant_exercises: &mut Vec<Exercise>,
    bodyweight: bool,
    snoozed_exercises: &[SnoozedExercise],
) {
    if bodyweight {
        relevant_exercises.retain(|e| e.bodyweight);
        info!(
            "Filtered out non-bodyweight exercises, {} exercies remaining",
            relevant_exercises.len()
        );
    }

    snoozed_exercises.iter().for_each(|snoozed| {
        relevant_exercises.retain(|e| e.name != snoozed.name);
    });
    info!(
        "Filtered out snoozed exercises, {} exercises remaining",
        relevant_exercises.len()
    );

    shuffle_vector(relevant_exercises);
    info!("Shuffled relevant exercises");
}

// --------------------------------------------------

// Generate a workout
fn generate_workout(
    relevant_exercises: &mut Vec<Exercise>,
    exercise_types: &[ExerciseType],
    exercise_level: &ExerciseLevel,
    num_groups: u32,
    snoozed_exercises: &mut Vec<SnoozedExercise>,
) -> Vec<WorkoutExercise> {
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
    for group in 0..num_groups {
        info!("Generating group {}", group + 1);
        let mut exercises_to_remove = Vec::new();
        for t in exercise_types {
            info!("Picking exercise of type {:?}", t);
            let exercise = relevant_exercises
                .iter()
                .filter(|e| filter_by_type(e, t))
                .filter(|e| filter_by_level(e, exercise_level))
                .filter(|e| filter_by_category(e, group, exercise_level, t))
                .next()
                .cloned();

            if let Some(exercise) = exercise {
                info!("Picked exercise {:?}", exercise);
                exercises_to_remove.push(exercise.name.clone());
                snoozed_exercises.push(SnoozedExercise {
                    name: exercise.name.clone(),
                    timestamp: Utc::now(),
                });
                let workout_exercise = WorkoutExercise::from_exercise(group + 2, &exercise);
                workout.push(workout_exercise);
            }
        }
        relevant_exercises.retain(|e| !exercises_to_remove.contains(&e.name));
    }

    workout
}

// --------------------------------------------------

// Add a cooldown exercise to the workout
fn add_cooldown_exercise(
    workout: &mut Vec<WorkoutExercise>,
    cooldown_exercises: &mut Vec<Exercise>,
    snoozed_exercises: &mut Vec<SnoozedExercise>,
    num_groups: u32,
) {
    let cooldown_exercise = remove_random(cooldown_exercises).unwrap();
    snoozed_exercises.push(SnoozedExercise {
        name: cooldown_exercise.name.clone(),
        timestamp: Utc::now(),
    });
    let workout_exercise = WorkoutExercise::from_exercise(num_groups + 2, &cooldown_exercise);
    workout.push(workout_exercise);
    info!(
        "Added cooldown exercise {} to workout",
        cooldown_exercise.name
    );
}

// --------------------------------------------------

// Save the workout to a CSV file
fn save_workout(workouts_dir: &PathBuf, workout: Vec<WorkoutExercise>) -> Result<()> {
    let date = Local::now().format("%Y_%m_%d").to_string();
    let file_name = workouts_dir.join(format!("{}.csv", date));
    write_csv(file_name.to_str().unwrap(), workout)?;
    info!("Saved workout to {}", file_name.to_str().unwrap());
    Ok(())
}

// --------------------------------------------------

// Update the snoozed exercises CSV file
fn update_snoozed_exercises(
    snoozed_file_path: &PathBuf,
    snoozed_exercises: Vec<SnoozedExercise>,
) -> Result<()> {
    write_csv(snoozed_file_path.to_str().unwrap(), snoozed_exercises)?;
    info!("Updated snoozed exercises");
    Ok(())
}

// --------------------------------------------------

// Main function
fn main() -> Result<()> {
    // Initialize the logger
    init_logger();

    let args = Args::parse();

    let exercise_types = args.types;
    info!("Exercise types: {:?}", exercise_types);
    let exercise_level = args.level;
    info!("Exercise level: {:?}", exercise_level);
    let num_groups = args.groups;
    info!("Number of groups: {:?}", num_groups);
    let bodyweight = args.bodyweight;
    info!("Bodyweight: {:?}", bodyweight);

    // Map exercise types to their corresponding file paths
    let file_paths = map_file_paths(&args.exercise_library_dir);

    let cooldown_file_path = file_paths.get(&ExerciseType::Cooldown).unwrap();
    let snoozed_file_path = args.exercise_library_dir.join(SNOOZED_FILE);

    // Load exercises
    let mut cooldown_exercises = load_exercises(cooldown_file_path)?;
    let mut snoozed_exercises = load_snoozed_exercises(&snoozed_file_path)?;

    // Filter out snoozed exercises from cooldown exercises
    cooldown_exercises.retain(|e| {
        !snoozed_exercises
            .iter()
            .any(|snoozed| snoozed.name == e.name)
    });

    let mut relevant_exercises = load_relevant_exercises(&exercise_types, &file_paths)?;

    // Filter exercises
    filter_exercises(&mut relevant_exercises, bodyweight, &snoozed_exercises);

    // Generate workout
    let mut workout = generate_workout(
        &mut relevant_exercises,
        &exercise_types,
        &exercise_level,
        num_groups,
        &mut snoozed_exercises,
    );

    // Add cooldown exercise
    add_cooldown_exercise(
        &mut workout,
        &mut cooldown_exercises,
        &mut snoozed_exercises,
        num_groups,
    );

    // Save the workout to a CSV file
    if !args.workouts_dir.exists() {
        std::fs::create_dir_all(&args.workouts_dir)?;
    }
    save_workout(&args.workouts_dir, workout)?;

    // Update snoozed exercises
    update_snoozed_exercises(&snoozed_file_path, snoozed_exercises)?;

    Ok(())
}

// --------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_exercises() -> Vec<Exercise> {
        vec![
            Exercise {
                name: String::from("Push Up"),
                exercise_type: ExerciseType::Push,
                exercise_category: ExerciseCategory::Primary,
                exercise_level: ExerciseLevel::Beginner,
                exercise_programming: ExerciseProgramming::Reps,
                bodyweight: true,
                goal: Some(String::from("Strength")),
                video: String::from("push_up.mp4"),
            },
            Exercise {
                name: String::from("Pull Up"),
                exercise_type: ExerciseType::Pull,
                exercise_category: ExerciseCategory::Primary,
                exercise_level: ExerciseLevel::Intermediate,
                exercise_programming: ExerciseProgramming::Reps,
                bodyweight: true,
                goal: Some(String::from("Strength")),
                video: String::from("pull_up.mp4"),
            },
            Exercise {
                name: String::from("Squat"),
                exercise_type: ExerciseType::Legs,
                exercise_category: ExerciseCategory::Primary,
                exercise_level: ExerciseLevel::Advanced,
                exercise_programming: ExerciseProgramming::Reps,
                bodyweight: false,
                goal: Some(String::from("Strength")),
                video: String::from("squat.mp4"),
            },
            Exercise {
                name: String::from("Plank"),
                exercise_type: ExerciseType::Core,
                exercise_category: ExerciseCategory::Secondary,
                exercise_level: ExerciseLevel::Beginner,
                exercise_programming: ExerciseProgramming::Time,
                bodyweight: true,
                goal: Some(String::from("Endurance")),
                video: String::from("plank.mp4"),
            },
        ]
    }

    // --------------------------------------------------

    #[test]
    fn test_filter_by_type() {
        let exercises = create_test_exercises();
        let push_exercises: Vec<&Exercise> = exercises
            .iter()
            .filter(|e| filter_by_type(e, &ExerciseType::Push))
            .collect();
        assert_eq!(push_exercises.len(), 1);
        assert_eq!(push_exercises[0].name, "Push Up");

        let pull_exercises: Vec<&Exercise> = exercises
            .iter()
            .filter(|e| filter_by_type(e, &ExerciseType::Pull))
            .collect();
        assert_eq!(pull_exercises.len(), 1);
        assert_eq!(pull_exercises[0].name, "Pull Up");
    }

    // --------------------------------------------------

    #[test]
    fn test_filter_by_level() {
        let exercises = create_test_exercises();
        let beginner_exercises: Vec<&Exercise> = exercises
            .iter()
            .filter(|e| filter_by_level(e, &ExerciseLevel::Beginner))
            .collect();
        assert_eq!(beginner_exercises.len(), 2);
        assert!(beginner_exercises.iter().any(|e| e.name == "Push Up"));
        assert!(beginner_exercises.iter().any(|e| e.name == "Plank"));

        let intermediate_exercises: Vec<&Exercise> = exercises
            .iter()
            .filter(|e| filter_by_level(e, &ExerciseLevel::Intermediate))
            .collect();
        assert_eq!(intermediate_exercises.len(), 3);
        assert!(intermediate_exercises.iter().any(|e| e.name == "Push Up"));
        assert!(intermediate_exercises.iter().any(|e| e.name == "Pull Up"));
        assert!(intermediate_exercises.iter().any(|e| e.name == "Plank"));

        let advanced_exercises: Vec<&Exercise> = exercises
            .iter()
            .filter(|e| filter_by_level(e, &ExerciseLevel::Advanced))
            .collect();
        assert_eq!(advanced_exercises.len(), 4);
    }

    // --------------------------------------------------

    #[test]
    fn test_filter_by_category() {
        let exercises = create_test_exercises();
        let primary_exercises: Vec<&Exercise> = exercises
            .iter()
            .filter(|e| filter_by_category(e, 0, &ExerciseLevel::Intermediate, &ExerciseType::Push))
            .collect();
        assert_eq!(primary_exercises.len(), 3);
        assert_eq!(primary_exercises[0].name, "Push Up");

        let secondary_exercises: Vec<&Exercise> = exercises
            .iter()
            .filter(|e| filter_by_category(e, 2, &ExerciseLevel::Intermediate, &ExerciseType::Core))
            .collect();
        assert_eq!(secondary_exercises.len(), 1);
        assert_eq!(secondary_exercises[0].name, "Plank");

        let accessory_exercises: Vec<&Exercise> = exercises
            .iter()
            .filter(|e| filter_by_category(e, 3, &ExerciseLevel::Advanced, &ExerciseType::Legs))
            .collect();
        assert_eq!(accessory_exercises.len(), 0);
    }
}
