use clap::{Arg, Command};
use std::collections::HashMap;

const EXERCISE_LIBRARY_DIR: &str = "exercise_library";
const COOLDOWN_FILE: &str = "cooldown.csv";
const CORE_FILE: &str = "core.csv";
const LEGS_FILE: &str = "legs.csv";
const PULL_FILE: &str = "pull.csv";
const PUSH_FILE: &str = "push.csv";

#[derive(Debug, PartialEq)]
enum ExerciseType {
    Cooldown,
    Core,
    Legs,
    Pull,
    Push,
}

#[derive(Debug, PartialEq)]
enum ExerciseLevel {
    Beginner,
    Intermediate,
    Advanced,
}

#[derive(Debug)]
enum ExerciseProgramming {
    Distance,
    Reps,
    Time,
}

#[derive(Debug)]
struct Exercise {
    name: String,
    exercise_type: ExerciseType,
    exercise_level: ExerciseLevel,
    exercise_programming: ExerciseProgramming,
    goal: Option<String>,
    video: String,
}

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
            _ => ExerciseType::Push,
        };
        let exercise_level = match record.get(2).unwrap() {
            "beginner" => ExerciseLevel::Beginner,
            "intermediate" => ExerciseLevel::Intermediate,
            _ => ExerciseLevel::Advanced,
        };
        let exercise_programming = match record.get(3).unwrap() {
            "distance" => ExerciseProgramming::Distance,
            "reps" => ExerciseProgramming::Reps,
            _ => ExerciseProgramming::Time,
        };
        let goal = match record.get(4) {
            None => None,
            Some(goal) => {
                if goal.is_empty() {
                    None
                } else {
                    Some(goal.to_string())
                }
            }
        };
        let video = record.get(5).unwrap().to_string();
        exercises.push(Exercise {
            name,
            exercise_type,
            exercise_level,
            exercise_programming,
            goal,
            video,
        });
    }
    exercises
}

fn main() {
    let matches = Command::new("wodgen")
        .version("1.0")
        .author("Taha Hachana <tahahachana@gmail.com>")
        .about("Generates a workout based on specified types and level")
        .arg(
            Arg::new("types")
                .short('t')
                .long("types")
                .value_name("TYPES")
                .help("A comma-separated list of exercise types (core, legs, pull, push)")
                .required(true)
                .num_args(0..),
        )
        // .arg(
        //     Arg::with_name("level")
        //         .short('l')
        //         .long("level")
        //         .value_name("LEVEL")
        //         .help("The exercise level (beginner, intermediate, advanced)")
        //         .takes_value(true)
        //         .required(true),
        // )
        .get_matches();

    let types: Vec<String> = matches.get_many("types").unwrap().cloned().collect();
    // let level = matches.value_of("level").unwrap();

    let exercise_types: Vec<ExerciseType> = types
        .iter()
        .map(|t| match t.as_str() {
            "core" => ExerciseType::Core,
            "legs" => ExerciseType::Legs,
            "pull" => ExerciseType::Pull,
            "push" => ExerciseType::Push,
            _ => panic!("Invalid exercise type"),
        })
        .collect();

    // let exercise_level = match level {
    //     "beginner" => ExerciseLevel::Beginner,
    //     "intermediate" => ExerciseLevel::Intermediate,
    //     "advanced" => ExerciseLevel::Advanced,
    //     _ => panic!("Invalid exercise level"),
    // };

    let cooldown_file_path = format!("{}/{}", EXERCISE_LIBRARY_DIR, COOLDOWN_FILE);
    let core_file_path = format!("{}/{}", EXERCISE_LIBRARY_DIR, CORE_FILE);
    let legs_file_path = format!("{}/{}", EXERCISE_LIBRARY_DIR, LEGS_FILE);
    let pull_file_path = format!("{}/{}", EXERCISE_LIBRARY_DIR, PULL_FILE);
    let push_file_path = format!("{}/{}", EXERCISE_LIBRARY_DIR, PUSH_FILE);

    let cooldown_exercises = parse_csv(&cooldown_file_path);
    
    let mut relevant_exercises = Vec::new();
    exercise_types
        .iter()
        .for_each(|t| match t {
            ExerciseType::Core => relevant_exercises.extend(parse_csv(&core_file_path)),
            ExerciseType::Legs => relevant_exercises.extend(parse_csv(&legs_file_path)),
            ExerciseType::Pull => relevant_exercises.extend(parse_csv(&pull_file_path)),
            ExerciseType::Push => relevant_exercises.extend(parse_csv(&push_file_path)),
            // Cooldown is always included at the end
            _ => ()
        });

    let selected_exercises: Vec<&Exercise> = relevant_exercises
        .iter()
        .filter(|&e| exercise_types.contains(&e.exercise_type)) // && e.exercise_level == exercise_level)
        .collect();

    for exercise in selected_exercises {
        println!("{:?}", exercise);
    }
}
