use clap::Parser;
use rand::seq::SliceRandom;
use rand::{thread_rng, Rng};
// use std::collections::HashMap;

// --------------------------------------------------
const EXERCISE_LIBRARY_DIR: &str = "exercise_library";
const COOLDOWN_FILE: &str = "cooldown.csv";
const CORE_FILE: &str = "core.csv";
const LEGS_FILE: &str = "legs.csv";
const PULL_FILE: &str = "pull.csv";
const PUSH_FILE: &str = "push.csv";

// --------------------------------------------------
#[derive(Debug, PartialEq, Clone)]
enum ExerciseType {
    Cooldown,
    Core,
    Legs,
    Pull,
    Push,
}

#[derive(Debug, PartialEq, Clone)]
enum ExerciseCategory {
    Primary,
    Secondary,
    Accessory,
}

#[derive(Debug, PartialEq, Clone)]
enum ExerciseLevel {
    Beginner,
    Intermediate,
    Advanced,
}

#[derive(Debug, Clone)]
enum ExerciseProgramming {
    Distance,
    Reps,
    Time,
}

#[derive(Debug, Clone)]
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

#[derive(Debug)]
struct WorkoutExercise {
    group: u32,
    exercise: Exercise,
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
// Parse a CSV file and return a vector of Exercise structs
fn parse_csv(file_path: &str) -> Vec<Exercise> {
    let mut exercises: Vec<Exercise> = Vec::new();
    let mut rdr = csv::Reader::from_path(file_path).unwrap();
    for result in rdr.records() {
        let record = result.unwrap();
        let name = record.get(0).unwrap().to_string();
        let exercise_type = match record.get(1).unwrap() {
            "cooldown" => ExerciseType::Cooldown,
            "core" => ExerciseType::Core,
            "legs" => ExerciseType::Legs,
            "pull" => ExerciseType::Pull,
            "push" => ExerciseType::Push,
            _ => panic!("Invalid exercise type"),
        };
        let exercise_category = match record.get(2).unwrap() {
            "primary" => ExerciseCategory::Primary,
            "secondary" => ExerciseCategory::Secondary,
            "accessory" => ExerciseCategory::Accessory,
            _ => panic!("Invalid exercise category"),
        };
        let exercise_level = match record.get(3).unwrap() {
            "beginner" => ExerciseLevel::Beginner,
            "intermediate" => ExerciseLevel::Intermediate,
            "advanced" => ExerciseLevel::Advanced,
            _ => panic!("Invalid exercise level"),
        };
        let exercise_programming = match record.get(4).unwrap() {
            "distance" => ExerciseProgramming::Distance,
            "reps" => ExerciseProgramming::Reps,
            "time" => ExerciseProgramming::Time,
            _ => panic!("Invalid exercise programming"),
        };
        let is_bodyweight = match record.get(5).unwrap() {
            "true" => true,
            "false" => false,
            _ => panic!("Invalid bodyweight value"),
        };
        let goal = match record.get(6) {
            None => None,
            Some(goal) => {
                if goal.is_empty() {
                    None
                } else {
                    Some(goal.to_string())
                }
            }
        };
        let video = record.get(7).unwrap().to_string();
        exercises.push(Exercise {
            name,
            exercise_type,
            exercise_category,
            exercise_level,
            exercise_programming,
            bodyweight: is_bodyweight,
            goal,
            video,
        });
    }
    exercises
}

// --------------------------------------------------
fn remove_random<T>(vec: &mut Vec<T>) -> Option<T> {
    if vec.is_empty() {
        None
    } else {
        let index = thread_rng().gen_range(0..vec.len());
        Some(vec.swap_remove(index))
    }
}

// --------------------------------------------------
// fn select_random_exercise(
//     exercises: &mut Vec<Exercise>,
//     exercise_level: Option<ExerciseLevel>,
// ) -> &Exercise {
//     let mut rng = thread_rng();
//     exercises.shuffle(&mut rng);

//     match exercise_level {
//         Some(level) => {
//             let test: Vec<&Exercise> = exercises.iter().filter(|&e| e.exercise_level == ExerciseLevel::Advanced).collect();
//             random_exercise(&test)
//         }
//         None => random_exercise(exercises),
//     }
// }

// --------------------------------------------------
fn main() {
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

    // A cooldown exercise is always included at the end
    let mut cooldown_exercises = parse_csv(&cooldown_file_path);

    let mut relevant_exercises = Vec::new();
    exercise_types.iter().for_each(|t| match t {
        ExerciseType::Core => relevant_exercises.extend(parse_csv(&core_file_path)),
        ExerciseType::Legs => relevant_exercises.extend(parse_csv(&legs_file_path)),
        ExerciseType::Pull => relevant_exercises.extend(parse_csv(&pull_file_path)),
        ExerciseType::Push => relevant_exercises.extend(parse_csv(&push_file_path)),
        // A cooldown exercise is always included at the end
        _ => (),
    });

    shuffle_vector(&mut relevant_exercises);

    let mut workout = Vec::<WorkoutExercise>::new();
    
    // Strength training
    for group in 0..args.groups {
        exercise_types.iter().for_each(|t| {
            let mut exercises_subset: Vec<&Exercise> = relevant_exercises
                .iter()
                .filter(|&e| e.exercise_type == *t)
                // Filter further if exercise_level is some
                .filter(|&e| match &exercise_level {
                    Some(level) => e.exercise_level == *level,
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
                let workout_exercise = WorkoutExercise {
                    group : group + 1,
                    exercise: exercise.clone(),
                };
                workout.push(workout_exercise);
            }
        });
    }
    
    // Add cooldown exercise
    let cooldown_exercise = remove_random(&mut cooldown_exercises).unwrap();
    let workout_exercise = WorkoutExercise {
        group: args.groups + 1,
        exercise: cooldown_exercise.clone(),
    };
    workout.push(workout_exercise);
    
    println!("{:?}", workout)
}
