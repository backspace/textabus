use regex::Regex;

pub fn parse_command(input: &str) -> Command {
    let cleaned_input = clean_input(input);

    if let Ok(command) = parse_stop_and_routes(&cleaned_input) {
        return Command::Times(command);
    }

    if let Ok(command) = parse_stops_and_location(&cleaned_input) {
        return Command::Stops(command);
    }

    if let Ok(command) = parse_help(&cleaned_input) {
        return Command::Help(command);
    }

    Command::Unknown(UnknownCommand {})
}

fn clean_input(input: &str) -> String {
    let string_of_whitespace = regex::Regex::new(r"\s+").unwrap();
    let normalised_input = string_of_whitespace.replace_all(&input, " ").to_string();

    let mut parts = normalised_input.trim().splitn(2, char::is_whitespace);
    let input_with_downcased_command =
        parts.next().unwrap_or("").to_lowercase() + " " + parts.next().unwrap_or("");

    let cleaned_input = input_with_downcased_command.trim().to_string();
    cleaned_input
}

fn parse_stop_and_routes(input: &str) -> Result<TimesCommand, &'static str> {
    let re = Regex::new(r"^(times )?(\d{5})(?:\s+(.*))?$").unwrap();

    if let Some(captures) = re.captures(input) {
        let stop_number = captures.get(2).map_or("", |m| m.as_str()).to_string();
        let routes: Vec<String> = captures
            .get(3)
            .map_or("", |m| m.as_str())
            .split_whitespace()
            .map(|s| s.to_string())
            .collect();
        Ok(TimesCommand {
            stop_number,
            routes,
        })
    } else {
        Err("Input string doesn't match the expected pattern")
    }
}

fn parse_stops_and_location(input: &str) -> Result<StopsCommand, &'static str> {
    let re = Regex::new(r"^stops\s+(.*)$").unwrap();

    if let Some(captures) = re.captures(input) {
        let location = captures.get(1).map_or("", |m| m.as_str()).to_string();
        Ok(StopsCommand { location })
    } else {
        Err("Input string does not match a stops request")
    }
}

fn parse_help(input: &str) -> Result<HelpCommand, &'static str> {
    let re = Regex::new(r"^help").unwrap();

    if let Some(_captures) = re.captures(input) {
        Ok(HelpCommand {})
    } else {
        Err("Input string does not match a help request")
    }
}

pub enum Command {
    Times(TimesCommand),
    Stops(StopsCommand),
    Help(HelpCommand),
    Unknown(UnknownCommand),
}

pub struct TimesCommand {
    pub stop_number: String,
    pub routes: Vec<String>,
}

pub struct StopsCommand {
    pub location: String,
}

pub struct HelpCommand;

pub struct UnknownCommand;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_times_command() {
        let command = parse_command("10619 16 18 BLUE");
        match command {
            Command::Times(times_command) => {
                assert_eq!(times_command.stop_number, "10619");
                assert_eq!(times_command.routes, vec!["16", "18", "BLUE"]);
            }
            _ => panic!("Expected TimesCommand"),
        }

        let command_with_whitespace = parse_command(" 10064 ");
        match command_with_whitespace {
            Command::Times(times_command) => {
                assert_eq!(times_command.stop_number, "10064");
                assert_eq!(times_command.routes, Vec::<String>::new());
            }
            _ => panic!("Expected TimesCommand"),
        }

        let command_with_optional_prefix = parse_command("times 10064");
        match command_with_optional_prefix {
            Command::Times(times_command) => {
                assert_eq!(times_command.stop_number, "10064");
                assert_eq!(times_command.routes, Vec::<String>::new());
            }
            _ => panic!("Expected TimesCommand"),
        }
    }

    #[test]
    fn test_parse_stops_command() {
        let command = parse_command("Stops 245 Smith");
        match command {
            Command::Stops(stops_command) => {
                assert_eq!(stops_command.location, "245 Smith");
            }
            _ => panic!("Expected StopsCommand"),
        }

        let command_with_line_breaks = parse_command("Stops\n245\nSmith");
        match command_with_line_breaks {
            Command::Stops(stops_command) => {
                assert_eq!(stops_command.location, "245 Smith");
            }
            _ => panic!("Expected StopsCommand"),
        }

        let command_with_extra_spaces = parse_command("stops  245   smith");
        match command_with_extra_spaces {
            Command::Stops(stops_command) => {
                assert_eq!(stops_command.location, "245 smith");
            }
            _ => panic!("Expected StopsCommand"),
        }
    }

    #[test]
    fn test_parse_unknown_command() {
        let command = parse_command("unknown command");
        match command {
            Command::Unknown(_) => (),
            _ => panic!("Expected UnknownCommand"),
        }
    }
}
